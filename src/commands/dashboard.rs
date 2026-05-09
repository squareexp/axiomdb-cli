use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum DashboardCmd {
    /// Open the terminal dashboard
    #[clap(visible_alias = "open")]
    Open,
}

pub async fn run(cmd: DashboardCmd) -> Result<()> {
    match cmd {
        DashboardCmd::Open => crate::tui::app::run().await,
    }
}
