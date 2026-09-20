// Self-restart: spawn a hidden PowerShell that relaunches the exe after a brief delay, then exit
// cleanly.
//
// Real bug found by actually using this (not caught by any test): the relaunch passed no CLI
// args at all, so `--debug` (and anything else the user launched with) silently vanished on
// restart — the new instance came back up with no console/logging attached, which looks
// indistinguishable from "it just died" if you were watching the terminal that started the
// original one. Fixed: forward the current process's actual args to the relaunch.
//
// Second real bug, found the same way: `creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)`
// made `spawn()` report success (a real PID came back) but the PowerShell host never actually
// ran the `-Command` script — no relaunch, no error, nothing. Confirmed in isolation with a
// throwaway example binary: `CREATE_NO_WINDOW` alone relaunches fine; adding `DETACHED_PROCESS`
// on top breaks it every time. `CREATE_NO_WINDOW` is sufficient on its own to keep the relauncher
// invisible, so `DETACHED_PROCESS` is dropped entirely rather than worked around.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn execute() -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    let exe_str = normalize_path(&exe);

    // Preserve whatever this process was actually launched with (`--debug`, `--config`, etc.).
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg_list_clause = powershell_argument_list_clause(&args);

    let ps = format!(
        "Start-Sleep -Milliseconds 800; Start-Process -FilePath '{}'{}",
        exe_str.replace('\'', "''"),
        arg_list_clause
    );

    Command::new("powershell.exe")
        .args(["-WindowStyle", "Hidden", "-NoProfile", "-NonInteractive", "-Command", &ps])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;

    std::process::exit(0);
}

fn normalize_path(p: &Path) -> String {
    let s = p.to_string_lossy().into_owned();
    s.strip_prefix(r"\\?\").map(String::from).unwrap_or(s)
}

/// PowerShell `-ArgumentList` clause forwarding `args` to the relaunched exe, or an empty string
/// if there are none. Each arg is quoted separately (not joined into one string) so an arg
/// containing spaces (e.g. a `--config` path) survives intact; embedded single quotes are
/// doubled, PowerShell's own escaping convention.
fn powershell_argument_list_clause(args: &[String]) -> String {
    if args.is_empty() {
        return String::new();
    }
    let quoted: Vec<String> = args.iter().map(|a| format!("'{}'", a.replace('\'', "''"))).collect();
    format!(" -ArgumentList {}", quoted.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    // `execute()` itself can't be unit-tested (it calls `std::process::exit`), but the
    // arg-quoting logic that was actually wrong — the real bug found by using this — is pure
    // and testable in isolation, and this tests the real function, not a re-derived copy of it.

    #[test]
    fn empty_args_produce_no_clause() {
        assert_eq!(powershell_argument_list_clause(&[]), "");
    }

    #[test]
    fn quotes_each_arg_separately_so_spaces_survive() {
        assert_eq!(powershell_argument_list_clause(&["--debug".to_string()]), " -ArgumentList '--debug'");
        assert_eq!(
            powershell_argument_list_clause(&["--config".to_string(), "C:\\a b\\c.toml".to_string()]),
            " -ArgumentList '--config','C:\\a b\\c.toml'"
        );
    }

    #[test]
    fn escapes_embedded_single_quotes() {
        assert_eq!(powershell_argument_list_clause(&["it's".to_string()]), " -ArgumentList 'it''s'");
    }
}
