// Generic accessibility-tree-based page reading — design doc §7.3. Replaces v2's hardcoded
// per-question-type EXTRACTOR_JS. Uses CDP's Accessibility domain (not a bespoke selector list),
// which is what fixes all three confirmed v2 bugs (design doc §7.1):
//   1. A question with no heading text but an image still produces a `PageField` — field
//      creation is keyed on the structural boundary (a `listitem`-role node), never on whether
//      any label text was found inside it.
//   2. Inline/`data:`-URI images are never filtered — there is no branch anywhere below that
//      inspects an image URL's scheme/prefix before including it.
//   3. Page-level heading/paragraph text encountered before the first question boundary is
//      collected into `PageContent::page_context`, not dropped.
//
// API surface verified against chromiumoxide 0.9.1's actual generated CDP bindings (read
// directly from the crate's source — `chromiumoxide_cdp-0.9.1/src/cdp.rs`'s `accessibility`
// module — not guessed): `Page::execute` takes any `T: chromiumoxide_types::Command` and returns
// `CommandResponse<T::Response>` (deref's to the response, e.g. `.result.nodes` /
// `.nodes` are equivalent). The PDL command `getFullAXTree` compiles (via chromiumoxide's
// `heck::ToUpperCamelCase` codegen convention, confirmed against the checked-in generated
// source rather than assumed) to `GetFullAxTreeParams` / `GetFullAxTreeReturns { nodes: Vec<AxNode> }`
// — note the acronym `AX` becomes `Ax`, not `AX`, in every generated identifier
// (`AxNode`, `AxNodeId`, `AxValue`, `AxProperty`, `AxPropertyName`), which is easy to get wrong
// by guessing and was confirmed directly against source instead.
//
// Image URLs (including inline `data:` URIs) come from `AxNode.properties`' `AxPropertyName::Url`
// entry — the CDP `Accessibility` PDL schema documents `url` as a valid `AXPropertyName` enum
// value (`chromiumoxide_cdp-0.9.1/pdl/domains/Accessibility.pdl`), and Chromium's accessibility
// tree computes this as the resolved image source for both hosted and inline images alike. This
// is what lets this implementation stay purely within the Accessibility domain (no correlated
// `DOM.describeNode` call needed to read a `src` attribute) — matching the design doc's framing
// that the accessibility-tree approach makes "inline vs. hosted images stop being structurally
// different" (§7.3).
//
// Genuine open uncertainty, not verified against a live browser (no Chrome available in this
// environment — see the milestone's own constraints): the exact `AXRole` strings Chromium emits
// for Google Forms' specific DOM structure (`listitem`, `heading`, `image`, `radio`, `checkbox`,
// `textbox`, `row` are all standard ARIA-role-shaped strings and are the ones matched below, but
// their exact casing/spelling in Chromium's real serialized output — e.g. whether text runs come
// through as `"StaticText"` verbatim — is assumed from general CDP/DevTools knowledge, not
// confirmed against this specific page). Role comparisons below are done case-insensitively for
// resilience against that uncertainty. See the unit tests for the hand-built AX-tree-shaped JSON
// this logic is proven against instead.

use std::collections::HashMap;

use chromiumoxide::cdp::browser_protocol::accessibility::{
    AxNode, AxNodeId, AxPropertyName, EnableParams, GetFullAxTreeParams,
};
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};

/// One question/field found on the current page, however it was structurally represented —
/// deliberately loose/generic rather than the old per-type enum, since the whole point of the
/// accessibility-tree approach is not needing to hardcode "this is a radio, this is a grid."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageField {
    pub id: String,
    pub label: String,
    pub kind_hint: String, // e.g. "radio", "checkbox", "text", "grid" — a hint, not an exhaustive enum
    pub options: Vec<String>,
    pub rows: Vec<String>,
    pub current_value: Option<String>,
    pub image_urls: Vec<String>,
}

impl PageField {
    fn new(id: String) -> Self {
        PageField {
            id,
            label: String::new(),
            kind_hint: String::new(),
            options: Vec::new(),
            rows: Vec::new(),
            current_value: None,
            image_urls: Vec::new(),
        }
    }
}

/// Everything read from the current page: shared context (page title/description — the thing
/// v2's extractor never read at all, design doc §7.1) plus every field found.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContent {
    pub page_context: String,
    pub fields: Vec<PageField>,
}

pub async fn read_page(page: &Page) -> anyhow::Result<PageContent> {
    // `Accessibility.enable` — per the CDP doc comment on the command itself, this "causes
    // AXNodeIds to remain consistent between method calls." Not strictly required for a single
    // getFullAXTree snapshot, but cheap, harmless, and keeps node ids stable if this page is read
    // again later in the same walk-forward loop (design doc §7.3).
    page.execute(EnableParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("Accessibility.enable failed: {e}"))?;

    let resp = page
        .execute(GetFullAxTreeParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("Accessibility.getFullAXTree failed: {e}"))?;

    Ok(build_page_content(&resp.result.nodes))
}

