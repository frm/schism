mod export;
mod github;
mod input;
mod parse;
mod render;
mod tui;
mod types;

use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::Result;
use clap::Parser;

use github::PrReviewContext;

#[derive(Parser)]
#[command(name = "schism", about = "Terminal diff reviewer")]
struct Cli {
    #[arg(long)] no_pager: bool,
    #[arg(long)] json: bool,
    #[arg(long)] tree: bool,
    #[arg(long)] pr: Option<String>,
    #[arg(long)] debug: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let (input, raw_input, pr_context) = if let Some(pr_ref) = &cli.pr {
        github::check_installed()?;
        let pr = github::parse_pr_ref(pr_ref)?;
        eprint!("Fetching pull request...");
        let diff = github::fetch_diff(&pr)?;
        let metadata = github::fetch_metadata(&pr)?;
        let commits = github::fetch_commits(&pr).unwrap_or_default();
        eprintln!(" done");
        let ctx = PrReviewContext { pr, metadata, commits };
        (diff.clone(), diff, Some(ctx))
    } else if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        let (stripped, raw) = input::read_piped_stdin()?;
        (stripped, raw, None)
    } else {
        return Ok(());
    };

    if input.is_empty() {
        return Ok(());
    }

    let files = parse::parse_diff(&input);

    if files.is_empty() {
        return passthrough(&raw_input, cli.no_pager);
    }

    if cli.no_pager {
        let is_tty = std::io::IsTerminal::is_terminal(&std::io::stdout());
        render::pipe::render_pipe(&files, is_tty)?;
    } else if let Some((files, review_body)) = tui::viewport::run(files, cli.tree, pr_context, cli.debug)? {
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

fn passthrough(input: &str, no_pager: bool) -> Result<()> {
    if no_pager {
        std::io::stdout().write_all(input.as_bytes())?;
        return Ok(());
    }

    let pager = std::env::var("SCHISM_PAGER")
        .or_else(|_| std::env::var("PAGER"))
        .unwrap_or_else(|_| "less".to_string());

    let parts: Vec<&str> = pager.split_whitespace().collect();
    let (bin, args) = parts.split_first().unwrap_or((&"less", &[]));

    let is_less = std::path::Path::new(bin)
        .file_name()
        .map(|n| n == "less")
        .unwrap_or(false);

    let mut cmd = Command::new(bin);
    if is_less && args.is_empty() {
        cmd.args(["--RAW-CONTROL-CHARS", "--quit-if-one-screen"]);
    } else {
        cmd.args(args);
    }

    let mut child = cmd.stdin(Stdio::piped()).spawn()?;
    let stdin = child.stdin.as_mut().unwrap();
    // Ignore BrokenPipe — user quit the pager early
    let _ = stdin.write_all(input.as_bytes());
    let _ = stdin.flush();
    drop(child.stdin.take());
    child.wait()?;

    Ok(())
}
