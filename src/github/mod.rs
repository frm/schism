pub mod fetch;
pub mod gh;
pub mod parse;
pub mod review;
pub mod types;

// Re-export commonly used items so callers don't need to know the internal structure
pub use types::{PrCommit, PrReviewContext, ReviewEvent};
pub use parse::parse_pr_ref;
pub use gh::check_installed;
pub use fetch::{fetch_diff, fetch_metadata, fetch_commits, fetch_commit_diff, fetch_file_content};
pub use review::{collect_review_comments, build_review_payload, submit_review};
