use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenUrl, PkceCodeChallenge, TokenResponse,
    RefreshToken,
};
use anyhow::{Result, Context, anyhow};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;
use crate::auth::AuthData;

pub async fn perform_auth(client_id: Option<String>, client_secret: Option<String>, redirect_port: u16) -> Result<AuthData> {
    let client_id = client_id.unwrap_or_else(|| "299386765275-be243f65652614a80630727027156942.apps.googleusercontent.com".to_string());
    let client_secret = client_secret.unwrap_or_else(|| "notasecret".to_string());

    let redirect_url = format!("http://localhost:{}", redirect_port);
    
    let secret = if client_id == "299386765275-be243f65652614a80630727027156942.apps.googleusercontent.com" {
        None
    } else {
        Some(ClientSecret::new(client_secret))
    };

    // OAuth2 v4.4 Constructor
    let client = BasicClient::new(
        ClientId::new(client_id),
        secret,
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?)
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url.clone())?);

    // PKCE
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate URL
    let (auth_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
        .add_scope(Scope::new("https://www.googleapis.com/auth/cloud-platform".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    println!("[*] Opening Browser for Authentication...");
    if let Err(e) = open::that(auth_url.as_str()) {
        eprintln!("[!] Failed to open browser: {}", e);
        println!("[*] Please visit: {}", auth_url);
    }

    // Start Local Server
    let listener = TcpListener::bind(format!("127.0.0.1:{}", redirect_port))?;
    println!("[*] Waiting for callback on port {}...", redirect_port);

    // Accept one connection
    if let Some(mut stream) = listener.incoming().flatten().next() {
        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        // Parse Code from "GET /?code=... HTTP/1.1"
        let code = request_line
            .split_whitespace()
            .nth(1)
            .and_then(|path| {
                let url = Url::parse(&format!("http://localhost{}", path)).ok()?;
                let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
                pairs.get("code").cloned()
            });

        let message = "HTTP/1.1 200 OK\r\n\r\nShadowPrompt: Authentication Successful! You can close this window.";
        let _ = stream.write_all(message.as_bytes());

        if let Some(auth_code) = code {
            println!("[*] Code received. Exchanging for token...");
            
            let token_result = client
                .exchange_code(oauth2::AuthorizationCode::new(auth_code))
                .set_pkce_verifier(pkce_verifier)
                .request_async(oauth2::reqwest::async_http_client)
                .await
                .context("Failed to exchange token")?;

            return Ok(AuthData {
                access_token: token_result.access_token().secret().clone(),
                refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
                expiry: token_result.expires_in().map(|d| { // Type inference works better in v4 hopefully, or reuse d type
                    use std::time::{SystemTime, UNIX_EPOCH};
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + d.as_secs()
                }),
            });
        }
    }

    Err(anyhow!("Authentication failed or timed out."))
}
