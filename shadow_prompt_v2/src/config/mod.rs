// Config: TOML schema, defaults, path resolution.
// See docs/architecture.md "State Management".

pub mod load;
pub mod paths;
pub mod schema;

pub use schema::Config;
