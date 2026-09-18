// The `fill_page` tool — design doc §7.3: one batched tool call per page, robust label-based
// fuzzy matching internally (not v2's brittle exact-string matching), reports per-field
// success/failure back to the model in the same turn so a failed match can be retried
// immediately.
//
// Hard backstop, on top of the system prompt instruction (design doc §7.3, explicitly
// re-confirmed): re-checks the live page state at the moment of acting and skips any field that
// already has a value, regardless of what the model sent for it. Grid/matrix questions check
// per-row; every other question type (including multi-select checkboxes) checks whole-question.
//
// ## Element interaction approach — AX backendDOMNodeId -> DOM.resolveNode -> Runtime.callFunctionOn
//
// `read.rs`'s `PageField`/`PageContent` (built from the Accessibility domain) give us *which*
// question/option/row exists by label text, but nothing here can act on an accessibility node id
// directly — CDP's `Input`/`DOM` domains operate on DOM identifiers, not AX ones. The bridge
// chosen, and why:
//
// - `chromiumoxide::element::Element` (the crate's own high-level click()/type_str() wrapper) is
//   the obvious first thing to reach for, but `Element::new` is `pub(crate)`
//   (`chromiumoxide-0.9.1/src/element.rs:40`) — unconstructable from outside the crate except via
//   `Page::find_element`/`find_elements`/`find_xpath`, all CSS/XPath-selector-based. There is no
//   public constructor from a `NodeId`/`BackendNodeId`, so an AX-tree-driven lookup (which has
//   neither a reliable CSS selector nor an XPath, only an accessible name/role) cannot produce an
//   `Element` through any public API.
// - `AxNode.backend_dom_node_id: Option<BackendNodeId>` *is* present on every AX node
//   (`chromiumoxide_cdp-0.9.1/src/cdp.rs`, `AxNode` struct — the same file `read.rs` already
//   cites for `AxNode`/`AxPropertyName`), confirmed real by reading the generated struct, not
//   assumed. `DOM.resolveNode` (`ResolveNodeParams::builder().backend_node_id(id)`) turns that
//   into a `Runtime.RemoteObjectId` — this is exactly what `Element::new` itself does internally
//   (`DescribeNodeParams` -> `ResolveNodeParams`, `element.rs:40-71`), just reached here through
//   the public, generic `Page::execute::<T: Command>` surface instead of the private wrapper.
// - Once we have a `RemoteObjectId`, `Runtime.callFunctionOn` (`CallFunctionOnParams`) invokes a
//   JS function with `this` bound to that object — again mirroring what `Element::call_js_fn`
//   does for `click()`/`type_str()`/`focus()` (`element.rs:188-323`), just without needing the
//   private `Element` type to get there.
// - The CDP `Input` domain (synthetic mouse/keyboard events at screen coordinates) was
//   considered and rejected for the *primary* path: it needs a bounding box/clickable point
//   (extra `DOM.getContentQuads`/`getBoxModel` round trips) and is more fragile for elements that
//   are scrolled out of view or inside a grid row — `Element::click()` itself only resorts to
//   `Input`-domain mouse events (`tab.click(point)`) after first computing a clickable point via
//   `scroll_into_view()` + `clickable_point()`. Calling `this.click()` (for controls) or setting
//   `.value`/`textContent` and dispatching `input`/`change` events (for text controls) via
//   `Runtime.callFunctionOn` gets the same effect in one round trip per action instead of three,
//   and is the same DOM-level mechanism a real click ultimately triggers.
//
// Genuine open uncertainty (no live Chrome in this environment — same constraint `read.rs`
// documents): whether Google Forms' custom radio/checkbox `div`s actually respond to a synthetic
// `.click()` call the same way they respond to a real pointer event, and whether `Next`/`Submit`
// controls (see `advance_to_next_page` in `mod.rs`) are really exposed with AX role `"button"`.
// Both are the milestone's own stated, unavoidable limit — flagged here rather than glossed over.
//
// `PageField`/`PageContent` don't carry per-control node identity or (for grids) per-row current
// values — by design, `read.rs` aggregates a grid's checked state into one whole-field
// `current_value` string, which is exactly right for M6's page-level "anything blank?" check and
// not granular enough for this milestone's row-level backstop. So this file re-walks a *fresh*
// Accessibility-tree snapshot of its own for element resolution and grid row state, independently
// of `read.rs`'s private walking helpers (which aren't visible outside that file) — see `AxIndex`
// and the `collect_*`/`best_match` helpers below. The whole-field backstop, however, reuses
// `read::read_page`/`PageField` directly, so it stays byte-for-byte consistent with the exact
// values M6's `all_fields_answered` walk-forward loop already trusts.

