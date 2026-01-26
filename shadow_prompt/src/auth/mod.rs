use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

pub mod google;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expiry: Option<u64>, // Unix timestamp
}

pub struct AuthManager;

impl AuthManager {
    const AUTH_FILE: &'static str = "auth.json";

    pub fn load_token() -> Option<AuthData> {
        if !Path::new(Self::AUTH_FILE).exists() {
            return None;
        }

        let content = fs::read_to_string(Self::AUTH_FILE).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save_token(data: &AuthData) -> Result<()> {
        let content = serde_json::to_string_pretty(data)?;
        fs::write(Self::AUTH_FILE, content).context("Failed to write auth.json")
    }

    pub fn clear_token() -> Result<()> {
        if Path::new(Self::AUTH_FILE).exists() {
            fs::remove_file(Self::AUTH_FILE)?;
        }
        Ok(())
    }
}
