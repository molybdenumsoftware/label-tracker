#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

mod github;
mod types;

use std::{
    borrow::ToOwned,
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    env,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    process,
    str::FromStr,
};

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use clap::{Args, Parser, Subcommand};
use github::Github;
use regex::Regex;
use rss::{Channel, ChannelBuilder, Guid, Item, ItemBuilder};
use serde_json::{from_reader, to_writer};
use tempfile::NamedTempFile;
use types::{DateTime, IssueAction, PullAction, State, STATE_VERSION};

#[derive(Debug)]
struct ChannelPatterns {
    patterns: Vec<(Regex, Vec<String>)>,
}

impl ChannelPatterns {
    fn find_channels(&self, base: &str) -> BTreeSet<String> {
        self.patterns
            .iter()
            .filter_map(|(b, c)| match b.find_at(base, 0) {
                Some(m) if m.end() == base.len() => Some((b, c)),
                _ => None,
            })
            .flat_map(|(b, c)| c.iter().map(|chan| b.replace(base, chan).to_string()))
            .collect()
    }
}

impl FromStr for ChannelPatterns {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let patterns = s
            .split(',')
            .map(|s| match s.trim().split_once(':') {
                Some((base, channels)) => Ok((
                    Regex::new(base)?,
                    channels
                        .split_whitespace()
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>(),
                )),
                None => bail!("invalid channel pattern `{s}`"),
            })
            .collect::<Result<_>>()?;
        Ok(ChannelPatterns { patterns })
    }
}

#[derive(Parser)]
#[clap(version)]
/// Poll github issues and PRs by label and generate RSS feeds.
struct App {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
#[clap(about)]
enum Command {
    /// Initialize a tracker state.
    ///
    /// Each tracker state applies to only one repository and only one label.
    Init {
        /// Path of the newly created state.
        state_file: PathBuf,
        /// Owner of the repository to query.
        owner: String,
        /// Name of the repository.
        repo: String,
        /// Name of the label to track.
        label: String,
    },
    /// Sync issues on a state.
    SyncIssues(SyncIssuesArgs),
    /// Sync pull requests on a state.
    SyncPrs(SyncPrsArgs),
    /// Emit an RSS feed for issue changes.
    EmitIssues(EmitArgs),
    /// Emit an RSS feed for PR changes.
    EmitPrs(EmitArgs),
}

#[derive(Args)]
struct SyncIssuesArgs {
    /// State to sync.
    state_file: PathBuf,
}

#[derive(Args)]
struct SyncPrsArgs {
    /// State to sync.
    state_file: PathBuf,

    /// Path to git repo used for landing detection.
    #[clap(short = 'l', long)]
    local_repo: PathBuf,

    /// PR landing patterns.
    #[clap(short = 'p', long)]
    patterns: ChannelPatterns,
}

#[derive(Args)]
struct EmitArgs {
    /// State to read.
    state_file: PathBuf,

    #[clap(short, long = "max-age")]
    /// How far to look back in history, in hours.
    age_hours: u32,

