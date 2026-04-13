use schism::github::parse_pr_ref;

#[test]
fn parses_owner_repo_number() {
    let pr = parse_pr_ref("frm/schism#123").unwrap();
    assert_eq!(pr.owner, "frm");
    assert_eq!(pr.repo, "schism");
    assert_eq!(pr.number, 123);
}

#[test]
fn rejects_missing_hash() {
    assert!(parse_pr_ref("frm/schism").is_err());
}

#[test]
fn rejects_non_numeric_pr_number() {
    assert!(parse_pr_ref("frm/schism#abc").is_err());
}

#[test]
fn parses_github_url() {
    let pr = parse_pr_ref("https://github.com/frm/schism/pull/42").unwrap();
    assert_eq!(pr.owner, "frm");
    assert_eq!(pr.repo, "schism");
    assert_eq!(pr.number, 42);
}

#[test]
fn parses_github_url_with_trailing_slash() {
    let pr = parse_pr_ref("https://github.com/frm/schism/pull/42/").unwrap();
    assert_eq!(pr.number, 42);
}

#[test]
fn rejects_non_github_url() {
    assert!(parse_pr_ref("https://gitlab.com/frm/schism/pull/42").is_err());
}

#[test]
fn rejects_github_url_without_pull() {
    assert!(parse_pr_ref("https://github.com/frm/schism/issues/42").is_err());
}