/// Pure parsing logic over an already-fetched AX node list. Kept separate from `read_page` so it
/// can be exercised in unit tests against hand-built AX-tree-shaped data without a live CDP
/// connection — see the `tests` module below, which covers the three design-doc §7.1 bugs plus
/// page-level context extraction.
fn build_page_content(nodes: &[AxNode]) -> PageContent {
    let by_id: HashMap<&AxNodeId, &AxNode> = nodes.iter().map(|n| (&n.node_id, n)).collect();
    let children_of = |id: &AxNodeId| -> Vec<&AxNode> {
        by_id
            .get(id)
            .and_then(|n| n.child_ids.as_ref())
            .map(|ids| ids.iter().filter_map(|cid| by_id.get(cid).copied()).collect())
            .unwrap_or_default()
    };

    // `getFullAXTree` returns a flat list forming one tree via `parent_id`/`child_ids`; the root
    // (document/RootWebArea) is the one node with no parent.
    let Some(root) = nodes.iter().find(|n| n.parent_id.is_none()) else {
        return PageContent { page_context: String::new(), fields: Vec::new() };
    };

    let mut page_context_parts: Vec<String> = Vec::new();
    let mut fields: Vec<PageField> = Vec::new();
    walk(root, &children_of, false, &mut page_context_parts, &mut fields);

    PageContent { page_context: page_context_parts.join("\n").trim().to_string(), fields }
}

fn role_of(node: &AxNode) -> Option<String> {
    node.role.as_ref()?.value.as_ref()?.as_str().map(|s| s.to_ascii_lowercase())
}

fn is_role(node: &AxNode, candidates: &[&str]) -> bool {
    role_of(node).is_some_and(|r| candidates.contains(&r.as_str()))
}

