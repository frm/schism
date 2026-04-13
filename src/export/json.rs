use crate::render::line::LineRenderer;
use crate::types::DiffFile;

pub struct Review<'a> {
    pub body: Option<&'a str>,
    pub files: &'a [DiffFile],
}

pub fn format_json(review: &Review) -> String {
    let body_field = match review.body {
        Some(b) => format!("\"{}\"", escape(b)),
        None    => "null".to_string(),
    };

    let mut comments = Vec::new();
    for file in review.files {
        // File-level comment (line: 0, no change field)
        if let Some(comment) = &file.comment {
            comments.push(format!(
                "    {{\n      \"path\": \"{}\",\n      \"line\": 0,\n      \"change\": null,\n      \"text\": \"{}\"\n    }}",
                escape(&file.path),
                escape(&comment.text),
            ));
        }

        // Line-level comments
        for hunk in &file.hunks {
            for line in &hunk.lines {
                if let Some(comment) = &line.comment {
                    let lineno = line.new_lineno.or(line.old_lineno).unwrap_or(0);
                    let prefix = LineRenderer::line_prefix(&line.kind);
                    let change = format!("{}{}", prefix, line.content);

                    comments.push(format!(
                        "    {{\n      \"path\": \"{}\",\n      \"line\": {},\n      \"change\": \"{}\",\n      \"text\": \"{}\"\n    }}",
                        escape(&file.path),
                        lineno,
                        escape(&change),
                        escape(&comment.text),
                    ));
                }
            }
        }
    }

    let comments_str = if comments.is_empty() {
        "[]".to_string()
    } else {
        format!("[\n{}\n  ]", comments.join(",\n"))
    };

    format!(
        "{{\n  \"body\": {},\n  \"comments\": {}\n}}\n",
        body_field,
        comments_str,
    )
}

fn escape(s: &str) -> String {
    s.chars().flat_map(|c| match c {
        '"'  => vec!['\\', '"'],
        '\\' => vec!['\\', '\\'],
        '\n' => vec!['\\', 'n'],
        '\r' => vec!['\\', 'r'],
        '\t' => vec!['\\', 't'],
        c    => vec![c],
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DiffFile, Hunk, DiffLine, LineKind, Comment};

    fn make_file(path: &str, comment: &str) -> DiffFile {
        DiffFile {
            path: path.to_string(),
            old_path: None,
            status: crate::types::FileStatus::Modified,
            hunks: vec![Hunk {
                header: String::new(),
                old_start: 1, old_count: 1, new_start: 1, new_count: 1,
                lines: vec![DiffLine {
                    kind: LineKind::Added,
                    content: "let x = 1;".to_string(),
                    old_lineno: None,
                    new_lineno: Some(10),
                    comment: Some(Comment { text: comment.to_string() }),
                }],
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
    fn no_body_no_comments() {
        let out = format_json(&Review { body: None, files: &[] });
        assert_eq!(out, "{\n  \"body\": null,\n  \"comments\": []\n}\n");
    }

    #[test]
    fn body_and_comment() {
        let file = make_file("src/main.rs", "looks good");
        let out = format_json(&Review { body: Some("overall fine"), files: &[file] });
        assert!(out.contains("\"body\": \"overall fine\""));
        assert!(out.contains("\"path\": \"src/main.rs\""));
        assert!(out.contains("\"line\": 10"));
        assert!(out.contains("\"change\": \"+let x = 1;\""));
        assert!(out.contains("\"text\": \"looks good\""));
    }

    #[test]
    fn escapes_special_chars() {
        let file = make_file("src/lib.rs", "say \"hello\"");
        let out = format_json(&Review { body: None, files: &[file] });
        assert!(out.contains(r#"\"hello\""#));
    }

    #[test]
    fn multiline_comment_escaped() {
        let file = make_file("a.rs", "line one\nline two");
        let out = format_json(&Review { body: None, files: &[file] });
        assert!(out.contains(r#"line one\nline two"#));
    }
}