    #[clap(short, long)]
    /// Target file for the generated feed. Defaults to stdout.
    out: Option<PathBuf>,
}

fn with_state<F>(state_file: impl AsRef<Path>, f: F) -> Result<()>
where
    F: FnOnce(State) -> Result<Option<State>>,
{
    let state_file = state_file.as_ref();
    let old_state: State = from_reader(BufReader::new(File::open(state_file)?))?;
    if old_state.version != STATE_VERSION {
        bail!(
            "expected state version {}, got {}",
            STATE_VERSION,
            old_state.version
        );
    }

    let new_state = f(old_state)?;

    if let Some(state) = new_state {
        let new_state_file = NamedTempFile::new_in(
            state_file
                .ancestors()
                .nth(1)
                .unwrap_or_else(|| Path::new(".")),
        )?;

        to_writer(BufWriter::new(&new_state_file), &state)?;
        new_state_file.persist(state_file)?;
    }

    Ok(())
}

fn with_state_and_github<F>(state_file: impl AsRef<Path>, f: F) -> Result<()>
where
    F: FnOnce(State, &Github) -> Result<Option<State>>,
{
    let github_api_token =
        env::var("GITHUB_API_TOKEN").context("failed to load GITHUB_API_TOKEN")?;

    with_state(state_file, |old_state| {
        let client = github::Github::new(
            &github_api_token,
            &old_state.owner,
            &old_state.repo,
            &old_state.label,
        )?;

        f(old_state, &client)
    })
}

fn sync_issues(mut state: State, github: &github::Github) -> Result<Option<State>> {
    let issues = github.query_issues(state.issues_updated)?;

    let mut new_history = vec![];

    for updated in issues {
        let issue_state = |is_new| match (updated.is_open, is_new) {
            (true, _) => IssueAction::New,
            (false, true) => IssueAction::NewClosed,
            (false, false) => IssueAction::Closed,
        };
        match state.issues.entry(updated.id.clone()) {
            Entry::Occupied(mut e) => {
                let stored = e.get_mut();
                if stored.is_open != updated.is_open {
                    new_history.push((updated.last_update, updated.id.clone(), issue_state(false)));
                }
                *stored = updated;
            }
            Entry::Vacant(e) => {
                new_history.push((updated.last_update, updated.id.clone(), issue_state(true)));
                e.insert(updated);
            }
        }
    }

    new_history.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    if let Some(&(at, _, _)) = new_history.last() {
        state.issues_updated = Some(at);
    }
    state.issue_history.append(&mut new_history);

    Ok(Some(state))
}

fn sync_prs(
    mut state: State,
    github: &github::Github,
    local_repo: impl AsRef<Path>,
    channel_patterns: &ChannelPatterns,
) -> Result<Option<State>> {
    let local_repo = local_repo.as_ref();
    let prs = github.query_pulls(state.pull_requests_updated)?;

    let mut new_history = vec![];

    for updated in prs {
        let pr_state = |is_new| match (updated.is_open, updated.is_merged, is_new) {
            (false, false, true) => PullAction::NewClosed,
            (false, false, false) => PullAction::Closed,
            (true, false, _) => PullAction::New,
            (_, true, true) => PullAction::NewMerged,
            (_, true, false) => PullAction::Merged,
        };
        match state.pull_requests.entry(updated.id.clone()) {
            Entry::Occupied(mut e) => {
                let stored = e.get_mut();
                if (stored.is_open, stored.is_merged) != (updated.is_open, updated.is_merged) {
                    new_history.push((updated.last_update, updated.id.clone(), pr_state(false)));
                }
                stored.update(updated);
            }
            Entry::Vacant(e) => {
                new_history.push((updated.last_update, updated.id.clone(), pr_state(true)));
                e.insert(updated);
            }
        }
    }

    let mut git_cmd = process::Command::new("git");
    let kind = if local_repo.exists() {
        git_cmd.arg("-C").arg(local_repo).args([
            "fetch",
            "--force",
            "--prune",
            "origin",
            "refs/heads/*:refs/heads/*",
        ]);
        "fetch"
    } else {
        let url = format!("https://github.com/{}/{}", &state.owner, &state.repo);
        git_cmd
            .arg("clone")
            .args([&url, "--filter", "tree:0", "--bare"])
            .arg(local_repo);
        "clone"
    };

    let git_status = git_cmd.spawn()?.wait()?;
    if !git_status.success() {
        bail!("{kind} failed: {git_status}");
    }

    let branches = state
        .pull_requests
        .values()
        .map(|pr| pr.base_ref.clone())
        .collect::<BTreeSet<_>>();
    let patterns = branches
        .iter()
        .map(|b| (b.as_str(), channel_patterns.find_channels(b)))
        .filter(|(_, cs)| !cs.is_empty())
        .collect::<BTreeMap<_, _>>();

    for (id, pr) in &mut state.pull_requests {
        let Some(merge) = pr.merge_commit.as_ref() else {
            continue;
        };
        let chans = match patterns.get(pr.base_ref.as_str()) {
            Some(chans) if chans != &pr.landed_in => chans,
            _ => continue,
        };
        let landed = process::Command::new("git")
            .arg("-C")
            .arg(local_repo)
            .args(["branch", "--contains", merge, "--list"])
            .args(chans)
            .output()?;
        let landed = if landed.status.success() {
            std::str::from_utf8(&landed.stdout)?
                .split_whitespace()
                .filter(|&b| !pr.landed_in.contains(b))
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        } else {
            bail!(
                "failed to check landing status of {}: {}, {}",
                id,
                landed.status,
                String::from_utf8_lossy(&landed.stderr)
            );
        };
        if landed.is_empty() {
            continue;
        }
        pr.landed_in.extend(landed.iter().cloned());
        new_history.push((Utc::now(), id.clone(), PullAction::Landed(landed)));
    }

    new_history.sort_by(|a, b| (a.0, &a.1, &a.2).cmp(&(b.0, &b.1, &b.2)));
    if let Some(&(at, _, _)) = new_history.last() {
        state.pull_requests_updated = Some(at);
    }
    state.pull_history.append(&mut new_history);

    Ok(Some(state))
}

fn format_history<V, A: Clone, F: Fn(&V, DateTime, &A) -> Item>(
    items: &BTreeMap<String, V>,
    history: &[(DateTime, String, A)],
    age_hours: u32,
    format_entry: F,
    // backwards compat of GUIDs requires this. we need either a different ID format
    // or an id suffix to give landing events unique ids in all cases, and the suffix
    // is easier for now
    id_suffix: impl Fn(&A) -> String,
) -> Result<Vec<Item>> {
    let since = Utc::now() - Duration::hours(age_hours.into());

    history
        .iter()
        .rev()
        .take_while(|(changed, _, _)| changed >= &since)
        .map(|(changed, id, how)| {
            let Some(entry) = items.get(id.as_str()) else {
                bail!("database is corrupted (dangling key {})", id)
            };
            Ok(Item {
                guid: Some(Guid {
                    value: format!("{}/{}{}", changed.to_rfc3339(), id, id_suffix(how)),
                    permalink: false,
                }),
                ..format_entry(entry, *changed, how)
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn new_rss_item(tag: &str, title: &str, url: &str, changed: DateTime, body: &str) -> Item {
    ItemBuilder::default()
        .title(Some(format!("{tag} {title}")))
        .link(Some(url.to_string()))
        .pub_date(Some(changed.to_rfc2822()))
        .content(Some(body.to_string()))
        .build()
}

fn emit_issues(state: &State, age_hours: u32) -> Result<Channel> {
    let entries = format_history(
        &state.issues,
        &state.issue_history,
        age_hours,
        |issue, changed, how| {
            let tag = match how {
                IssueAction::New => "[NEW]",
                IssueAction::Closed => "[CLOSED]",
                IssueAction::NewClosed => "[NEW][CLOSED]",
            };
            new_rss_item(tag, &issue.title, &issue.url, changed, &issue.body)
        },
        |_| String::default(),
    )?;

    let channel = ChannelBuilder::default()
        .title(format!(
            "Issues labeled `{}' in {}/{}",
            state.label, state.owner, state.repo
        ))
        .items(entries)
        .build();

    Ok(channel)
}

fn emit_prs(state: &State, age_hours: u32) -> Result<Channel> {
    let entries = format_history(
        &state.pull_requests,
        &state.pull_history,
        age_hours,
        |pr, changed, how| {
            let (tag, refs) = match how {
                PullAction::New => ("[NEW]", None),
                PullAction::NewMerged => ("[NEW][MERGED]", None),
                PullAction::Closed => ("[CLOSED]", None),
                PullAction::NewClosed => ("[NEW][CLOSED]", None),
                PullAction::Merged => ("[MERGED]", None),
                PullAction::Landed(l) => ("[LANDED]", Some(l.join(" "))),
            };
            let info = format!("{}({})", tag, refs.as_ref().unwrap_or(&pr.base_ref));
            new_rss_item(&info, &pr.title, &pr.url, changed, &pr.body)
        },
        |how| match how {
            PullAction::Landed(chans) => format!("/landed/{}", chans.join("/")),
            _ => String::default(),
        },
    )?;

    let channel = ChannelBuilder::default()
        .title(format!(
            "Pull requests labeled `{}' in {}/{}",
            state.label, state.owner, state.repo
        ))
        .items(entries)
        .build();

    Ok(channel)
}

fn write_feed(to: Option<PathBuf>, channel: &Channel) -> Result<Option<State>> {
    match to {
        Some(to) => {
            let new_file =
                NamedTempFile::new_in(to.ancestors().nth(1).unwrap_or_else(|| Path::new(".")))?;

            channel.write_to(BufWriter::new(&new_file))?;
            new_file.persist(to)?;
        }
        None => println!("{}", channel.to_string()),
    }
    Ok(None)
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    match App::parse().command {
        Command::Init {
            state_file,
            owner,
            repo,
            label,
        } => {
            let state = State {
                version: STATE_VERSION,
                owner,
                repo,
                label,
                ..State::default()
            };

            let file = File::options()
                .create_new(true)
                .write(true)
                .open(state_file)?;
            to_writer(file, &state)?;
        }
        Command::SyncIssues(cmd) => {
            with_state_and_github(cmd.state_file, sync_issues)?;
        }
        Command::SyncPrs(cmd) => {
            with_state_and_github(&cmd.state_file, |s, g| {
                sync_prs(s, g, cmd.local_repo, &cmd.patterns)
            })?;
        }
        Command::EmitIssues(cmd) => {
            with_state(cmd.state_file, |s| {
                write_feed(cmd.out, &emit_issues(&s, cmd.age_hours)?)
            })?;
        }
        Command::EmitPrs(cmd) => {
            with_state(cmd.state_file, |s| {
                write_feed(cmd.out, &emit_prs(&s, cmd.age_hours)?)
            })?;
        }
    };

    Ok(())
}
