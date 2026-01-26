# Development Setup Guide

This guide explains how to set up ShadowPrompt for development, ensuring your credentials remain secure.

## 1. Configuration Setup

The project uses a local configuration file that is **ignored by git** to prevent accidental secret leaks.

1.  **Navigate to the config directory:**
    `shadow_prompt/config/`

2.  **Create your local config:**
    Copy the example template to create your active config file.
    
    ```powershell
    cp config.example.toml config.toml
    ```

    > **Note:** `config.toml` is listed in `.gitignore`. **NEVER** force add this file if it contains personal keys.

## 2. Google Authentication (Required Setup)

To enable "Sign in with Google" for your users, you (the developer) must create a Client ID **once**. Your users will **not** need to do this; they will simply log in.

### Step 1: Create the Client ID
1.  Go to the [Google Cloud Console](https://console.cloud.google.com/apis/credentials).
2.  Create a new Project (e.g., "ShadowPrompt").
3.  Navigate to **APIs & Services > OAuth consent screen**.
    - Users: **External** (allows any Google account to sign in).
    - Fill in app name ("ShadowPrompt") and support email.
    - **Scopes**: Add `.../auth/cloud-platform`.
    - **Test Users (CRITICAL)**: You must add the **exact email address** you intend to log in with. Result: "Error 403: access_denied" if you skip this.
4.  Navigate to **Credentials > Create Credentials > OAuth client ID**.
    - Application type: **Desktop app**.
    - Name: "ShadowPrompt Desktop".
5.  Copy the **Client ID** and **Client Secret**.

### Step 2: Configure Project
Update your local `shadow_prompt/config/config.toml`:

```toml
[auth.google]
enabled = true
client_id = "YOUR_NEW_CLIENT_ID"
client_secret = "YOUR_NEW_CLIENT_SECRET"
```

### Step 3: Distributing to Users
When you release the app:
- You will embed this **Client ID** (and usually the secret, for Desktop apps) into the compiled binary or default config.
- Users will just see a Google Login window with your App Name.
- **Note:** For broad release, you may need to "Publish" your OAuth consent screen so users don't see a "Unverified App" warning.

## 3. Running the App

```powershell
cd shadow_prompt
cargo run
```

On first use of a feature requiring auth, your browser will open to a Google Login page.
1.  Sign in with your Google Account.
2.  Grant the requested permissions.
3.  Close the window when prompted.
