use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use dialoguer::{Input, Password};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};

use crate::{art, config, display};

// Re-export spinner so other commands can import from here if needed
#[allow(unused_imports)]
pub use crate::art::{pulse_spinner, spinner};

#[derive(Subcommand)]
pub enum AuthCmd {
    /// Log in to a AxiomDB server
    #[clap(visible_alias = "li")]
    Login {
        #[arg(short, long, help = "Server base URL")]
        url: Option<String>,
        #[arg(short, long, help = "Email address")]
        email: Option<String>,
        #[arg(short, long, help = "Password (prefer interactive)")]
        password: Option<String>,
    },
    /// Clear local session tokens
    #[clap(visible_alias = "lo")]
    Logout,
    /// Show current logged-in identity
    #[clap(visible_alias = "me")]
    Whoami,
    /// Set the server URL
    #[clap(visible_alias = "server")]
    Use { url: String },
}

#[derive(Deserialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    user: UserOut,
}

#[derive(Deserialize)]
struct UserOut {
    #[allow(dead_code)]
    id: String,
    #[serde(default)]
    email: Option<String>,
    role: String,
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct CliAuthStartRequest<'a> {
    redirect_uri: &'a str,
    state: &'a str,
    code_challenge: &'a str,
    code_challenge_method: &'a str,
}

#[derive(Deserialize)]
struct CliAuthStartResponse {
    authorization_url: String,
}

#[derive(Serialize)]
struct CliAuthExchangeRequest<'a> {
    code: &'a str,
    redirect_uri: &'a str,
    code_verifier: &'a str,
}

#[derive(Deserialize)]
struct CliAuthExchangeResponse {
    access_token: String,
    refresh_token: String,
    expires_at: Option<String>,
    client_id: String,
    user: UserOut,
}

pub async fn run(cmd: AuthCmd) -> Result<()> {
    match cmd {
        AuthCmd::Login {
            url,
            email,
            password,
        } => do_login(url, email, password).await,
        AuthCmd::Logout => do_logout(),
        AuthCmd::Whoami => do_whoami(),
        AuthCmd::Use { url } => {
            config::set_base_url(&url)?;
            display::ok(&format!("Server set to {url}"));
            Ok(())
        }
    }
}

// ── Called by both auth sub-command and root-level shortcuts ─────────────────

pub async fn do_login(
    url: Option<String>,
    email: Option<String>,
    password: Option<String>,
) -> Result<()> {
    // Show welcome art on interactive login
    art::print_banner();

    if let Some(u) = url {
        config::set_base_url(&u)?;
    }

    art::section("Authentication");

    if email.is_none() && password.is_none() {
        return do_browser_login().await;
    }

    let email: String = match email {
        Some(e) => e,
        None => Input::new()
            .with_prompt(format!("  {}", "Email".truecolor_str(255, 140, 0)))
            .interact_text()?,
    };

    let password: String = match password {
        Some(p) => p,
        None => Password::new()
            .with_prompt(format!("  {}", "Password".truecolor_str(255, 140, 0)))
            .interact()?,
    };

    let sp = art::spinner("Authenticating…");
    let res: LoginResponse = crate::api::post_no_auth(
        "/auth/login",
        &LoginRequest {
            email: &email,
            password: &password,
        },
    )
    .await?;
    sp.finish_and_clear();

    config::set_tokens(config::Tokens {
        access_token: res.access_token,
        refresh_token: res.refresh_token,
        email: email.clone(),
        role: res.user.role.clone(),
        expires_at: None,
        client_id: Some("legacy-password".to_string()),
    })?;

    println!();
    art::step_ok(&format!(
        "Logged in as {}  •  role: {}  •  server: {}",
        email.truecolor(255, 195, 60).bold(),
        res.user.role.truecolor(255, 140, 0),
        config::load().base_url.truecolor(150, 150, 150)
    ));
    art::divider();
    println!();
    Ok(())
}

async fn do_browser_login() -> Result<()> {
    let pkce = crate::auth::pkce::PkceState::new();
    let loopback = crate::auth::loopback::spawn(pkce.state.clone())
        .context("Could not start browser callback. Try again or paste the code manually.")?;
    let crate::auth::loopback::LoopbackHandle {
        redirect_uri,
        code_rx,
        shutdown_tx,
    } = loopback;

    let start: CliAuthStartResponse = crate::api::post_no_auth(
        "/auth/cli/start",
        &CliAuthStartRequest {
            redirect_uri: &redirect_uri,
            state: &pkce.state,
            code_challenge: &pkce.challenge,
            code_challenge_method: "S256",
        },
    )
    .await?;

    println!(
        "  {}  Complete sign-in in browser…",
        "◒".truecolor(255, 140, 0)
    );
    if let Err(error) = open::that(&start.authorization_url) {
        display::warn(&format!("Could not open browser automatically: {error}"));
        println!("  Open this URL:\n  {}", start.authorization_url);
    }

    let code = match timeout(Duration::from_secs(180), code_rx).await {
        Ok(Ok(code)) => code,
        _ => {
            display::warn("Browser callback did not complete. Paste the authorization code or full redirect URL.");
            let pasted: String = Input::new()
                .with_prompt("Authorization code or redirect URL")
                .interact_text()?;
            crate::auth::loopback::parse_manual_code(&pasted)?
        }
    };
    let _ = shutdown_tx.send(());

    let sp = art::spinner("Completing OAuth with AxiomDB…");
    let token: CliAuthExchangeResponse = crate::api::post_no_auth(
        "/auth/cli/exchange",
        &CliAuthExchangeRequest {
            code: &code,
            redirect_uri: &redirect_uri,
            code_verifier: &pkce.verifier,
        },
    )
    .await?;
    sp.finish_and_clear();

    let email = token
        .user
        .email
        .clone()
        .unwrap_or_else(|| "base-idp-user".to_string());
    config::set_tokens(config::Tokens {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        email: email.clone(),
        role: token.user.role.clone(),
        expires_at: token.expires_at,
        client_id: Some(token.client_id),
    })?;

    println!();
    art::step_ok(&format!(
        "AxiomDB OAuth complete  •  {}  •  role: {}",
        email.truecolor(255, 195, 60).bold(),
        token.user.role.truecolor(255, 140, 0)
    ));
    art::divider();
    println!();
    Ok(())
}

pub fn do_logout() -> Result<()> {
    config::clear_tokens()?;
    display::ok("Logged out. Local tokens cleared.");
    Ok(())
}

pub fn do_whoami() -> Result<()> {
    let cfg = config::load();
    match cfg.tokens {
        None => {
            display::err("Not logged in. Run: axm login");
            std::process::exit(1);
        }
        Some(t) => {
            art::section("Current session");
            display::kv(&[
                ("Email", t.email),
                ("Role", t.role),
                ("Server", cfg.base_url),
                (
                    "Client",
                    t.client_id.unwrap_or_else(|| "axiomdb-cli".to_string()),
                ),
            ]);
            println!();
        }
    }
    Ok(())
}

// ── Trait extension for truecolor in prompt strings ──────────────────────────

trait TrueColorStr {
    fn truecolor_str(&self, r: u8, g: u8, b: u8) -> String;
}

impl TrueColorStr for str {
    fn truecolor_str(&self, r: u8, g: u8, b: u8) -> String {
        use colored::Colorize;
        self.truecolor(r, g, b).to_string()
    }
}
