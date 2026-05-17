// Remove install directory from user PATH (HKCU\Environment\Path).
// Broadcasts WM_SETTINGCHANGE so other processes pick up the change.

use std::path::Path;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_READ, KEY_WRITE, REG_EXPAND_SZ, REG_VALUE_TYPE,
};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};

pub fn remove_from_user_path(install_dir: &Path) -> anyhow::Result<()> {
    let target = install_dir.to_string_lossy().to_lowercase();
    let mut hkey = HKEY::default();
    let env: Vec<u16> = "Environment\0".encode_utf16().collect();

    unsafe {
        RegOpenKeyExW(HKEY_CURRENT_USER, PCWSTR(env.as_ptr()), 0, KEY_READ | KEY_WRITE, &mut hkey).ok()?;

        let path_name: Vec<u16> = "Path\0".encode_utf16().collect();
        let mut size: u32 = 0;
        let mut kind: REG_VALUE_TYPE = REG_VALUE_TYPE(0);
        let _ = RegQueryValueExW(hkey, PCWSTR(path_name.as_ptr()), None, Some(&mut kind), None, Some(&mut size));
        let mut buf = vec![0u8; size as usize];
        if size > 0 {
            RegQueryValueExW(hkey, PCWSTR(path_name.as_ptr()), None, Some(&mut kind), Some(buf.as_mut_ptr()), Some(&mut size)).ok()?;
        }
        let wide = bytes_to_wide(&buf);
        let current = String::from_utf16_lossy(&wide);
        let trimmed = current.trim_end_matches('\0');

        let filtered: Vec<&str> = trimmed
            .split(';')
            .filter(|p| !p.trim().is_empty() && p.to_lowercase() != target)
            .collect();
        let new_value = filtered.join(";");

        let mut new_wide: Vec<u16> = new_value.encode_utf16().collect();
        new_wide.push(0);
        let bytes = wide_to_bytes(&new_wide);
        RegSetValueExW(hkey, PCWSTR(path_name.as_ptr()), 0, REG_EXPAND_SZ, Some(&bytes)).ok()?;

        RegCloseKey(hkey).ok()?;

        let env_str: Vec<u16> = "Environment\0".encode_utf16().collect();
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(env_str.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            1000,
            None,
        );
    }
    Ok(())
}

fn bytes_to_wide(b: &[u8]) -> Vec<u16> {
    b.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}

fn wide_to_bytes(w: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(w.len() * 2);
    for c in w {
        out.extend_from_slice(&c.to_le_bytes());
    }
    out
}
