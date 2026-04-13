use anyhow::{anyhow, Result};

use super::types::PrRef;

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

fn parse_pr_url(url: &str) -> Result<PrRef> {
    let path = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        .ok_or_else(|| anyhow!("expected a github.com PR URL"))?;

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