use std::collections::HashMap;

use async_trait::async_trait;
use chromiumoxide::cdp::browser_protocol::accessibility::{
    AxNode, AxPropertyName, EnableParams, GetFullAxTreeParams,
};
use chromiumoxide::cdp::browser_protocol::dom::ResolveNodeParams;
use chromiumoxide::cdp::js_protocol::runtime::{CallArgument, CallFunctionOnParams, RemoteObjectId};
use chromiumoxide::Page;
use serde_json::json;

use super::read::{self, PageField};
use crate::llm::messages::ToolDef;
use crate::llm::Tool;

pub struct FillPageTool {
    pub page: Page,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FieldResult {
    pub id: String,
    pub ok: bool,
    pub detail: String,
}

impl FieldResult {
    fn ok(id: &str, detail: impl Into<String>) -> Self {
        FieldResult { id: id.to_string(), ok: true, detail: detail.into() }
    }

    fn fail(id: &str, detail: impl Into<String>) -> Self {
        FieldResult { id: id.to_string(), ok: false, detail: detail.into() }
    }

    /// Same shape as `ok` — a skip is the *correct* outcome of the already-answered backstop,
    /// not a failure, so the model sees it as a clean, informative no-op rather than an error.
    fn skip(id: &str, detail: impl Into<String>) -> Self {
        FieldResult { id: id.to_string(), ok: true, detail: detail.into() }
    }
}

const OPTION_ROLES: &[&str] = &["radio", "checkbox", "menuitemradio", "menuitemcheckbox"];
const TEXT_INPUT_ROLES: &[&str] = &["textbox", "combobox", "searchbox"];
const ROW_ROLES: &[&str] = &["row"];

// --- Minimal local AX-tree indexing/role helpers -----------------------------------------------
// Deliberately reimplemented here rather than imported from `read.rs`: `read.rs`'s equivalents
// (`role_of`, `is_role`, `name_of`, `is_checked`, its `by_id`/`children_of` closures) are private
// to that file, and `read.rs` is out of scope for this milestone to modify. Same approach,
// verified against the same real `AxNode`/`AxValue`/`AxProperty` types `read.rs` already cites.

struct AxIndex<'a> {
    by_id: HashMap<&'a str, &'a AxNode>,
}

impl<'a> AxIndex<'a> {
    fn build(nodes: &'a [AxNode]) -> Self {
        AxIndex { by_id: nodes.iter().map(|n| (n.node_id.inner().as_str(), n)).collect() }
    }

    fn get(&self, id: &str) -> Option<&'a AxNode> {
        self.by_id.get(id).copied()
    }

    fn children(&self, node: &'a AxNode) -> Vec<&'a AxNode> {
        node.child_ids
            .as_ref()
            .map(|ids| ids.iter().filter_map(|cid| self.by_id.get(cid.inner().as_str()).copied()).collect())
            .unwrap_or_default()
    }
}

fn role_of(node: &AxNode) -> Option<String> {
    node.role.as_ref()?.value.as_ref()?.as_str().map(|s| s.to_ascii_lowercase())
}

/// `pub(super)` because `mod.rs`'s `advance_to_next_page` reuses this exact role check (for
/// finding a "button"-role Next/Submit control) rather than re-deriving it a third time.
pub(super) fn is_role(node: &AxNode, candidates: &[&str]) -> bool {
    role_of(node).is_some_and(|r| candidates.contains(&r.as_str()))
}

