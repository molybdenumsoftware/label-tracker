#![allow(non_camel_case_types)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub type DateTime = chrono::DateTime<chrono::Utc>;
#[allow(clippy::upper_case_acronyms)]
pub type HTML = String;
#[allow(clippy::upper_case_acronyms)]
pub type URI = String;

pub const STATE_VERSION: u32 = 1;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    pub owner: String,
    pub repo: String,
    pub label: String,
    pub issues_updated: Option<DateTime>,
    pub issues: BTreeMap<String, Issue>,
    pub issue_history: Vec<(DateTime, String, IssueAction)>,
    pub pull_requests_updated: Option<DateTime>,
    pub pull_requests: BTreeMap<String, PullRequest>,
    pub pull_history: Vec<(DateTime, String, PullAction)>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Issue {
    #[serde(skip)]
    pub id: String,
    pub title: String,
    pub is_open: bool,
    pub body: String,
    pub last_update: DateTime,
    pub url: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum IssueAction {
    New,
    Closed,
    NewClosed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PullRequest {
    #[serde(skip)]
    pub id: String,
    pub title: String,
    pub is_open: bool,
    pub is_merged: bool,
    pub body: String,
    pub last_update: DateTime,
    pub url: String,
    pub base_ref: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PullAction {
    New,
    Closed,
    NewClosed,
    Merged,
    NewMerged,
}
