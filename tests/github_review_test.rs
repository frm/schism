use schism::github::pr::{ReviewEvent, collect_review_comments, build_review_payload};
use schism::types::*;

#[test]
fn review_event_serializes_to_github_names() {
    assert_eq!(ReviewEvent::Comment.as_api_value(), "COMMENT");
    assert_eq!(ReviewEvent::Approve.as_api_value(), "APPROVE");
    assert_eq!(ReviewEvent::RequestChanges.as_api_value(), "REQUEST_CHANGES");
}

fn make_file() -> DiffFile {
    DiffFile {
        path: "src/lib.rs".to_string(),
        old_path: None,
        status: FileStatus::Modified,
        hunks: vec![Hunk {
            header: "@@ -1,2 +1,3 @@".to_string(),
            old_start: 1, old_count: 2, new_start: 1, new_count: 3,
            lines: vec![
                DiffLine {
                    kind: LineKind::Added,
                    content: "let x = 1;".to_string(),
                    old_lineno: None, new_lineno: Some(2),
                    comment: Some(Comment { text: "nit".to_string() }),
                },
                DiffLine {
                    kind: LineKind::Removed,
                    content: "let y = 2;".to_string(),
                    old_lineno: Some(3), new_lineno: None,
                    comment: Some(Comment { text: "why removed?".to_string() }),
                },
            ],
            collapsed: false,
        }],
        collapsed: false,
        binary: false,
        comment: None,
        old_sha: None,
        new_sha: None,
    }
}

#[test]
fn maps_added_line_to_right_side() {
    let files = vec![make_file()];
    let comments = collect_review_comments(&files);
    assert_eq!(comments[0].path, "src/lib.rs");
    assert_eq!(comments[0].line, 2);
    assert_eq!(comments[0].side, "RIGHT");
    assert_eq!(comments[0].body, "nit");
}

#[test]
fn maps_removed_line_to_left_side() {
    let files = vec![make_file()];
    let comments = collect_review_comments(&files);
    assert_eq!(comments[1].line, 3);
    assert_eq!(comments[1].side, "LEFT");
    assert_eq!(comments[1].body, "why removed?");
}

#[test]
fn builds_comment_review_payload() {
    let files = vec![make_file()];
    let payload = build_review_payload("Looks good overall", ReviewEvent::Comment, &files);
    assert_eq!(payload["body"], serde_json::json!("Looks good overall"));
    assert_eq!(payload["event"], serde_json::json!("COMMENT"));
    assert_eq!(payload["comments"].as_array().unwrap().len(), 2);
}

#[test]
fn builds_approve_review_payload() {
    let files = vec![make_file()];
    let payload = build_review_payload("", ReviewEvent::Approve, &files);
    assert_eq!(payload["event"], serde_json::json!("APPROVE"));
}
