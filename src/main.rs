use backoff::{retry, Error, ExponentialBackoff};
use clap::Parser;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::process::exit;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    branch: Option<String>,
}

fn main() {
    if !is_github_cli_installed() {
        eprintln!("Github CLI is not installed. Please install it.");
        exit(1)
    }

    let args = Args::parse();

    let branch = args.branch.unwrap_or({
        if let Some(branch) = get_current_branch() {
            branch
        } else {
            eprintln!("Error: You are not on any branch. Please checkout to a branch.");
            exit(1)
        }
    });

    let current_commit = get_current_commit(&branch);

    let mut spinner = Spinner::new(Spinners::Dots, "Waiting for GitHub".into());
    retry(ExponentialBackoff::default(), || {
        get_latest_commit(&branch)
            .filter(|latest_commit| &current_commit == latest_commit)
            .ok_or(Error::transient(()))
    })
    .expect("Failed to initialise exponential backoff");
    spinner.stop_and_persist("✔", "GitHub is up to date".into())
}

fn get_current_branch() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get current branch");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let branch = stdout.trim().to_string();
    if branch == "HEAD" {
        None
    } else {
        Some(branch)
    }
}

fn get_current_commit(branch: &str) -> String {
    let output = std::process::Command::new("git")
        .args(["rev-parse", branch])
        .output()
        .expect("Failed to execute git rev-parse");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprint!("{}", stderr);
        exit(1)
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().to_string()
}

fn get_latest_commit(branch: &str) -> Option<String> {
    let output = std::process::Command::new("gh")
        .args(["pr", "view", branch, "--json", "commits"])
        .output()
        .expect("Failed to execute gh pr view command");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprint!("{}", stderr);
        exit(1)
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pull_request: GitHubPullRequest =
        serde_json::from_str(&stdout).expect("Failed to parse PR JSON");
    pull_request
        .commits
        .last()
        .map(|commit| commit.oid.to_owned())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GitHubCommit {
    oid: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GitHubPullRequest {
    commits: Vec<GitHubCommit>,
}

fn is_github_cli_installed() -> bool {
    let output = std::process::Command::new("gh")
        .arg("--version")
        .output()
        .expect("Failed to check if gh cli is installed");
    output.status.success()
}
