mod export;
mod github;
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
    #[arg(long)] json: bool,
    #[arg(long)] tree: bool,
    #[arg(long)] pr: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let input = if let Some(pr_ref) = &cli.pr {
        github::pr::check_gh_installed()?;
        let pr = github::pr::parse_pr_ref(pr_ref)?;
        github::pr::fetch_diff(&pr)?
    } else if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        input::read_piped_stdin()?
    } else {
        return Ok(());
    };

    if input.is_empty() {
        return Ok(());
    }

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

    Ok(())
}
