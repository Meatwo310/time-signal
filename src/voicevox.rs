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

// #[derive(Deserialize)]
// pub struct AudioQuery {
//     pub accent_phrases: Vec<AccentPhrase>,
//     pub speed_scale: f64,
//     pub pitch_scale: f64,
//     pub intonation_scale: f64,
//     pub volume_scale: f64,
//     pub pre_phoneme_length: f64,
//     pub post_phoneme_length: f64,
//     pub output_sampling_rate: u32,
//     pub output_stereo: bool,
//     pub kana: String,
// }
//
// #[derive(Deserialize)]
// pub struct AccentPhrase {
//     pub moras: Vec<Mora>,
//     pub accent: u32,
//     pub pause_mora: Option<Mora>,
//     pub is_interrogative: bool,
// }
//
// #[derive(Deserialize)]
// pub struct Mora {
//     pub text: String,
//     pub consonant: Option<String>,
//     pub consonant_length: Option<f64>,
//     pub vowel: String,
//     pub vowel_length: f64,
//     pub pitch: f64,
// }

pub struct VoicevoxClient {
    base_url: Url,
    client: reqwest::blocking::Client,
}

impl VoicevoxClient {
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let req_url = self.base_url.join(endpoint)?;
        let response = self
            .client
            .get(req_url)
            .send()
            .with_context(|| format!("Failed to request VOICEVOX endpoint: {}", endpoint))?;

        if !response.status().is_success() {
            bail!("Request to VOICEVOX failed with status: {}", response.status());
        }

        Ok(response.json()?)
    }

    fn post(&self, endpoint: &str) -> Result<()> {
        let req_url = self.base_url.join(endpoint)?;
        let response = self
            .client
            .post(req_url)
            .send()
            .with_context(|| format!("Failed to request VOICEVOX endpoint: {}", endpoint))?;

        if !response.status().is_success() {
            bail!("Request to VOICEVOX failed with status: {}", response.status());
        }

        Ok(())
    }

    pub fn get_version(&self) -> Result<String> {
        self.get("version")
    }

    pub fn check_version(&self) -> Result<()> {
        let required_version = VersionReq::parse(">=0.24.0")?;
        let current_version = Version::parse(self.get_version()?.as_str())?;

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

    pub fn list_speakers(&self) -> Result<Vec<Speaker>> {
        self.get("speakers")
    }

    pub fn is_initialized_speaker(&self, speaker_id: u32) -> Result<bool> {
        let endpoint = format!("is_initialized_speaker?speaker={}", speaker_id);
        self.get(&endpoint)
    }

    pub fn initialize_speaker(&self, speaker_id: u32) -> Result<()> {
        let endpoint = format!("initialize_speaker?speaker={}&skip_reinit=true", speaker_id);
        self.post(&endpoint)
    }

    pub fn audio_query(&self, text: &str, speaker_id: u32) -> Result<String> {
        let mut req_url = self.base_url.join("audio_query")?;

        {
            let mut query_pairs = req_url.query_pairs_mut();
            query_pairs.append_pair("text", text);
            query_pairs.append_pair("speaker", &speaker_id.to_string());
        }

        let response = self
            .client
            .post(req_url)
            .send()
            .with_context(|| "Failed to request VOICEVOX audio_query endpoint")?;

        if !response.status().is_success() {
            bail!("audio_query request failed with status: {}", response.status());
        }

        let res = response.text()?;
        Ok(res)
    }

    pub fn multi_synthesis(&self, queries: &[String], speaker_id: u32) -> Result<Vec<u8>> {
        let mut req_url = self.base_url.join("multi_synthesis")?;

        {
            let mut query_pairs = req_url.query_pairs_mut();
            query_pairs.append_pair("speaker", &speaker_id.to_string());
        }

        // クエリ配列をJSON配列に変換
        let queries_json: Vec<serde_json::Value> = queries
            .iter()
            .map(|q| serde_json::from_str(q))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse audio queries as JSON")?;

        let response = self
            .client
            .post(req_url)
            .header("Content-Type", "application/json")
            .json(&queries_json)
            .send()
            .with_context(|| "Failed to request VOICEVOX multi_synthesis endpoint")?;

        if !response.status().is_success() {
            bail!("multi_synthesis request failed with status: {}", response.status());
        }

        Ok(response.bytes()?.to_vec())
    }
}
