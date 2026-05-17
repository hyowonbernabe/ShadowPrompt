// Parse hotkey strings like "ctrl+shift+v" into KeyCombo.

use rdev::Key;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
    pub key: Key,
}

pub fn parse(input: &str) -> anyhow::Result<KeyCombo> {
    let mut ctrl = false;
    let mut shift = false;
    let mut alt = false;
    let mut win = false;
    let mut main: Option<Key> = None;

    for raw in input.split('+') {
        let tok = raw.trim().to_lowercase();
        match tok.as_str() {
            "ctrl" | "control" => ctrl = true,
            "shift" => shift = true,
            "alt" => alt = true,
            "win" | "meta" | "super" => win = true,
            other => {
                if main.is_some() {
                    anyhow::bail!("hotkey '{input}' has more than one non-modifier key");
                }
                main = Some(key_from_str(other).ok_or_else(|| {
                    anyhow::anyhow!("hotkey '{input}': unknown key '{other}'")
                })?);
            }
        }
    }

    let key = main.ok_or_else(|| anyhow::anyhow!("hotkey '{input}' has no main key"))?;
    Ok(KeyCombo { ctrl, shift, alt, win, key })
}

fn key_from_str(s: &str) -> Option<Key> {
    Some(match s {
        "a" => Key::KeyA, "b" => Key::KeyB, "c" => Key::KeyC, "d" => Key::KeyD,
        "e" => Key::KeyE, "f" => Key::KeyF, "g" => Key::KeyG, "h" => Key::KeyH,
        "i" => Key::KeyI, "j" => Key::KeyJ, "k" => Key::KeyK, "l" => Key::KeyL,
        "m" => Key::KeyM, "n" => Key::KeyN, "o" => Key::KeyO, "p" => Key::KeyP,
        "q" => Key::KeyQ, "r" => Key::KeyR, "s" => Key::KeyS, "t" => Key::KeyT,
        "u" => Key::KeyU, "v" => Key::KeyV, "w" => Key::KeyW, "x" => Key::KeyX,
        "y" => Key::KeyY, "z" => Key::KeyZ,
        "0" => Key::Num0, "1" => Key::Num1, "2" => Key::Num2, "3" => Key::Num3,
        "4" => Key::Num4, "5" => Key::Num5, "6" => Key::Num6, "7" => Key::Num7,
        "8" => Key::Num8, "9" => Key::Num9,
        "space" => Key::Space,
        "enter" | "return" => Key::Return,
        "escape" | "esc" => Key::Escape,
        "tab" => Key::Tab,
        "backspace" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "insert" | "ins" => Key::Insert,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" | "pgup" => Key::PageUp,
        "pagedown" | "pgdn" => Key::PageDown,
        "up" => Key::UpArrow,
        "down" => Key::DownArrow,
        "left" => Key::LeftArrow,
        "right" => Key::RightArrow,
        "f1" => Key::F1, "f2" => Key::F2, "f3" => Key::F3, "f4" => Key::F4,
        "f5" => Key::F5, "f6" => Key::F6, "f7" => Key::F7, "f8" => Key::F8,
        "f9" => Key::F9, "f10" => Key::F10, "f11" => Key::F11, "f12" => Key::F12,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modifiers() {
        let c = parse("ctrl+shift+v").unwrap();
        assert!(c.ctrl && c.shift && !c.alt && !c.win);
        assert_eq!(c.key, Key::KeyV);
    }

    #[test]
    fn rejects_unknown_key() {
        assert!(parse("ctrl+shift+nope").is_err());
    }

    #[test]
    fn rejects_no_main_key() {
        assert!(parse("ctrl+shift").is_err());
    }
}
