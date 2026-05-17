// Parse hotkey strings like "ctrl+shift+v" into key combos.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
    pub key: String, // canonicalized key name
}

pub fn parse(_input: &str) -> anyhow::Result<KeyCombo> {
    // TODO: tokenize on '+', lowercase, classify modifier vs key.
    anyhow::bail!("not yet implemented")
}