/// `pub(super)` for the same reason as `is_role` above — `advance_to_next_page` needs a
/// control's accessible name to tell "Next" apart from "Submit".
pub(super) fn name_of(node: &AxNode) -> Option<String> {
    node.name
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

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

/// Collects every non-ignored descendant option/checkbox-like control under `node` (radios,
/// checkboxes, and their `menuitem*` variants). Used both for a plain field's own options and,
/// scoped to one row node, for a grid row's columns.
fn collect_option_controls<'a>(index: &AxIndex<'a>, node: &'a AxNode, out: &mut Vec<&'a AxNode>) {
    if node.ignored {
        for child in index.children(node) {
            collect_option_controls(index, child, out);
        }
        return;
    }
    if is_role(node, OPTION_ROLES) {
        out.push(node);
    }
    for child in index.children(node) {
        collect_option_controls(index, child, out);
    }
}

/// Collects the row-role nodes directly under a grid field. Rows aren't nested inside one
/// another, so descent stops the moment one is found — its own children (the row's option
/// controls) are gathered separately, by `collect_option_controls` scoped to that row.
fn collect_row_nodes<'a>(index: &AxIndex<'a>, node: &'a AxNode, out: &mut Vec<&'a AxNode>) {
    if node.ignored {
        for child in index.children(node) {
            collect_row_nodes(index, child, out);
        }
        return;
    }
    if is_role(node, ROW_ROLES) {
        out.push(node);
        return;
    }
    for child in index.children(node) {
        collect_row_nodes(index, child, out);
    }
}

/// Finds the first text-entry control (`textbox`/`combobox`/`searchbox`) under `node`.
fn find_text_input<'a>(index: &AxIndex<'a>, node: &'a AxNode) -> Option<&'a AxNode> {
    if node.ignored {
        return index.children(node).into_iter().find_map(|c| find_text_input(index, c));
    }
    if is_role(node, TEXT_INPUT_ROLES) {
        return Some(node);
    }
    index.children(node).into_iter().find_map(|c| find_text_input(index, c))
}

fn normalize(s: &str) -> String {
    s.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Fuzzy, not exact-string, label matching (design doc §7.3 — the specific fix over v2's brittle
/// exact matching): case/whitespace-insensitive equality scores highest, then substring
/// containment either direction. Pure and unit-tested directly against hand-built AX nodes.
fn best_match<'a>(candidates: &[&'a AxNode], target: &str) -> Option<&'a AxNode> {
    let target_n = normalize(target);
    if target_n.is_empty() {
        return None;
    }
    let mut best: Option<(&'a AxNode, u8)> = None;
    for &node in candidates {
        let Some(name) = name_of(node) else { continue };
        let name_n = normalize(&name);
        if name_n.is_empty() {
            continue;
        }
        let score = if name_n == target_n {
            2
        } else if name_n.contains(&target_n) || target_n.contains(&name_n) {
            1
        } else {
            0
        };
        if score > 0 && best.is_none_or(|(_, s)| score > s) {
            best = Some((node, score));
        }
    }
    best.map(|(n, _)| n)
}

fn split_csv(s: &str) -> Vec<String> {
    s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
}

/// Whole-field backstop (design doc §7.3): non-grid kinds — including multi-select checkboxes —
/// are checked as one unit. Any non-empty live value means "leave it alone," full stop.
fn whole_field_already_answered(current_value: &Option<String>) -> bool {
    current_value.as_deref().is_some_and(|v| !v.trim().is_empty())
}

/// Row-level backstop half for grids: true if *any* option within this specific row is already
/// checked, independent of every other row's state.
fn row_options_checked(index: &AxIndex<'_>, row: &AxNode) -> bool {
    let mut opts = Vec::new();
    collect_option_controls(index, row, &mut opts);
    opts.iter().any(|n| is_checked(n))
}

// --- CDP bridge: AX backendDOMNodeId -> DOM.resolveNode -> Runtime.callFunctionOn ---------------

