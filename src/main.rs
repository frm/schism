mod export;
mod input;
mod parse;
mod render;
mod tui;
mod types;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "schism", about = "Terminal diff reviewer")]
struct Cli {
    #[arg(long)] no_pager: bool,
    #[arg(short, long)] output: Option<std::path::PathBuf>,
    #[arg(long)] json: bool,
    #[arg(long)] tree: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        let input = input::read_piped_stdin()?;
        if input.is_empty() { return Ok(()); }

        let files = parse::parse_diff(&input);

        if cli.no_pager {
            let is_tty = std::io::IsTerminal::is_terminal(&std::io::stdout());
            render::pipe::render_pipe(&files, is_tty)?;
        } else if let Some((files, review_body)) = tui::viewport::run(files, cli.tree)? {
            if cli.json {
                let review = export::json::Review { body: review_body.as_deref(), files: &files };
                print!("{}", export::json::format_json(&review));
            } else {
                if let Some(s) = export::pipe::collect(&files, review_body.as_deref()) {
                    print!("{}", s);
                }
            }
        }
    }

    Ok(())
}
