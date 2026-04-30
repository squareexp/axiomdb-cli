mod api;
mod art;
mod commands;
mod config;
mod display;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

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
    name = "axiom",
    version = env!("CARGO_PKG_VERSION"),
    author = "Square Exp <info@squareexp.com>",
    about = "\x1b[38;2;255;140;0m\x1b[1mAxiomDB CLI\x1b[0m — multi-branch Postgres database management",
    long_about = None,
    after_help = concat!(
        "\x1b[38;2;255;140;0m\x1b[1mQuick start:\x1b[0m\n",
        "  \x1b[38;2;255;140;0maxiom login\x1b[0m                      Log in to AxiomDB\n",
        "  \x1b[38;2;255;140;0maxiom whoami\x1b[0m                     Show current session\n",
        "  \x1b[38;2;255;140;0maxiom projects list\x1b[0m              List all projects\n",
        "  \x1b[38;2;255;140;0maxiom projects create\x1b[0m            Create a new project\n",
        "  \x1b[38;2;255;140;0maxiom branches list <id>\x1b[0m         List branches\n",
        "  \x1b[38;2;255;140;0maxiom monitoring summary <id>\x1b[0m    Health metrics\n",
        "  \x1b[38;2;255;140;0maxiom monitoring stream <id>\x1b[0m     Live SSE stream\n",
        "  \x1b[38;2;255;140;0maxiom gen tk <id>\x1b[0m                Prisma-ready DB URLs\n",
        "  \x1b[38;2;255;140;0maxiom secrets generate\x1b[0m           Generate a secret\n",
        "  \x1b[38;2;255;140;0maxiom audit list\x1b[0m                 View audit log\n",
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
    Login {
        #[arg(short, long)] url: Option<String>,
        #[arg(short, long)] email: Option<String>,
        #[arg(short, long)] password: Option<String>,
    },
    /// Clear local session
    Logout,
    /// Show current session
    Whoami,

    // ── Grouped commands ─────────────────────────────────────────────────────
    /// Authentication and session management
    Auth {
        #[command(subcommand)] cmd: AuthCmd,
    },
    /// Manage projects
    #[clap(alias = "p", alias = "project")]
    Projects {
        #[command(subcommand)] cmd: ProjectsCmd,
    },
    /// Manage branches (max 10 per project)
    #[clap(alias = "br")]
    Branches {
        #[command(subcommand)] cmd: BranchesCmd,
    },
    /// Real-time monitoring
    #[clap(alias = "mon")]
    Monitoring {
        #[command(subcommand)] cmd: MonitoringCmd,
    },
    /// Inspect database tables
    #[clap(alias = "tb")]
    Tables {
        #[command(subcommand)] cmd: TablesCmd,
    },
    /// Backup catalog and restore
    #[clap(alias = "bk")]
    Backups {
        #[command(subcommand)] cmd: BackupsCmd,
    },
    /// Generate cryptographic secrets
    #[clap(alias = "sec")]
    Secrets {
        #[command(subcommand)] cmd: SecretsCmd,
    },
    /// View audit event log
    Audit {
        #[command(subcommand)] cmd: AuditCmd,
    },
    /// Inspect provisioning jobs
    Jobs {
        #[command(subcommand)] cmd: JobsCmd,
    },
    /// Generate helpers (Prisma URLs, tokens)
    Gen {
        #[command(subcommand)] cmd: GenCmd,
    },
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("\n  {}  {}\n", "✖".truecolor(220, 60, 60).bold(), e.to_string().white());
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login { url, email, password } => {
            commands::auth::do_login(url, email, password).await
        }
        Commands::Logout  => commands::auth::do_logout(),
        Commands::Whoami  => commands::auth::do_whoami(),

        Commands::Auth       { cmd } => commands::auth::run(cmd).await,
        Commands::Projects   { cmd } => commands::projects::run(cmd).await,
        Commands::Branches   { cmd } => commands::branches::run(cmd).await,
        Commands::Monitoring { cmd } => commands::monitoring::run(cmd).await,
        Commands::Tables     { cmd } => commands::tables::run(cmd).await,
        Commands::Backups    { cmd } => commands::backups::run(cmd).await,
        Commands::Secrets    { cmd } => commands::secrets::run(cmd).await,
        Commands::Audit      { cmd } => commands::audit::run(cmd).await,
        Commands::Jobs       { cmd } => commands::audit::run_jobs(cmd).await,
        Commands::Gen        { cmd } => commands::gen::run(cmd).await,
    }
}