async fn resolve_object_id(page: &Page, node: &AxNode) -> anyhow::Result<RemoteObjectId> {
    let backend_id = node.backend_dom_node_id.ok_or_else(|| {
        anyhow::anyhow!(
            "accessibility node {:?} has no backend DOM node id (purely accessibility-virtual or detached)",
            node.node_id
        )
    })?;
    let resp = page
        .execute(ResolveNodeParams::builder().backend_node_id(backend_id).build())
        .await
        .map_err(|e| anyhow::anyhow!("DOM.resolveNode failed: {e}"))?;
    resp.result
        .object
        .object_id
        .ok_or_else(|| anyhow::anyhow!("resolved node has no remote JS object id (detached from the document?)"))
}

/// Clicks the DOM element behind `node` — used for radio/checkbox options and (in `mod.rs`) the
/// Next/Submit-detection step of `advance_to_next_page`. Exposed at `pub(super)` so both this
/// module and `mod.rs`'s `advance_to_next_page` share one implementation rather than duplicating
/// the resolve-and-click bridge.
pub(super) async fn click_node(page: &Page, node: &AxNode) -> anyhow::Result<()> {
    let object_id = resolve_object_id(page, node).await?;
    let mut params = CallFunctionOnParams::new(
        "function() { this.scrollIntoView({block: 'center', inline: 'center'}); this.click(); return true; }",
    );
    params.object_id = Some(object_id);
    params.return_by_value = Some(true);
    params.user_gesture = Some(true);
    page.execute(params)
        .await
        .map_err(|e| anyhow::anyhow!("Runtime.callFunctionOn (click) failed: {e}"))?;
    Ok(())
}

async fn type_into_node(page: &Page, node: &AxNode, text: &str) -> anyhow::Result<()> {
    let object_id = resolve_object_id(page, node).await?;
    let mut params = CallFunctionOnParams::new(
        "function(v) {
            this.scrollIntoView({block: 'center', inline: 'center'});
            this.focus();
            if (this.tagName === 'INPUT' || this.tagName === 'TEXTAREA') { this.value = v; }
            else { this.textContent = v; }
            this.dispatchEvent(new Event('input', { bubbles: true }));
            this.dispatchEvent(new Event('change', { bubbles: true }));
            this.blur();
            return true;
        }",
    );
    params.object_id = Some(object_id);
    params.arguments = Some(vec![CallArgument {
        value: Some(serde_json::Value::String(text.to_string())),
        unserializable_value: None,
        object_id: None,
    }]);
    params.return_by_value = Some(true);
    params.user_gesture = Some(true);
    page.execute(params)
        .await
        .map_err(|e| anyhow::anyhow!("Runtime.callFunctionOn (type) failed: {e}"))?;
    Ok(())
}

/// Fresh `Accessibility.getFullAXTree` snapshot, independent of `read::read_page`'s own fetch —
/// see the module doc for why this file needs its own (node identity for interaction, per-row
/// grid state). Exposed at `pub(super)` so `mod.rs`'s `advance_to_next_page` reuses it instead of
/// issuing a third, differently-shaped fetch of the same tree.
pub(super) async fn fetch_ax_snapshot(page: &Page) -> anyhow::Result<Vec<AxNode>> {
    page.execute(EnableParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("Accessibility.enable failed: {e}"))?;
    let resp = page
        .execute(GetFullAxTreeParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("Accessibility.getFullAXTree failed: {e}"))?;
    Ok(resp.result.nodes)
}

// --- Per-kind field filling ----------------------------------------------------------------

async fn fill_single_choice(page: &Page, index: &AxIndex<'_>, root: &AxNode, field_id: &str, answer: &str) -> FieldResult {
    let target = answer.trim();
    if target.is_empty() {
        return FieldResult::fail(field_id, "empty answer given for a single-choice question");
    }
    let mut candidates = Vec::new();
    collect_option_controls(index, root, &mut candidates);
    let Some(node) = best_match(&candidates, target) else {
        return FieldResult::fail(field_id, format!("no option on the page matched \"{target}\""));
    };
    if is_checked(node) {
        return FieldResult::skip(field_id, "matched option already selected, not touched");
    }
    match click_node(page, node).await {
        Ok(()) => FieldResult::ok(field_id, format!("clicked option \"{}\"", name_of(node).unwrap_or_default())),
        Err(e) => FieldResult::fail(field_id, format!("click failed: {e}")),
    }
}

