mod types;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "schism", about = "Terminal diff reviewer")]
struct Cli {
    /// Pretty-print mode (no TUI)
    #[arg(long)]
    no_pager: bool,

    /// Output file for markdown export
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let input = std::io::read_to_string(std::io::stdin())?;

    if input.is_empty() {
        return Ok(());
    }

    if cli.no_pager {
        eprintln!("no-pager mode: TODO");
    } else {
        eprintln!("interactive mode: TODO");
    }

    Ok(())
}
