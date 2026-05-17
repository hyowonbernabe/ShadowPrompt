// Process lifecycle: panic, self-restart, self-delete, chrome cleanup.

pub mod chrome_cleanup;
pub mod panic;
pub mod path_cleanup;
pub mod self_delete;
pub mod self_restart;
pub mod startup;