async fn fill_multi_choice(page: &Page, index: &AxIndex<'_>, root: &AxNode, field_id: &str, answer: &str) -> FieldResult {
    let parts = split_csv(answer);
    if parts.is_empty() {
        return FieldResult::fail(field_id, "empty answer given for a multi-select question");
    }
    let mut candidates = Vec::new();
    collect_option_controls(index, root, &mut candidates);

    let mut clicked = Vec::new();
    let mut problems = Vec::new();
    for part in &parts {
        match best_match(&candidates, part) {
            Some(node) if is_checked(node) => problems.push(format!("\"{part}\" already selected, left alone")),
            Some(node) => match click_node(page, node).await {
                Ok(()) => clicked.push(part.clone()),
                Err(e) => problems.push(format!("\"{part}\": click failed: {e}")),
            },
            None => problems.push(format!("no option matched \"{part}\"")),
        }
    }
    if problems.is_empty() {
        FieldResult::ok(field_id, format!("clicked: {}", clicked.join(", ")))
    } else {
        FieldResult::fail(field_id, format!("clicked [{}]; problems: {}", clicked.join(", "), problems.join("; ")))
    }
}

async fn fill_text(page: &Page, index: &AxIndex<'_>, root: &AxNode, field_id: &str, answer: &str) -> FieldResult {
    let Some(node) = find_text_input(index, root) else {
        return FieldResult::fail(field_id, "no text input control found for this field");
    };
    match type_into_node(page, node, answer).await {
        Ok(()) => FieldResult::ok(field_id, format!("typed {} character(s)", answer.chars().count())),
        Err(e) => FieldResult::fail(field_id, format!("typing failed: {e}")),
    }
}

async fn fill_grid(page: &Page, index: &AxIndex<'_>, root: &AxNode, field_id: &str, raw_answer: &str) -> Vec<FieldResult> {
    let requested: HashMap<String, String> = match serde_json::from_str(raw_answer) {
        Ok(map) => map,
        Err(e) => {
            return vec![FieldResult::fail(
                field_id,
                format!("grid answer must be a JSON object mapping row label to column(s): {e}"),
            )];
        }
    };

    let mut rows = Vec::new();
    collect_row_nodes(index, root, &mut rows);

    let mut results = Vec::new();
    for (row_label, value) in &requested {
        let row_result_id = format!("{field_id}::{row_label}");

        let Some(row_node) = best_match(&rows, row_label) else {
            results.push(FieldResult::fail(&row_result_id, format!("no row on this grid matched \"{row_label}\"")));
            continue;
        };

        // Row-level backstop (design doc §7.3, explicitly re-confirmed): a partially-filled grid
        // is never skipped wholesale — only rows that are individually still blank get filled.
        if row_options_checked(index, row_node) {
            results.push(FieldResult::skip(&row_result_id, "already answered, not touched"));
            continue;
        }

        let parts = split_csv(value);
        if parts.is_empty() {
            results.push(FieldResult::fail(&row_result_id, "empty value given for this row"));
            continue;
        }

        let mut row_options = Vec::new();
        collect_option_controls(index, row_node, &mut row_options);

        let mut clicked = Vec::new();
        let mut problems = Vec::new();
        for part in &parts {
            match best_match(&row_options, part) {
                Some(node) if is_checked(node) => problems.push(format!("\"{part}\" already selected")),
                Some(node) => match click_node(page, node).await {
                    Ok(()) => clicked.push(part.clone()),
                    Err(e) => problems.push(format!("\"{part}\": click failed: {e}")),
                },
                None => problems.push(format!("no column matched \"{part}\"")),
            }
        }

        if problems.is_empty() {
            results.push(FieldResult::ok(&row_result_id, format!("clicked: {}", clicked.join(", "))));
        } else {
            results.push(FieldResult::fail(
                &row_result_id,
                format!("clicked [{}]; problems: {}", clicked.join(", "), problems.join("; ")),
            ));
        }
    }
    results
}

