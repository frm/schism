use crate::render::line::LineRenderer;
use crate::types::DiffFile;

pub fn collect(files: &[DiffFile], body: Option<&str>) -> Option<String> {
    let mut comments = String::new();

    for file in files {
        if let Some(comment) = &file.comment {
            if !comments.is_empty() { comments.push('\n'); }
            comments.push_str(&format!("{}\n", file.path));
            comments.push_str(&comment.text);
            comments.push('\n');
        }

        for hunk in &file.hunks {
            for line in &hunk.lines {
                if let Some(comment) = &line.comment {
                    let lineno = line.new_lineno.or(line.old_lineno).unwrap_or(0);
                    let prefix = LineRenderer::line_prefix(&line.kind);

                    if !comments.is_empty() { comments.push('\n'); }
                    comments.push_str(&format!("{}:{}\n", file.path, lineno));
                    comments.push_str(&format!("{}{}\n", prefix, line.content));
                    comments.push_str(&comment.text);
                    comments.push('\n');
                }
            }
        }
    }

    match (body, comments.is_empty()) {
        (None, true)       => None,
        (None, false)      => Some(comments),
        (Some(b), true)    => Some(b.to_string()),
        (Some(b), false)   => Some(format!("{}\n\n{}", b, comments)),
    }
}
