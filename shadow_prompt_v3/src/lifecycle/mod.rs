// Process lifecycle: panic, self-restart, self-delete, single-instance startup. No
// watchdog/dead-man's-switch module — design doc §11: explicitly deferred, not building for v3
// now. Do not add one without checking that decision first.

pub mod panic;
pub mod path_cleanup;
pub mod self_delete;
pub mod self_restart;
pub mod startup;
