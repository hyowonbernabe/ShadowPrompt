// SEB patching: detect installed version, copy + overwrite with patched DLLs, register
// .seb file association in HKCU, set RunOnce auto-cleanup. All non-admin.

mod resources;

use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context};
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegDeleteValueW, RegOpenKeyExW,
    RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn seb_install_dir() -> PathBuf {
    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
    PathBuf::from(pf).join("SafeExamBrowser").join("Application")
}

fn seb_local_dir() -> PathBuf {
    PathBuf::from(local_appdata()).join("SEBPatched").join("Application")
}

fn seb_local_base() -> PathBuf {
    PathBuf::from(local_appdata()).join("SEBPatched")
}

fn local_appdata() -> String {
    std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
        let up = std::env::var("USERPROFILE").unwrap_or_else(|_| r"C:\Users\Default".into());
        format!(r"{up}\AppData\Local")
    })
}

/// Detect the installed SEB version by reading the exe's file version.
pub fn detect_version() -> Option<String> {
    let exe = seb_install_dir().join("SafeExamBrowser.exe");
    if !exe.exists() { return None; }
    file_version(&exe)
}

fn file_version(path: &Path) -> Option<String> {
    let ps = format!(
        "(Get-Item '{}').VersionInfo.FileVersion",
        path.to_string_lossy().replace('\'', "''")
    );
    let out = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if v.is_empty() { None } else { Some(v) }
}

/// Whether SEB is currently patched (local patched copy exists).
pub fn is_patched() -> bool {
    seb_local_dir().join("SafeExamBrowser.exe").exists()
}

/// Patch SEB: copy install → local, overwrite with patched DLLs, register file association, set RunOnce.
pub fn patch() -> anyhow::Result<String> {
    let install = seb_install_dir();
    let seb_exe = install.join("SafeExamBrowser.exe");
    if !seb_exe.exists() {
        bail!("SEB not found at {}", install.display());
    }

    let version = file_version(&seb_exe).context("Could not read SEB version")?;

    let ver_data = resources::find_version(&version)
        .with_context(|| format!("SEB v{version} not supported"))?;

    let target = seb_local_dir();
    copy_dir(&install, &target).context("Failed to copy SEB to local folder")?;

    let is_64 = std::mem::size_of::<usize>() == 8;
    for pf in ver_data.files {
        let data = if is_64 { pf.x64 } else { pf.x86 };
        let dest = target.join(pf.name);
        if dest.exists() { fs::remove_file(&dest)?; }
        fs::write(&dest, data)?;
    }

    register_seb_assoc().context("Patched files OK but failed to register .seb file association")?;
    let _ = set_runonce(); // best-effort safety net — not critical

    Ok(format!("Patched SEB v{version}"))
}

/// Reset: delete local copy, remove file association, remove RunOnce.
pub fn reset() -> anyhow::Result<String> {
    let base = seb_local_base();
    if base.exists() {
        fs::remove_dir_all(&base).context("Failed to delete patched copy (close SEB first)")?;
    }
    let _ = remove_seb_assoc();
    let _ = remove_runonce();
    Ok("SEB reset to original".into())
}

/// Silent best-effort cleanup for panic/shutdown.
pub fn cleanup() {
    let base = seb_local_base();
    if base.exists() {
        let _ = fs::remove_dir_all(&base);
    }
    let _ = remove_seb_assoc();
    let _ = remove_runonce();
}

fn copy_dir(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let e = entry?;
        let dest = dst.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &dest)?;
        } else {
            fs::copy(e.path(), &dest)?;
        }
    }
    Ok(())
}

// ---- registry (HKCU, no admin) ----------------------------------------------------------------

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide_bytes(s: &str) -> Vec<u8> {
    let w = wide(s);
    let mut out = Vec::with_capacity(w.len() * 2);
    for c in &w { out.extend_from_slice(&c.to_le_bytes()); }
    out
}

fn reg_set(subkey: &str, name: &str, value: &str) -> anyhow::Result<()> {
    let sk = wide(subkey);
    let nm = wide(name);
    let vb = wide_bytes(value);
    unsafe {
        let mut hk = HKEY::default();
        RegCreateKeyExW(
            HKEY_CURRENT_USER, PCWSTR(sk.as_ptr()), 0, PCWSTR::null(),
            REG_OPTION_NON_VOLATILE, KEY_WRITE, None, &mut hk, None,
        ).ok()?;
        let r = RegSetValueExW(hk, PCWSTR(nm.as_ptr()), 0, REG_SZ, Some(&vb));
        let _ = RegCloseKey(hk);
        r.ok()?;
    }
    Ok(())
}

fn reg_del_tree(subkey: &str) -> anyhow::Result<()> {
    let sk = wide(subkey);
    unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, PCWSTR(sk.as_ptr())).ok()? };
    Ok(())
}

fn reg_del_value(subkey: &str, name: &str) -> anyhow::Result<()> {
    let sk = wide(subkey);
    let nm = wide(name);
    unsafe {
        let mut hk = HKEY::default();
        RegOpenKeyExW(HKEY_CURRENT_USER, PCWSTR(sk.as_ptr()), 0, KEY_WRITE, &mut hk).ok()?;
        let r = RegDeleteValueW(hk, PCWSTR(nm.as_ptr()));
        let _ = RegCloseKey(hk);
        r.ok()?;
    }
    Ok(())
}

fn register_seb_assoc() -> anyhow::Result<()> {
    let exe = seb_local_dir().join("SafeExamBrowser.exe");
    let cmd = format!("\"{}\" \"%1\"", exe.to_string_lossy());
    reg_set(r"Software\Classes\.seb", "", "SEBPatched.ConfigFile")?;
    reg_set(r"Software\Classes\SEBPatched.ConfigFile\shell\open\command", "", &cmd)?;
    shell_notify();
    Ok(())
}

fn remove_seb_assoc() -> anyhow::Result<()> {
    let _ = reg_del_tree(r"Software\Classes\.seb");
    let _ = reg_del_tree(r"Software\Classes\SEBPatched.ConfigFile");
    shell_notify();
    Ok(())
}

fn set_runonce() -> anyhow::Result<()> {
    let base = seb_local_base();
    let cmd = format!(
        "cmd /c rmdir /s /q \"{}\" & reg delete HKCU\\Software\\Classes\\.seb /f \
         & reg delete HKCU\\Software\\Classes\\SEBPatched.ConfigFile /f",
        base.to_string_lossy()
    );
    reg_set(r"Software\Microsoft\Windows\CurrentVersion\RunOnce", "SEBPatchCleanup", &cmd)
}

fn remove_runonce() -> anyhow::Result<()> {
    reg_del_value(r"Software\Microsoft\Windows\CurrentVersion\RunOnce", "SEBPatchCleanup")
}

fn shell_notify() {
    #[link(name = "shell32")]
    extern "system" {
        fn SHChangeNotify(wEventId: i32, uFlags: u32, dwItem1: usize, dwItem2: usize);
    }
    // SHCNE_ASSOCCHANGED, SHCNF_IDLIST
    unsafe { SHChangeNotify(0x08000000, 0, 0, 0); }
}
