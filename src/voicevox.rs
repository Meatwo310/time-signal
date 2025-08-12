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

    fn send_request(&self, request: reqwest::blocking::RequestBuilder, endpoint: &str) -> Result<reqwest::blocking::Response> {
        let response = request
            .send()
            .with_context(|| format!("VOICEVOXエンドポイントへのリクエストに失敗しました: {endpoint}"))?;

        if !response.status().is_success() {
            bail!("VOICEVOXへのリクエストがステータス {} で失敗しました", response.status());
        }

        Ok(response)
    }

    fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let req_url = self.base_url.join(endpoint)?;
        let request = self.client.get(req_url);
        let response = self.send_request(request, endpoint)?;
        Ok(response.json()?)
    }

    fn post(&self, endpoint: &str) -> Result<()> {
        let req_url = self.base_url.join(endpoint)?;
        let request = self.client.post(req_url);
        self.send_request(request, endpoint)?;
        Ok(())
    }

    pub fn get_version(&self) -> Result<String> {
        self.get("version")
    }

    pub fn check_version(&self) -> Result<()> {
        let required = VersionReq::parse(">=0.24.0")?;
        let current = Version::parse(self.get_version()?.as_str())?;

        if required.matches(&current) {
            println!("VOICEVOX: {current}");
        } else {
            println!(
                "警告: VOICEVOX {current} は必要なバージョン {required} を満たしていません",
            );
        }

        Ok(())
    }

    pub fn list_speakers(&self) -> Result<Vec<Speaker>> {
        self.get("speakers")
    }

    pub fn is_initialized_speaker(&self, speaker_id: u32) -> Result<bool> {
        let endpoint = format!("is_initialized_speaker?speaker={speaker_id}");
        self.get(&endpoint)
    }

    pub fn initialize_speaker(&self, speaker_id: u32) -> Result<()> {
        let endpoint = format!("initialize_speaker?speaker={speaker_id}&skip_reinit=true");
        self.post(&endpoint)
    }

    pub fn audio_query(&self, text: &str, speaker_id: u32) -> Result<String> {
        let mut req_url = self.base_url.join("audio_query")?;

        {
            let mut query_pairs = req_url.query_pairs_mut();
            query_pairs.append_pair("text", text);
            query_pairs.append_pair("speaker", &speaker_id.to_string());
        }

        let request = self.client.post(req_url);
        let response = self.send_request(request, "audio_query")?;
        Ok(response.text()?)
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
            .context("音声クエリを解析できませんでした")?;

        let request = self.client
            .post(req_url)
            .header("Content-Type", "application/json")
            .json(&queries_json);

        let response = self.send_request(request, "multi_synthesis")?;
        Ok(response.bytes()?.to_vec())
    }
}
