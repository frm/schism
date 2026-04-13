use anyhow::Result;

use super::types::{PrReviewContext, ReviewEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCommentPayload {
    pub path: String,
    pub line: u32,
    pub side: &'static str,
    pub body: String,
    pub content: String,
}

pub fn collect_review_comments(files: &[crate::types::DiffFile]) -> Vec<ReviewCommentPayload> {
    let mut comments = Vec::new();

    for file in files {
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
                    content: line.content.clone(),
                });
            }
        }

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
                        content: line.content.clone(),
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
    commit_id: &str,
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
        "commit_id": commit_id,
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
    let payload = build_review_payload(body, event, &context.metadata.head_ref_oid, files);
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
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !stdout.is_empty() {
            anyhow::bail!("review submit failed: {}\n{}", stderr, stdout);
        }
        anyhow::bail!("review submit failed: {}", stderr);
    }

    Ok(())
}
