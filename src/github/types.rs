/// A parsed PR reference: owner/repo#number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrRef {
    pub owner: String,
    pub repo: String,
    pub number: u64,
}

impl PrRef {
    pub fn repo_slug(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrMetadata {
    pub title: String,
    pub url: String,
    pub author: String,
    pub body: String,
    pub head_branch: String,
    pub base_branch: String,
    pub head_ref_oid: String,
    pub base_ref_oid: String,
}

#[derive(Debug, Clone)]
pub struct PrCommit {
    pub sha: String,
    pub message: String,
    pub author: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewEvent {
    Comment,
    Approve,
    RequestChanges,
}

impl ReviewEvent {
    pub fn as_api_value(self) -> &'static str {
        match self {
            ReviewEvent::Comment => "COMMENT",
            ReviewEvent::Approve => "APPROVE",
            ReviewEvent::RequestChanges => "REQUEST_CHANGES",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ReviewEvent::Comment => "comment",
            ReviewEvent::Approve => "approve",
            ReviewEvent::RequestChanges => "request changes",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrReviewContext {
    pub pr: PrRef,
    pub metadata: PrMetadata,
    pub commits: Vec<PrCommit>,
}
