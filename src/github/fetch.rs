use anyhow::{anyhow, Result};
use serde_json::Value;

use super::gh;
use super::types::{PrRef, PrMetadata, PrCommit};

pub fn fetch_diff(pr: &PrRef) -> Result<String> {
    let args = vec![
        "pr".into(), "diff".into(),
        pr.number.to_string(),
        "--repo".into(), pr.repo_slug(),
    ];
    let raw = gh::run(&args)?;
    Ok(crate::input::strip_ansi(&raw))
}

pub fn fetch_metadata(pr: &PrRef) -> Result<PrMetadata> {
    let args = vec![
        "pr".into(), "view".into(),
        pr.number.to_string(),
        "--repo".into(), pr.repo_slug(),
        "--json".into(), "headRefOid,baseRefOid,headRefName,baseRefName,title,url,author,body".into(),
    ];
    let json_str = gh::run(&args)?;
    let v: Value = serde_json::from_str(&json_str)?;

    Ok(PrMetadata {
        title: v["title"].as_str().unwrap_or("").to_string(),
        url: v["url"].as_str().unwrap_or("").to_string(),
        author: v["author"]["login"].as_str().unwrap_or("").to_string(),
        body: v["body"].as_str().unwrap_or("").to_string(),
        head_branch: v["headRefName"].as_str().unwrap_or("").to_string(),
        base_branch: v["baseRefName"].as_str().unwrap_or("").to_string(),
        head_ref_oid: v["headRefOid"].as_str().unwrap_or("").to_string(),
        base_ref_oid: v["baseRefOid"].as_str().unwrap_or("").to_string(),
    })
}

pub fn fetch_commits(pr: &PrRef) -> Result<Vec<PrCommit>> {
    let args = vec![
        "pr".into(), "view".into(),
        pr.number.to_string(),
        "--repo".into(), pr.repo_slug(),
        "--json".into(), "commits".into(),
    ];
    let json_str = gh::run(&args)?;
    let v: Value = serde_json::from_str(&json_str)?;

    let commits = v["commits"].as_array()
        .ok_or_else(|| anyhow!("no commits array in gh response"))?
        .iter()
        .map(|c| PrCommit {
            sha: c["oid"].as_str().unwrap_or("").to_string(),
            message: c["messageHeadline"].as_str().unwrap_or("").to_string(),
            author: c["authors"].as_array()
                .and_then(|a| a.first())
                .and_then(|a| a["login"].as_str())
                .unwrap_or("")
                .to_string(),
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
    gh::run(&args)
}

pub fn fetch_file_content(pr: &PrRef, path: &str, ref_oid: &str) -> Result<String> {
    let args = vec![
        "api".into(),
        format!("/repos/{}/{}/contents/{}?ref={}", pr.owner, pr.repo, path, ref_oid),
        "-H".into(), "Accept: application/vnd.github.raw+json".into(),
    ];
    let raw = gh::run(&args)?;

    if let Ok(v) = serde_json::from_str::<Value>(&raw) {
        if let Some(content) = v["content"].as_str() {
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