async fn fill_field(
    page: &Page,
    index: &AxIndex<'_>,
    meta: Option<&PageField>,
    field_id: &str,
    raw_answer: &str,
) -> Vec<FieldResult> {
    let Some(meta) = meta else {
        return vec![FieldResult::fail(field_id, "unknown field id — not present in the current live page read")];
    };
    let Some(root) = index.get(field_id) else {
        return vec![FieldResult::fail(field_id, "field id not found in the live accessibility tree snapshot")];
    };

    if meta.kind_hint.eq_ignore_ascii_case("grid") {
        return fill_grid(page, index, root, field_id, raw_answer).await;
    }

    // Whole-field backstop (design doc §7.3): re-checked here, against the live value fetched
    // moments ago in `fill_page`, never against what the caller's tool-call arguments claim.
    if whole_field_already_answered(&meta.current_value) {
        return vec![FieldResult::skip(field_id, "already answered, not touched")];
    }

    let result = match meta.kind_hint.as_str() {
        "radio" | "menuitemradio" => fill_single_choice(page, index, root, field_id, raw_answer).await,
        "checkbox" | "menuitemcheckbox" => fill_multi_choice(page, index, root, field_id, raw_answer).await,
        "textbox" | "combobox" | "searchbox" => fill_text(page, index, root, field_id, raw_answer).await,
        other => FieldResult::fail(field_id, format!("unrecognized field kind \"{other}\" — cannot fill safely")),
    };
    vec![result]
}

pub async fn fill_page(page: &Page, answers: &HashMap<String, String>) -> anyhow::Result<Vec<FieldResult>> {
    // Live re-check, right before acting on anything (design doc §7.3's non-negotiable): two
    // fresh reads, both fetched for this call, never anything cached from an earlier turn.
    //   - `read::read_page` — the exact same whole-field `current_value` M6's walk-forward loop
    //     already trusts, so the non-grid backstop below stays byte-for-byte consistent with what
    //     decided this page was worth showing to the model at all.
    //   - a raw accessibility-tree snapshot (`fetch_ax_snapshot`), walked locally, for the node
    //     identity `PageField` doesn't carry and for grids' per-row checked state.
    let content = read::read_page(page)
        .await
        .map_err(|e| anyhow::anyhow!("re-reading live page state before filling: {e}"))?;
    let fields_by_id: HashMap<&str, &PageField> = content.fields.iter().map(|f| (f.id.as_str(), f)).collect();

    let nodes = fetch_ax_snapshot(page).await?;
    let index = AxIndex::build(&nodes);

    let mut results = Vec::new();
    for (field_id, raw_answer) in answers {
        let meta = fields_by_id.get(field_id.as_str()).copied();
        results.extend(fill_field(page, &index, meta, field_id, raw_answer).await);
    }
    Ok(results)
}

