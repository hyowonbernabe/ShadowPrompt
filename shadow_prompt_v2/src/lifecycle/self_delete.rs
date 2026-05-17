// Insta-delete: wipe clipboard, kill spawned Chrome procs, spawn detached cleanup
// script that strips PATH entry + removes install folder, then exit.

pub fn execute() -> anyhow::Result<()> {
    // TODO: see docs/architecture.md "Self-Delete and Self-Restart" section.
    anyhow::bail!("not yet implemented")
}
