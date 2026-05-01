mod api;
mod art;
mod commands;
mod config;
mod credentials;
mod display;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::ffi::OsString;

use commands::{
    audit::{AuditCmd, JobsCmd},
    auth::AuthCmd,
    backups::BackupsCmd,
    branches::BranchesCmd,
    gen::GenCmd,
    monitoring::MonitoringCmd,
    projects::ProjectsCmd,
    secrets::SecretsCmd,
    tables::TablesCmd,
};

// ── CLI definition ───────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "axm",
    bin_name = "axm",
    version = env!("CARGO_PKG_VERSION"),
    author = "Square Exp <info@squareexp.com>",
    about = "\x1b[38;2;255;140;0m\x1b[1mAxiomDB CLI\x1b[0m — multi-branch Postgres database management",
    long_about = None,
    after_help = concat!(
        "\x1b[38;2;255;140;0m\x1b[1mQuick start:\x1b[0m\n",
        "  \x1b[38;2;255;140;0maxm login\x1b[0m                      Log in to AxiomDB\n",
        "  \x1b[38;2;255;140;0maxm whoami\x1b[0m                     Show current session\n",
        "  \x1b[38;2;255;140;0maxm projects list\x1b[0m              List all projects\n",
        "  \x1b[38;2;255;140;0maxm projects use <id>\x1b[0m           Set active project context\n",
        "  \x1b[38;2;255;140;0maxm branches list\x1b[0m               List branches for active project\n",
        "  \x1b[38;2;255;140;0maxm branches urls --name feature-x\x1b[0m  Print branch Prisma URLs\n",
        "  \x1b[38;2;255;140;0maxm gen tk --branch feature-x\x1b[0m   Same branch URL flow via gen\n",
        "  \x1b[38;2;255;140;0maxm monitoring stream <id>\x1b[0m     Live SSE stream\n",
        "\n\x1b[38;2;255;140;0m\x1b[1mShortcuts:\x1b[0m\n",
        "  -li login, -lo logout, -me whoami\n",
        "  -au auth, -pr projects, -br branches, -mo monitoring\n",
        "  -tb tables, -bk backups, -se secrets, -ad audit, -jb jobs, -g gen\n",
        "  Subcommands also accept compact forms: -ls list, -cr create, -rm delete, -url urls\n",
    )
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // ── Root shortcuts ───────────────────────────────────────────────────────
    /// Log in to AxiomDB
    #[clap(visible_alias = "li")]
    Login {
        #[arg(short, long)]
        url: Option<String>,
        #[arg(short, long)]
        email: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Clear local session
    #[clap(visible_alias = "lo")]
    Logout,
    /// Show current session
    #[clap(visible_alias = "me")]
    Whoami,

    // ── Grouped commands ─────────────────────────────────────────────────────
    /// Authentication and session management
    #[clap(visible_alias = "au")]
    Auth {
        #[command(subcommand)]
        cmd: AuthCmd,
    },
    /// Manage projects
    #[clap(visible_alias = "pr", alias = "p", alias = "project")]
    Projects {
        #[command(subcommand)]
        cmd: ProjectsCmd,
    },
    /// Manage branches (max 10 per project)
    #[clap(visible_alias = "br")]
    Branches {
        #[command(subcommand)]
        cmd: BranchesCmd,
    },
    /// Real-time monitoring
    #[clap(visible_alias = "mo", alias = "mon")]
    Monitoring {
        #[command(subcommand)]
        cmd: MonitoringCmd,
    },
    /// Inspect database tables
    #[clap(visible_alias = "tb")]
    Tables {
        #[command(subcommand)]
        cmd: TablesCmd,
    },
    /// Backup catalog and restore
    #[clap(visible_alias = "bk")]
    Backups {
        #[command(subcommand)]
        cmd: BackupsCmd,
    },
    /// Generate cryptographic secrets
    #[clap(visible_alias = "se", alias = "sec")]
    Secrets {
        #[command(subcommand)]
        cmd: SecretsCmd,
    },
    /// View audit event log
    #[clap(visible_alias = "ad")]
    Audit {
        #[command(subcommand)]
        cmd: AuditCmd,
    },
    /// Inspect provisioning jobs
    #[clap(visible_alias = "jb")]
    Jobs {
        #[command(subcommand)]
        cmd: JobsCmd,
    },
    /// Generate helpers (Prisma URLs, tokens)
    #[clap(visible_alias = "g")]
    Gen {
        #[command(subcommand)]
        cmd: GenCmd,
    },
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!(
            "\n  {}  {}\n",
            "✖".truecolor(220, 60, 60).bold(),
            e.to_string().white()
        );
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse_from(normalize_shortcuts(std::env::args_os()));

    match cli.command {
        Commands::Login {
            url,
            email,
            password,
        } => commands::auth::do_login(url, email, password).await,
        Commands::Logout => commands::auth::do_logout(),
        Commands::Whoami => commands::auth::do_whoami(),

        Commands::Auth { cmd } => commands::auth::run(cmd).await,
        Commands::Projects { cmd } => commands::projects::run(cmd).await,
        Commands::Branches { cmd } => commands::branches::run(cmd).await,
        Commands::Monitoring { cmd } => commands::monitoring::run(cmd).await,
        Commands::Tables { cmd } => commands::tables::run(cmd).await,
        Commands::Backups { cmd } => commands::backups::run(cmd).await,
        Commands::Secrets { cmd } => commands::secrets::run(cmd).await,
        Commands::Audit { cmd } => commands::audit::run(cmd).await,
        Commands::Jobs { cmd } => commands::audit::run_jobs(cmd).await,
        Commands::Gen { cmd } => commands::gen::run(cmd).await,
    }
}

fn normalize_shortcuts<I>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = OsString>,
{
    args.into_iter()
        .map(|arg| match arg.to_str() {
            Some("-li") => OsString::from("login"),
            Some("-lo") => OsString::from("logout"),
            Some("-me") => OsString::from("whoami"),
            Some("-au") => OsString::from("auth"),
            Some("-pr") => OsString::from("projects"),
            Some("-br") => OsString::from("branches"),
            Some("-mo") => OsString::from("monitoring"),
            Some("-tb") => OsString::from("tables"),
            Some("-bk") => OsString::from("backups"),
            Some("-se") => OsString::from("secrets"),
            Some("-ad") => OsString::from("audit"),
            Some("-jb") => OsString::from("jobs"),
            Some("-g") => OsString::from("gen"),
            Some("-ls") => OsString::from("list"),
            Some("-cr") => OsString::from("create"),
            Some("-rm") => OsString::from("delete"),
            Some("-url") => OsString::from("urls"),
            Some("-tk") => OsString::from("tk"),
            _ => arg,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::normalize_shortcuts;
    use std::ffi::OsString;

    #[test]
    fn expands_root_and_nested_shortcuts() {
        let got = normalize_shortcuts(
            ["axm", "-br", "-url", "--name", "feature-proof"].map(OsString::from),
        );
        assert_eq!(
            got,
            ["axm", "branches", "urls", "--name", "feature-proof"].map(OsString::from)
        );
    }

    #[test]
    fn keeps_normal_help_flag_intact() {
        let got = normalize_shortcuts(["axm", "-h"].map(OsString::from));
        assert_eq!(got, ["axm", "-h"].map(OsString::from));
    }
}
