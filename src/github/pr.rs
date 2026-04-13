use std::process::Command;

use anyhow::{anyhow, Result};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrRef {
    pub owner: String,
    pub repo: String,
    pub number: u64,
}

/// Parse `owner/repo#123` or `https://github.com/owner/repo/pull/123`.
pub fn parse_pr_ref(input: &str) -> Result<PrRef> {
    if input.contains("://") {
        return parse_pr_url(input);
    }

    let (repo_part, number_part) = input
        .split_once('#')
        .ok_or_else(|| anyhow!("expected owner/repo#number or GitHub PR URL"))?;

    let (owner, repo) = repo_part
        .split_once('/')
        .ok_or_else(|| anyhow!("expected owner/repo#number or GitHub PR URL"))?;

    if owner.is_empty() || repo.is_empty() {
        return Err(anyhow!("expected owner/repo#number or GitHub PR URL"));
    }

    let number = number_part.parse::<u64>()?;

    Ok(PrRef {
        owner: owner.to_string(),
        repo: repo.to_string(),
        number,
    })
}

/// Parse `https://github.com/owner/repo/pull/123`.
fn parse_pr_url(url: &str) -> Result<PrRef> {
    let path = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        .ok_or_else(|| anyhow!("expected a github.com PR URL"))?;

    // path = "owner/repo/pull/123" (possibly with trailing slash)
    let parts: Vec<&str> = path.trim_end_matches('/').split('/').collect();

    if parts.len() < 4 || parts[2] != "pull" {
        return Err(anyhow!("expected URL like https://github.com/owner/repo/pull/123"));
    }

    let number = parts[3].parse::<u64>()?;

    Ok(PrRef {
        owner: parts[0].to_string(),
        repo: parts[1].to_string(),
        number,
    })
}

