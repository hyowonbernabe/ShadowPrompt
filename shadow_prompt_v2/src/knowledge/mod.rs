// Knowledge layer: load *.md from <install_dir>/knowledge/_default and
// /knowledge/<subject>/ into a single deterministic concatenated string,
// then inject as a cached system block on every LLM call.

pub mod loader;

pub use loader::{load, KnowledgeBundle};
