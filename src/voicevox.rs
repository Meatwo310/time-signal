use anyhow::{bail, Context, Result};
use reqwest::Url;
use semver::{Version, VersionReq};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Speaker {
    pub name: String,
    pub styles: Vec<Style>,
}

#[derive(Deserialize)]
pub struct Style {
    pub name: String,
    pub id: u32,
}

fn get_voicevox_version(url: &Url) -> Result<String> {
    let req_url = url.join("version")?;
    let response = reqwest::blocking::get(req_url)
        .context("Failed to fetch VOICEVOX version. Is the server running?")?;

    if !response.status().is_success() {
        bail!("Request to VOICEVOX failed with status: {}", response.status());
    }

    Ok(response.json()?)
}

pub fn check_voicevox_version(url: &Url) -> Result<()> {
    let required_version = VersionReq::parse(">=0.24.0")?;
    let current_version = Version::parse(get_voicevox_version(&url)?.as_str())?;

    if required_version.matches(&current_version) {
        println!("VOICEVOX: {}", current_version);
    } else {
        eprintln!(
            "VOICEVOX {} does not satisfy the required version: {}",
            current_version,
            required_version
        );
    }

    Ok(())
}

pub fn list_speakers(url: &Url) -> Result<Vec<Speaker>> {
    let req_url = url.join("speakers")?;
    let response = reqwest::blocking::get(req_url)
        .context("Failed to fetch speakers from VOICEVOX server.")?;

    if !response.status().is_success() {
        bail!("Request to VOICEVOX failed with status: {}", response.status());
    }

    Ok(serde_json::from_str(response.text()?.as_str())?)
}
