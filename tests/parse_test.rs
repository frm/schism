use schism::parse::parse_diff;
use schism::types::{FileStatus, LineKind};

#[test]
fn test_parse_single_file_modification() {
    let input = r#"diff --git a/src/main.rs b/src/main.rs
index abc1234..def5678 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,4 +1,5 @@
 fn main() {
-    println!("hello");
+    println!("hello world");
+    println!("goodbye");
 }
"#;

    let files = parse_diff(input);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "src/main.rs");
    assert_eq!(files[0].status, FileStatus::Modified);
    assert_eq!(files[0].hunks.len(), 1);

    let hunk = &files[0].hunks[0];
    assert_eq!(hunk.old_start, 1);
    assert_eq!(hunk.old_count, 4);
    assert_eq!(hunk.new_start, 1);
    assert_eq!(hunk.new_count, 5);
    assert_eq!(hunk.lines.len(), 5);

    assert_eq!(hunk.lines[0].kind, LineKind::Context);
    assert_eq!(hunk.lines[0].content, "fn main() {");
    assert_eq!(hunk.lines[0].old_lineno, Some(1));
    assert_eq!(hunk.lines[0].new_lineno, Some(1));

    assert_eq!(hunk.lines[1].kind, LineKind::Removed);
    assert_eq!(hunk.lines[1].content, "    println!(\"hello\");");
    assert_eq!(hunk.lines[1].old_lineno, Some(2));
    assert_eq!(hunk.lines[1].new_lineno, None);

    assert_eq!(hunk.lines[2].kind, LineKind::Added);
    assert_eq!(hunk.lines[2].content, "    println!(\"hello world\");");
    assert_eq!(hunk.lines[2].old_lineno, None);
    assert_eq!(hunk.lines[2].new_lineno, Some(2));

    assert_eq!(hunk.lines[3].kind, LineKind::Added);
    assert_eq!(hunk.lines[3].content, "    println!(\"goodbye\");");
    assert_eq!(hunk.lines[3].old_lineno, None);
    assert_eq!(hunk.lines[3].new_lineno, Some(3));

    assert_eq!(hunk.lines[4].kind, LineKind::Context);
    assert_eq!(hunk.lines[4].content, "}");
    assert_eq!(hunk.lines[4].old_lineno, Some(3));
    assert_eq!(hunk.lines[4].new_lineno, Some(4));
}

#[test]
fn test_parse_new_file() {
    let input = r#"diff --git a/new_file.txt b/new_file.txt
new file mode 100644
index 0000000..abc1234
--- /dev/null
+++ b/new_file.txt
@@ -0,0 +1,2 @@
+line one
+line two
"#;

    let files = parse_diff(input);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "new_file.txt");
    assert_eq!(files[0].status, FileStatus::Added);
    assert_eq!(files[0].hunks[0].lines.len(), 2);
    assert!(files[0].hunks[0].lines.iter().all(|l| l.kind == LineKind::Added));
}

#[test]
fn test_parse_deleted_file() {
    let input = r#"diff --git a/old_file.txt b/old_file.txt
deleted file mode 100644
index abc1234..0000000
--- a/old_file.txt
+++ /dev/null
@@ -1,2 +0,0 @@
-line one
-line two
"#;

    let files = parse_diff(input);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "old_file.txt");
    assert_eq!(files[0].status, FileStatus::Deleted);
}

#[test]
fn test_parse_renamed_file() {
    let input = r#"diff --git a/old_name.rs b/new_name.rs
similarity index 95%
rename from old_name.rs
rename to new_name.rs
index abc1234..def5678 100644
--- a/old_name.rs
+++ b/new_name.rs
@@ -1,3 +1,3 @@
 fn hello() {
-    println!("old");
+    println!("new");
 }
"#;

    let files = parse_diff(input);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "new_name.rs");
    assert_eq!(files[0].old_path, Some("old_name.rs".to_string()));
    assert_eq!(files[0].status, FileStatus::Renamed);
}

#[test]
fn test_parse_multiple_files() {
    let input = r#"diff --git a/file_a.rs b/file_a.rs
index abc..def 100644
--- a/file_a.rs
+++ b/file_a.rs
@@ -1,2 +1,2 @@
-old a
+new a
diff --git a/file_b.rs b/file_b.rs
index abc..def 100644
--- a/file_b.rs
+++ b/file_b.rs
@@ -1,2 +1,2 @@
-old b
+new b
"#;

    let files = parse_diff(input);

    assert_eq!(files.len(), 2);
    assert_eq!(files[0].path, "file_a.rs");
    assert_eq!(files[1].path, "file_b.rs");
}

#[test]
fn test_parse_multiple_hunks() {
    let input = r#"diff --git a/src/lib.rs b/src/lib.rs
index abc..def 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,3 @@
 fn first() {
-    old();
+    new();
 }
@@ -10,3 +10,3 @@
 fn second() {
-    old();
+    new();
 }
"#;

    let files = parse_diff(input);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].hunks.len(), 2);
    assert_eq!(files[0].hunks[0].old_start, 1);
    assert_eq!(files[0].hunks[1].old_start, 10);
}

#[test]
fn test_parse_binary_file() {
    let input = r#"diff --git a/image.png b/image.png
index abc..def 100644
Binary files a/image.png and b/image.png differ
"#;

    let files = parse_diff(input);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "image.png");
    assert_eq!(files[0].hunks.len(), 0);
}

#[test]
fn test_parse_no_newline_at_eof() {
    let input = "diff --git a/file.txt b/file.txt\nindex abc..def 100644\n--- a/file.txt\n+++ b/file.txt\n@@ -1,2 +1,2 @@\n-old line\n\\ No newline at end of file\n+new line\n\\ No newline at end of file\n";

    let files = parse_diff(input);

    assert_eq!(files.len(), 1);
    let lines: Vec<_> = files[0].hunks[0].lines.iter().map(|l| &l.kind).collect();
    assert_eq!(lines, vec![&LineKind::Removed, &LineKind::Added]);
}

#[test]
fn test_parse_empty_input() {
    let files = parse_diff("");
    assert!(files.is_empty());
}
