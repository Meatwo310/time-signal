use anyhow::bail;
use reqwest::Url;
use semver::{Version, VersionReq};

fn get_voicevox_version(url: &Url) -> anyhow::Result<String> {
    let response = reqwest::blocking::get(url.join("version")?)?;

    if !response.status().is_success() {
        bail!("Request to VOICEVOX failed with status: {}", response.status());
    }

    Ok(response.json()?)
}

pub fn check_voicevox_version(url: &Url) -> anyhow::Result<()> {
    let required_version = VersionReq::parse(">=0.24.0")?;
    let current_version = Version::parse(&get_voicevox_version(&url)?)?;
    if !required_version.matches(&current_version) {
        bail!("VOICEVOX version {} does not satisfy the required version {}", current_version, required_version);
    }
    println!("VOICEVOX: {}", current_version);
    Ok(())
}