impl PrRef {
    fn repo_slug(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrMetadata {
    pub title: String,
    pub url: String,
    pub author: String,
    pub body: String,
    pub head_branch: String,
    pub base_branch: String,
    pub head_ref_oid: String,
    pub base_ref_oid: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewEvent {
    Comment,
    Approve,
    RequestChanges,
}

impl ReviewEvent {
    pub fn as_api_value(self) -> &'static str {
        match self {
            ReviewEvent::Comment => "COMMENT",
            ReviewEvent::Approve => "APPROVE",
            ReviewEvent::RequestChanges => "REQUEST_CHANGES",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ReviewEvent::Comment => "comment",
            ReviewEvent::Approve => "approve",
            ReviewEvent::RequestChanges => "request changes",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrReviewContext {
    pub pr: PrRef,
    pub metadata: PrMetadata,
    pub commits: Vec<PrCommit>,
}

// ── command builders (testable without gh) ───────────────────────────────

pub fn build_diff_args(pr: &PrRef) -> Vec<String> {
    vec![
        "pr".into(), "diff".into(),
        pr.number.to_string(),
        "--repo".into(), pr.repo_slug(),
    ]
}

pub fn build_view_args(pr: &PrRef) -> Vec<String> {
    vec![
        "pr".into(), "view".into(),
        pr.number.to_string(),
        "--repo".into(), pr.repo_slug(),
        "--json".into(), "headRefOid,baseRefOid,headRefName,baseRefName,title,url,author,body".into(),
    ]
}

// ── gh execution ────────────────────────────────────────────────────

pub fn check_gh_installed() -> Result<()> {
    match Command::new("gh").arg("--version").output() {
        Ok(o) if o.status.success() => Ok(()),
        Ok(_) => Err(anyhow!("gh found but returned an error — run `gh auth login`")),
        Err(_) => Err(anyhow!("gh not found — install it from https://cli.github.com")),
    }
}

fn run_gh(args: &[String]) -> Result<String> {
    let output = Command::new("gh")
        .args(args)
        .env("NO_COLOR", "1")
        .env("GH_FORCE_TTY", "")
        .output()
        .map_err(|e| anyhow!("failed to run gh: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh failed: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[derive(Debug, Clone)]
pub struct PrCommit {
    pub sha: String,
    pub message: String,
    pub author: String,
}

pub fn build_commits_args(pr: &PrRef) -> Vec<String> {
    vec![
        "pr".into(), "view".into(),
        pr.number.to_string(),
        "--repo".into(), pr.repo_slug(),
        "--json".into(), "commits".into(),
    ]
}

pub fn fetch_commits(pr: &PrRef) -> Result<Vec<PrCommit>> {
    let json_str = run_gh(&build_commits_args(pr))?;
    let v: Value = serde_json::from_str(&json_str)?;

    let commits = v["commits"].as_array()
        .ok_or_else(|| anyhow!("no commits array in gh response"))?
        .iter()
        .map(|c| {
            let sha = c["oid"].as_str().unwrap_or("").to_string();
            let message = c["messageHeadline"].as_str().unwrap_or("").to_string();
            let author = c["authors"].as_array()
                .and_then(|a| a.first())
                .and_then(|a| a["login"].as_str())
                .unwrap_or("")
                .to_string();
            PrCommit { sha, message, author }
        })
        .collect();

    Ok(commits)
}

pub fn fetch_commit_diff(pr: &PrRef, sha: &str) -> Result<String> {
    let args = vec![
        "api".into(),
        format!("/repos/{}/{}/commits/{}", pr.owner, pr.repo, sha),
        "-H".into(), "Accept: application/vnd.github.diff".into(),
    ];
    run_gh(&args)
}

pub fn fetch_file_content(pr: &PrRef, path: &str, ref_oid: &str) -> Result<String> {
    let args = vec![
        "api".into(),
        format!("/repos/{}/{}/contents/{}?ref={}", pr.owner, pr.repo, path, ref_oid),
        "-H".into(), "Accept: application/vnd.github.raw+json".into(),
    ];
    let raw = run_gh(&args)?;
    // GitHub may return JSON with "content" field or raw text depending on accept header
    // Try to parse as JSON first, fall back to raw
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
        if let Some(content) = v["content"].as_str() {
            // base64 decode
            let cleaned: String = content.chars().filter(|c| !c.is_whitespace()).collect();
            if let Ok(bytes) = base64_decode(&cleaned) {
                return Ok(String::from_utf8_lossy(&bytes).into_owned());
            }
        }
    }
    Ok(raw)
}

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in input.as_bytes() {
        if b == b'=' { break; }
        let val = table.iter().position(|&t| t == b)
            .ok_or_else(|| anyhow!("invalid base64"))? as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Ok(out)
}

// ── review comment mapping ────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCommentPayload {
    pub path: String,
    pub line: u32,
    pub side: &'static str,
    pub body: String,
}

pub fn collect_review_comments(files: &[crate::types::DiffFile]) -> Vec<ReviewCommentPayload> {
    let mut comments = Vec::new();

    for file in files {
        // File-level comment → attach to first commentable line
        if let Some(file_comment) = &file.comment {
            let first = file.hunks.iter()
                .flat_map(|h| &h.lines)
                .find(|l| l.new_lineno.is_some() || l.old_lineno.is_some());
            if let Some(line) = first {
                let (line_no, side) = match (line.new_lineno, line.old_lineno) {
                    (Some(n), _) => (n, "RIGHT"),
                    (None, Some(n)) => (n, "LEFT"),
                    _ => continue,
                };
                comments.push(ReviewCommentPayload {
                    path: file.path.clone(), line: line_no, side,
                    body: file_comment.text.clone(),
                });
            }
        }

        // Line-level comments
        for hunk in &file.hunks {
            for line in &hunk.lines {
                if let Some(comment) = &line.comment {
                    let (line_no, side) = match (&line.kind, line.new_lineno, line.old_lineno) {
                        (crate::types::LineKind::Removed, _, Some(n)) => (n, "LEFT"),
                        (_, Some(n), _) => (n, "RIGHT"),
                        _ => continue,
                    };
                    comments.push(ReviewCommentPayload {
                        path: file.path.clone(), line: line_no, side,
                        body: comment.text.clone(),
                    });
                }
            }
        }
    }

    comments
}

pub fn build_review_payload(
    body: &str,
    event: ReviewEvent,
    files: &[crate::types::DiffFile],
) -> serde_json::Value {
    let comments: Vec<_> = collect_review_comments(files).into_iter().map(|c| {
        serde_json::json!({
            "path": c.path,
            "line": c.line,
            "side": c.side,
            "body": c.body,
        })
    }).collect();

    serde_json::json!({
        "body": body,
        "event": event.as_api_value(),
        "comments": comments,
    })
}

pub fn submit_review(
    context: &PrReviewContext,
    body: &str,
    event: ReviewEvent,
    files: &[crate::types::DiffFile],
) -> Result<()> {
    let payload = build_review_payload(body, event, files);
    let endpoint = format!(
        "/repos/{}/{}/pulls/{}/reviews",
        context.pr.owner, context.pr.repo, context.pr.number,
    );

    let mut child = std::process::Command::new("gh")
        .args(["api", "--method", "POST", &endpoint, "--input", "-"])
        .env("NO_COLOR", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    {
        use std::io::Write;
        child.stdin.as_mut().unwrap()
            .write_all(payload.to_string().as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        anyhow::bail!("review submit failed: {}", String::from_utf8_lossy(&output.stderr).trim());
    }

    Ok(())
}

// ── gh execution ────────────────────────────────────────────────────

pub fn fetch_diff(pr: &PrRef) -> Result<String> {
    let raw = run_gh(&build_diff_args(pr))?;
    Ok(crate::input::strip_ansi(&raw))
}

pub fn fetch_metadata(pr: &PrRef) -> Result<PrMetadata> {
    let json_str = run_gh(&build_view_args(pr))?;
    let v: Value = serde_json::from_str(&json_str)?;

    let author = v["author"]["login"].as_str().unwrap_or("").to_string();
    let body = v["body"].as_str().unwrap_or("").to_string();

    Ok(PrMetadata {
        title: v["title"].as_str().unwrap_or("").to_string(),
        url: v["url"].as_str().unwrap_or("").to_string(),
        author,
        body,
        head_branch: v["headRefName"].as_str().unwrap_or("").to_string(),
        base_branch: v["baseRefName"].as_str().unwrap_or("").to_string(),
        head_ref_oid: v["headRefOid"].as_str().unwrap_or("").to_string(),
        base_ref_oid: v["baseRefOid"].as_str().unwrap_or("").to_string(),
    })
}