fn name_of(node: &AxNode) -> Option<String> {
    node.name
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// The image's resolved URL/data-URI, read from the `url` accessibility property (see the
/// module-level comment for why this needs no separate DOM lookup, and structurally can't drop
/// `data:` URIs the way v2's old scraping-hygiene filter did — design doc §7.1 bug #2).
fn url_property(node: &AxNode) -> Option<String> {
    node.properties
        .as_ref()?
        .iter()
        .find(|p| p.name == AxPropertyName::Url)
        .and_then(|p| p.value.value.as_ref())
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Best-effort "is this control currently selected/checked" read, from the `checked` property
/// (an `AXValueType::Tristate`/boolean per the PDL schema — `true`/`"true"`/`"mixed"` all count
/// as "has a selection" for the purposes of the already-answered check downstream).
fn is_checked(node: &AxNode) -> bool {
    let Some(props) = node.properties.as_ref() else { return false };
    let Some(checked) = props.iter().find(|p| p.name == AxPropertyName::Checked) else {
        return false;
    };
    match checked.value.value.as_ref() {
        Some(v) if v.as_bool() == Some(true) => true,
        Some(v) => matches!(v.as_str(), Some("true") | Some("mixed")),
        None => false,
    }
}

const HEADING_ROLES: &[&str] = &["heading"];
const CONTEXT_TEXT_ROLES: &[&str] = &["paragraph", "statictext"];
const FIELD_BOUNDARY_ROLES: &[&str] = &["listitem"];
const IMAGE_ROLES: &[&str] = &["image", "img"];
const OPTION_ROLES: &[&str] = &["radio", "checkbox", "menuitemradio", "menuitemcheckbox"];
const ROW_ROLES: &[&str] = &["row"];
const TEXT_INPUT_ROLES: &[&str] = &["textbox", "combobox", "searchbox"];

/// Walks the AX tree in document order. `inside_field` tracks whether we're already inside a
/// question's `listitem` subtree — once true, nested `walk` calls only ever gather into that same
/// field (via `collect_field`) and never start a second field partway through one question.
fn walk<'a>(
    node: &'a AxNode,
    children_of: &impl Fn(&'a AxNodeId) -> Vec<&'a AxNode>,
    inside_field: bool,
    page_context: &mut Vec<String>,
    fields: &mut Vec<PageField>,
) {
    if node.ignored {
        // Ignored nodes (decorative wrappers, presentational containers) can still have
        // non-ignored descendants — a real question can be nested inside one. Keep descending.
        for child in children_of(&node.node_id) {
            walk(child, children_of, inside_field, page_context, fields);
        }
        return;
    }

    if !inside_field && is_role(node, FIELD_BOUNDARY_ROLES) {
        // Field creation is unconditional on reaching this structural boundary — this is what
        // makes design-doc §7.1 bug #1 (heading-less image questions silently skipped)
        // structurally impossible: there is no "if no label text, skip" branch anywhere here.
        let mut field = PageField::new(node.node_id.inner().clone());
        collect_field(node, children_of, &mut field);
        fields.push(field);
        return;
    }

    if !inside_field && (is_role(node, HEADING_ROLES) || is_role(node, CONTEXT_TEXT_ROLES)) {
        if let Some(text) = name_of(node) {
            page_context.push(text);
        }
    }

    for child in children_of(&node.node_id) {
        walk(child, children_of, inside_field, page_context, fields);
    }
}

/// Gathers everything within one question's subtree into `field`. Recurses through the whole
/// subtree (including nested groups/rows) but never treats a nested `listitem` as the start of a
/// new top-level field — Google Forms' grid questions can nest row-shaped structure inside one
/// question's boundary, and that nested structure belongs to this field, not a new one.
fn collect_field<'a>(node: &'a AxNode, children_of: &impl Fn(&'a AxNodeId) -> Vec<&'a AxNode>, field: &mut PageField) {
    let mut label_parts: Vec<String> = Vec::new();
    collect_field_inner(node, children_of, field, &mut label_parts, true);
    field.label = label_parts.join(" ").trim().to_string();
}

fn collect_field_inner<'a>(
    node: &'a AxNode,
    children_of: &impl Fn(&'a AxNodeId) -> Vec<&'a AxNode>,
    field: &mut PageField,
    label_parts: &mut Vec<String>,
    is_field_root: bool,
) {
    if node.ignored {
        for child in children_of(&node.node_id) {
            collect_field_inner(child, children_of, field, label_parts, false);
        }
        return;
    }

    if is_role(node, HEADING_ROLES) || is_role(node, CONTEXT_TEXT_ROLES) {
        if let Some(text) = name_of(node) {
            label_parts.push(text);
        }
    }

    if is_role(node, IMAGE_ROLES) {
        // Included unconditionally, whatever the URL's shape — design-doc §7.1 bug #2's fix:
        // there is no scheme/prefix check here that could exclude a `data:` URI.
        if let Some(url) = url_property(node) {
            field.image_urls.push(url);
        }
    }

    if is_role(node, OPTION_ROLES) {
        if field.kind_hint.is_empty() {
            field.kind_hint = role_of(node).unwrap_or_default();
        }
        if let Some(text) = name_of(node) {
            field.options.push(text);
        }
        if is_checked(node) {
            let selected = name_of(node).unwrap_or_else(|| "selected".to_string());
            field.current_value = Some(match field.current_value.take() {
                Some(existing) => format!("{existing}, {selected}"),
                None => selected,
            });
        }
    }

    if is_role(node, TEXT_INPUT_ROLES) {
        if field.kind_hint.is_empty() {
            field.kind_hint = role_of(node).unwrap_or_default();
        }
        if let Some(text) = node.value.as_ref().and_then(|v| v.value.as_ref()).and_then(|v| v.as_str()) {
            let text = text.trim();
            if !text.is_empty() {
                field.current_value = Some(text.to_string());
            }
        }
    }

    if !is_field_root && is_role(node, ROW_ROLES) {
        if field.kind_hint.is_empty() {
            field.kind_hint = "grid".to_string();
        }
        if let Some(text) = name_of(node) {
            field.rows.push(text);
        }
    }

    for child in children_of(&node.node_id) {
        collect_field_inner(child, children_of, field, label_parts, false);
    }
}

pub fn all_fields_answered(content: &PageContent) -> bool {
    content.fields.iter().all(|f| f.current_value.as_deref().is_some_and(|v| !v.trim().is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One hand-built AX-tree-shaped JSON payload, structured exactly the way
    /// `Accessibility.getFullAXTree` documents its response (a flat `nodes` array linked via
    /// `parentId`/`childIds`), covering all three design-doc §7.1 bugs plus page-level context in
    /// a single page:
    ///   - node 2/3: page-level heading + paragraph, outside any question — page_context.
    ///   - node 10-14: a normal MCQ question, already answered ("Blue" checked).
    ///   - node 20/21: a question with an image and *no* heading at all — must still produce a
    ///     field (bug #1).
    ///   - node 30-32: a question with a heading *and* an inline `data:` URI image — the image
    ///     must survive unfiltered (bug #2).
    fn fixture_nodes() -> Vec<AxNode> {
        let json = r#"[
            { "nodeId": "1", "ignored": false, "childIds": ["2", "3", "10", "20", "30"] },
            { "nodeId": "2", "ignored": false, "parentId": "1",
              "role": { "type": "role", "value": "heading" },
              "name": { "type": "computedString", "value": "Midterm Exam" } },
            { "nodeId": "3", "ignored": false, "parentId": "1",
              "role": { "type": "role", "value": "paragraph" },
              "name": { "type": "computedString", "value": "Answer every question below." } },

            { "nodeId": "10", "ignored": false, "parentId": "1", "childIds": ["11", "12", "13"],
              "role": { "type": "role", "value": "listitem" } },
            { "nodeId": "11", "ignored": false, "parentId": "10",
              "role": { "type": "role", "value": "heading" },
              "name": { "type": "computedString", "value": "Favorite color?" } },
            { "nodeId": "12", "ignored": false, "parentId": "10",
              "role": { "type": "role", "value": "radio" },
              "name": { "type": "computedString", "value": "Red" },
              "properties": [ { "name": "checked", "value": { "type": "boolean", "value": false } } ] },
            { "nodeId": "13", "ignored": false, "parentId": "10",
              "role": { "type": "role", "value": "radio" },
              "name": { "type": "computedString", "value": "Blue" },
              "properties": [ { "name": "checked", "value": { "type": "boolean", "value": true } } ] },

            { "nodeId": "20", "ignored": false, "parentId": "1", "childIds": ["21"],
              "role": { "type": "role", "value": "listitem" } },
            { "nodeId": "21", "ignored": false, "parentId": "20",
              "role": { "type": "role", "value": "image" },
              "properties": [ { "name": "url", "value": { "type": "string", "value": "https://example.com/diagram.png" } } ] },

            { "nodeId": "30", "ignored": false, "parentId": "1", "childIds": ["31", "32"],
              "role": { "type": "role", "value": "listitem" } },
            { "nodeId": "31", "ignored": false, "parentId": "30",
              "role": { "type": "role", "value": "heading" },
              "name": { "type": "computedString", "value": "Identify the chart below" } },
            { "nodeId": "32", "ignored": false, "parentId": "30",
              "role": { "type": "role", "value": "image" },
              "properties": [ { "name": "url", "value": { "type": "string", "value": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUg==" } } ] }
        ]"#;
        serde_json::from_str(json).expect("fixture AX-tree JSON must match AxNode's real wire shape")
    }

    #[test]
    fn page_level_context_is_captured() {
        let content = build_page_content(&fixture_nodes());
        assert!(content.page_context.contains("Midterm Exam"), "{}", content.page_context);
        assert!(content.page_context.contains("Answer every question below."), "{}", content.page_context);
    }

    #[test]
    fn normal_question_extracts_label_options_and_answer() {
        let content = build_page_content(&fixture_nodes());
        let q1 = content.fields.iter().find(|f| f.id == "10").expect("question 1 must be a field");
        assert!(q1.label.contains("Favorite color?"));
        assert_eq!(q1.kind_hint, "radio");
        assert_eq!(q1.options, vec!["Red".to_string(), "Blue".to_string()]);
        assert_eq!(q1.current_value.as_deref(), Some("Blue"));
    }

    #[test]
    fn image_only_question_with_no_heading_still_produces_a_field() {
        // Design doc §7.1 bug #1: v2's `EXTRACTOR_JS` did `if (!text) return;` and silently
        // dropped exactly this case. Here the field must exist regardless.
        let content = build_page_content(&fixture_nodes());
        let q2 = content.fields.iter().find(|f| f.id == "20").expect("image-only question must still produce a PageField");
        assert_eq!(q2.label, "");
        assert_eq!(q2.image_urls, vec!["https://example.com/diagram.png".to_string()]);
    }

    #[test]
    fn inline_data_uri_image_is_not_filtered_out() {
        // Design doc §7.1 bug #2: v2 explicitly filtered `data:` URIs out during image
        // collection. Here it must survive verbatim.
        let content = build_page_content(&fixture_nodes());
        let q3 = content.fields.iter().find(|f| f.id == "30").expect("question 3 must be a field");
        assert!(q3.label.contains("Identify the chart below"));
        assert_eq!(q3.image_urls, vec!["data:image/png;base64,iVBORw0KGgoAAAANSUhEUg==".to_string()]);
    }

    #[test]
    fn all_fields_answered_is_false_while_any_field_is_blank() {
        let content = build_page_content(&fixture_nodes());
        // q1 has an answer, q2/q3 don't — the page as a whole must not read as fully answered.
        assert!(!all_fields_answered(&content));
    }

    #[test]
    fn all_fields_answered_is_true_once_every_field_has_a_value() {
        let mut content = build_page_content(&fixture_nodes());
        for f in &mut content.fields {
            f.current_value = Some("anything".to_string());
        }
        assert!(all_fields_answered(&content));
    }

    #[test]
    fn empty_node_list_yields_empty_page_content() {
        let content = build_page_content(&[]);
        assert_eq!(content.page_context, "");
        assert!(content.fields.is_empty());
    }
}
