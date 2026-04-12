mod parse;
mod render;
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

    let files = parse::parse_diff(&input);

    if cli.no_pager {
        let is_tty = std::io::IsTerminal::is_terminal(&std::io::stdout());
        render::pipe::render_pipe(&files, is_tty)?;
    } else {
        eprintln!("interactive mode: TODO");
    }

    Ok(())
}