#[async_trait]
impl Tool for FillPageTool {
    fn name(&self) -> &'static str {
        "fill_page"
    }

    fn definition(&self) -> ToolDef {
        ToolDef {
            kind: "function",
            function: crate::llm::messages::ToolFunctionDef {
                name: self.name().to_string(),
                description: "Submit answers for the current form page. Keys are question ids, values are shaped per the delivery rules in the system prompt.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "answers": {
                            "type": "object",
                            "additionalProperties": { "type": "string" }
                        }
                    },
                    "required": ["answers"]
                }),
            },
        }
    }

    async fn execute(&self, args_json: &str) -> anyhow::Result<String> {
        #[derive(serde::Deserialize)]
        struct Args {
            answers: HashMap<String, String>,
        }
        let args: Args = serde_json::from_str(args_json)
            .map_err(|e| anyhow::anyhow!("invalid arguments: {e}"))?;
        let results = fill_page(&self.page, &args.answers).await?;
        Ok(serde_json::to_string(&results)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-built AX-tree-shaped JSON, same wire shape `read.rs`'s own fixtures use:
    ///   - node 10/11/12: a standalone radio question ("Yes"/"No", both unchecked) — for
    ///     `best_match` matching tests.
    ///   - node 20: a grid field with two rows — row 30 ("Row 1") already has "Always" checked;
    ///     row 40 ("Row 2") is entirely blank. Tests the row-level, not whole-field, backstop.
    ///   - node 50/51: a field whose only interactive descendant is a `textbox`.
    fn fixture_nodes() -> Vec<AxNode> {
        let json = r#"[
            { "nodeId": "1", "ignored": false, "childIds": ["10", "20", "50"] },

            { "nodeId": "10", "ignored": false, "parentId": "1", "childIds": ["11", "12"],
              "role": { "type": "role", "value": "listitem" }, "backendDOMNodeId": 100 },
            { "nodeId": "11", "ignored": false, "parentId": "10",
              "role": { "type": "role", "value": "radio" },
              "name": { "type": "computedString", "value": "Yes" },
              "backendDOMNodeId": 101,
              "properties": [ { "name": "checked", "value": { "type": "boolean", "value": false } } ] },
            { "nodeId": "12", "ignored": false, "parentId": "10",
              "role": { "type": "role", "value": "radio" },
              "name": { "type": "computedString", "value": "No" },
              "backendDOMNodeId": 102,
              "properties": [ { "name": "checked", "value": { "type": "boolean", "value": false } } ] },

            { "nodeId": "20", "ignored": false, "parentId": "1", "childIds": ["30", "40"],
              "role": { "type": "role", "value": "listitem" } },
            { "nodeId": "30", "ignored": false, "parentId": "20", "childIds": ["31", "32"],
              "role": { "type": "role", "value": "row" },
              "name": { "type": "computedString", "value": "Row 1" } },
            { "nodeId": "31", "ignored": false, "parentId": "30",
              "role": { "type": "role", "value": "radio" },
              "name": { "type": "computedString", "value": "Always" },
              "properties": [ { "name": "checked", "value": { "type": "boolean", "value": true } } ] },
            { "nodeId": "32", "ignored": false, "parentId": "30",
              "role": { "type": "role", "value": "radio" },
              "name": { "type": "computedString", "value": "Never" },
              "properties": [ { "name": "checked", "value": { "type": "boolean", "value": false } } ] },
            { "nodeId": "40", "ignored": false, "parentId": "20", "childIds": ["41", "42"],
              "role": { "type": "role", "value": "row" },
              "name": { "type": "computedString", "value": "Row 2" } },
            { "nodeId": "41", "ignored": false, "parentId": "40",
              "role": { "type": "role", "value": "radio" },
              "name": { "type": "computedString", "value": "Always" },
              "properties": [ { "name": "checked", "value": { "type": "boolean", "value": false } } ] },
            { "nodeId": "42", "ignored": false, "parentId": "40",
              "role": { "type": "role", "value": "radio" },
              "name": { "type": "computedString", "value": "Never" },
              "properties": [ { "name": "checked", "value": { "type": "boolean", "value": false } } ] },

            { "nodeId": "50", "ignored": false, "parentId": "1", "childIds": ["51"],
              "role": { "type": "role", "value": "listitem" } },
            { "nodeId": "51", "ignored": false, "parentId": "50",
              "role": { "type": "role", "value": "textbox" }, "backendDOMNodeId": 151 }
        ]"#;
        serde_json::from_str(json).expect("fixture AX-tree JSON must match AxNode's real wire shape")
    }

    #[test]
    fn normalize_collapses_case_and_whitespace() {
        assert_eq!(normalize("  Blue   Sky "), "blue sky");
    }

    #[test]
    fn split_csv_trims_and_drops_empties() {
        assert_eq!(split_csv("Apple, Cherry ,, Date"), vec!["Apple", "Cherry", "Date"]);
    }

    #[test]
    fn whole_field_already_answered_true_for_any_nonempty_value() {
        assert!(whole_field_already_answered(&Some("Apple".to_string())));
        assert!(!whole_field_already_answered(&Some("   ".to_string())));
        assert!(!whole_field_already_answered(&None));
    }

    #[test]
    fn checkbox_field_with_any_selection_is_treated_as_whole_field_answered() {
        // Design doc §7.3: multi-select checkbox stays whole-question skip, never per-option —
        // built from a hand-built `PageField` exactly as this milestone's tests are meant to.
        let field = PageField {
            id: "q1".into(),
            label: "Fruits?".into(),
            kind_hint: "checkbox".into(),
            options: vec!["Apple".into(), "Cherry".into()],
            rows: vec![],
            current_value: Some("Apple".into()),
            image_urls: vec![],
        };
        assert!(whole_field_already_answered(&field.current_value));
    }

    #[test]
    fn radio_field_with_no_current_value_proceeds() {
        let field = PageField {
            id: "q2".into(),
            label: "Favorite color?".into(),
            kind_hint: "radio".into(),
            options: vec!["Red".into(), "Blue".into()],
            rows: vec![],
            current_value: None,
            image_urls: vec![],
        };
        assert!(!whole_field_already_answered(&field.current_value));
    }

    #[test]
    fn best_match_prefers_exact_over_substring() {
        let nodes = fixture_nodes();
        let index = AxIndex::build(&nodes);
        let root = index.get("10").expect("field 10 must exist");
        let mut candidates = Vec::new();
        collect_option_controls(&index, root, &mut candidates);
        let hit = best_match(&candidates, "yes").expect("must match Yes");
        assert_eq!(name_of(hit), Some("Yes".to_string()));
    }

    #[test]
    fn best_match_returns_none_when_nothing_matches() {
        let nodes = fixture_nodes();
        let index = AxIndex::build(&nodes);
        let root = index.get("10").unwrap();
        let mut candidates = Vec::new();
        collect_option_controls(&index, root, &mut candidates);
        assert!(best_match(&candidates, "Maybe").is_none());
    }

    #[test]
    fn find_text_input_locates_nested_textbox() {
        let nodes = fixture_nodes();
        let index = AxIndex::build(&nodes);
        let root = index.get("50").unwrap();
        let found = find_text_input(&index, root).expect("must find the textbox");
        assert_eq!(found.node_id.inner(), "51");
    }

    #[test]
    fn grid_row_already_answered_is_detected_per_row_not_whole_field() {
        // Design doc §7.3's explicitly re-confirmed rule: a partially-filled grid is checked row
        // by row, not skipped wholesale just because one row already has an answer.
        let nodes = fixture_nodes();
        let index = AxIndex::build(&nodes);
        let grid_root = index.get("20").unwrap();
        let mut rows = Vec::new();
        collect_row_nodes(&index, grid_root, &mut rows);
        assert_eq!(rows.len(), 2);

        let row1 = best_match(&rows, "Row 1").expect("Row 1 must resolve");
        let row2 = best_match(&rows, "Row 2").expect("Row 2 must resolve");
        assert!(row_options_checked(&index, row1), "Row 1 was pre-answered in the fixture");
        assert!(!row_options_checked(&index, row2), "Row 2 was left blank in the fixture");
    }

    #[test]
    fn grid_answer_json_parses_into_row_to_column_map() {
        let raw = r#"{"Row 2": "Always"}"#;
        let parsed: HashMap<String, String> = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.get("Row 2"), Some(&"Always".to_string()));
    }

    #[test]
    fn grid_answer_rejects_non_json_payload() {
        let raw = "Always"; // a bare string, not the required {row: column} object
        assert!(serde_json::from_str::<HashMap<String, String>>(raw).is_err());
    }

    #[test]
    fn field_result_skip_and_ok_both_report_success() {
        let ok = FieldResult::ok("f1", "did it");
        let skip = FieldResult::skip("f1", "already answered, not touched");
        let fail = FieldResult::fail("f1", "nope");
        assert!(ok.ok);
        assert!(skip.ok);
        assert!(!fail.ok);
    }
}
