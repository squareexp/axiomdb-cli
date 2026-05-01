use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use dialoguer::{Input, Password};
use serde::{Deserialize, Serialize};

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
    role: String,
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    email: &'a str,
    password: &'a str,
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
