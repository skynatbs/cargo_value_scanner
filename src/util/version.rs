use std::fmt;

use reqwest::Client;
use semver::Version;
use serde::Deserialize;
use thiserror::Error;

pub const APP_NAME: &str = "Cargo Value Scanner";
pub const APP_AUTHOR: &str = "SetScallywag";
pub const APP_REPO_URL: &str = "https://github.com/skynatbs/cargo_value_scanner";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_TAG: Option<&str> = option_env!("GIT_TAG");

const GITHUB_OWNER: &str = "skynatbs";
const GITHUB_REPO: &str = "cargo_value_scanner";

#[derive(Clone, Debug)]
pub struct TagVersion {
    pub raw: String,
    pub version: Version,
}

#[derive(Clone, Debug)]
pub struct UpdateInfo {
    pub current: Version,
    pub latest: Option<TagVersion>,
}

impl UpdateInfo {
    pub fn update_available(&self) -> bool {
        self.latest
            .as_ref()
            .map(|candidate| candidate.version > self.current)
            .unwrap_or(false)
    }

    pub fn latest_display(&self) -> Option<&str> {
        self.latest.as_ref().map(|tag| tag.raw.as_str())
    }
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("failed to build HTTP client: {0}")]
    BuildClient(String),
    #[error("request failed: {0}")]
    Request(String),
    #[error("failed to decode response: {0}")]
    Decode(String),
    #[error("invalid version format: {0}")]
    InvalidVersion(String),
}

#[derive(Deserialize)]
struct GitTag {
    name: String,
}

pub async fn check_for_update() -> Result<UpdateInfo, UpdateError> {
    let user_agent = format!("{}/{} (+{})", APP_NAME, version_label(), APP_REPO_URL);
    let client = Client::builder()
        .user_agent(user_agent)
        .build()
        .map_err(|err| UpdateError::BuildClient(err.to_string()))?;

    let current = current_version()?;
    let tags = fetch_tags(&client).await?;
    let mut latest: Option<TagVersion> = None;

    for tag in tags.into_iter().filter_map(parse_tag) {
        if latest
            .as_ref()
            .map(|candidate| tag.version > candidate.version)
            .unwrap_or(true)
        {
            latest = Some(tag);
        }
    }

    Ok(UpdateInfo { current, latest })
}

async fn fetch_tags(client: &Client) -> Result<Vec<GitTag>, UpdateError> {
    let url = format!(
        "https://api.github.com/repos/{owner}/{repo}/tags?per_page=100",
        owner = GITHUB_OWNER,
        repo = GITHUB_REPO
    );

    client
        .get(&url)
        .send()
        .await
        .map_err(|err| UpdateError::Request(err.to_string()))?
        .error_for_status()
        .map_err(|err| UpdateError::Request(err.to_string()))?
        .json::<Vec<GitTag>>()
        .await
        .map_err(|err| UpdateError::Decode(err.to_string()))
}

fn parse_tag(tag: GitTag) -> Option<TagVersion> {
    parse_version_str(&tag.name).ok().map(|version| TagVersion {
        raw: tag.name,
        version,
    })
}

fn parse_version_str(input: &str) -> Result<Version, UpdateError> {
    let trimmed = input.trim_start_matches(|ch| ch == 'v' || ch == 'V');
    Version::parse(trimmed).map_err(|err| UpdateError::InvalidVersion(err.to_string()))
}

pub fn current_version() -> Result<Version, UpdateError> {
    if let Some(tag) = GIT_TAG {
        return parse_version_str(tag);
    }

    parse_version_str(APP_VERSION)
}

pub fn version_label() -> String {
    if let Some(tag) = GIT_TAG {
        tag.to_string()
    } else {
        format!("v{}", APP_VERSION)
    }
}

impl fmt::Display for UpdateInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.latest, self.update_available()) {
            (Some(tag), true) => write!(
                f,
                "New version available: {} (current {})",
                tag.raw, self.current
            ),
            (Some(tag), false) => write!(f, "Up to date on {}", tag.raw),
            (None, _) => write!(f, "No release information found"),
        }
    }
}
