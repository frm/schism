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
    pub head_branch: String,
    pub base_branch: String,
    pub head_ref_oid: String,
    pub base_ref_oid: String,
}

#[derive(Debug, Clone)]
pub struct PrReviewContext {
    pub pr: PrRef,
    pub metadata: PrMetadata,
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
        "--json".into(), "headRefOid,baseRefOid,headRefName,baseRefName,title,url,author".into(),
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
        .output()
        .map_err(|e| anyhow!("failed to run gh: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh failed: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn fetch_diff(pr: &PrRef) -> Result<String> {
    run_gh(&build_diff_args(pr))
}

pub fn fetch_metadata(pr: &PrRef) -> Result<PrMetadata> {
    let json_str = run_gh(&build_view_args(pr))?;
    let v: Value = serde_json::from_str(&json_str)?;

    let author = v["author"]["login"].as_str().unwrap_or("").to_string();

    Ok(PrMetadata {
        title: v["title"].as_str().unwrap_or("").to_string(),
        url: v["url"].as_str().unwrap_or("").to_string(),
        author,
        head_branch: v["headRefName"].as_str().unwrap_or("").to_string(),
        base_branch: v["baseRefName"].as_str().unwrap_or("").to_string(),
        head_ref_oid: v["headRefOid"].as_str().unwrap_or("").to_string(),
        base_ref_oid: v["baseRefOid"].as_str().unwrap_or("").to_string(),
    })
}
