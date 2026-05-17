// Kill chrome.exe / msedge.exe processes spawned by this daemon.
//
// NOTE: Toolhelp doesn't expose command-line directly, and matching by parent
// PID requires the daemon to have stored its children — which it doesn't,
// because we spawn detached. For v2 MVP this is a soft no-op: Chrome's debug
// session terminates when its WS connection closes (which happens when we
// exit). The temp profile dir is removed by self_delete's rmdir /S.
//
// A future hardening pass can use WMI (CIM_Process) to match by command line.

pub fn kill_spawned_browsers(_marker: &str) -> anyhow::Result<()> {
    Ok(())
}
