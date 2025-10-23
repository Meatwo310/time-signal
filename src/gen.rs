use crate::voicevox::VoicevoxClient;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::io::{stdout, Cursor, Read, Write};
use url::Url;
use zip::ZipArchive;

pub fn validate_interval(interval: u8) -> Result<()> {
    if interval == 0 || interval > 60 {
        anyhow::bail!("intervalは1から60の間で指定してください。指定された値: {interval}");
    }
    Ok(())
}

pub fn handle_gen(speaker_id: Option<u32>, url: String, interval: u8) -> Result<()> {
    validate_interval(interval)?;

    let client = VoicevoxClient::new(Url::parse(&url).context("VOICEVOXサーバーのURLが不正です")?);

    let required = VersionReq::parse(">=0.24.0")?;
    let current = Version::parse(&client.get_version()?)?;

    if required.matches(&current) {
        println!("VOICEVOX: {current}");
    } else {
        println!("警告: VOICEVOX {current} は必要なバージョン {required} を満たしていません",);
    }

    let speakers = client.list_speakers()?;

    if speaker_id.is_none() {
        println!("\nスタイル一覧:");
        for speaker in speakers {
            println!("\n{}:", speaker.name);
            for style in speaker.styles {
                println!("{:4}. {}", style.id, style.name);
            }
        }
        return Ok(());
    }

    let speaker_id = speaker_id.context("ここでスタイルIDが提供されるべきです")?;
    let (speaker, style) = client.find_speaker_and_style(speaker_id, &speakers)?;

    println!("{}. {} ({})", style.id, speaker.name, style.name,);

    if !client.is_initialized_speaker(speaker_id)? {
        print!("スタイルを初期化中... ");
        stdout().flush()?;
        client.initialize_speaker(speaker_id)?;
        println!("完了");
    }

    generate_voice_files(&client, speaker_id, interval)?;

    Ok(())
}

fn create_progress_bar(length: u64, message: &str) -> Result<ProgressBar> {
    let bar = ProgressBar::new(length)
        .with_style(
            ProgressStyle::with_template(&format!(
                "{} [{{bar:24}}] {{pos:>2}}/{{len:>2}}",
                message
            ))?
            .progress_chars("#.."),
        )
        .with_finish(ProgressFinish::AndLeave);
    bar.force_draw();
    Ok(bar)
}

fn generate_voice_files(client: &VoicevoxClient, speaker_id: u32, interval: u8) -> Result<()> {
    let mut queries: HashMap<u32, HashMap<u32, String>> = HashMap::new();

    let minutes_per_hour: Vec<u32> = (0..60).step_by(interval as usize).collect(); // [0, 15, 30, 45]
    let total_queries = (24 * minutes_per_hour.len()) as u64;

    let bar = create_progress_bar(total_queries, "クエリを生成")?;

    for hour in 0..24 {
        let mut minute_queries: HashMap<u32, String> = HashMap::new();
        for &minute in &minutes_per_hour {
            let hour_text = if hour == 0 { "零" } else { &hour.to_string() };
            let text = if minute == 0 {
                format!("{hour_text}時です")
            } else {
                format!("{hour_text}時{minute}分です")
            };
            let query = client.audio_query(&text, speaker_id)?;
            minute_queries.insert(minute, query);
            bar.inc(1);
        }
        queries.insert(hour, minute_queries);
    }

    bar.finish();

    std::fs::create_dir_all("voice_files")?;

    let bar = create_progress_bar(total_queries, "ボイスを生成")?;

    for hour in 0..24 {
        let hour_queries = queries.get(&hour).unwrap();
        let query_vec: Vec<String> = minutes_per_hour
            .iter()
            .map(|minute| hour_queries.get(minute).unwrap().clone())
            .collect();
        let zip_data = client.multi_synthesis(&query_vec, speaker_id)?;

        let cursor = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(cursor)?;

        for (i, &minute) in minutes_per_hour.iter().enumerate() {
            if i < archive.len() {
                let mut file = archive.by_index(i)?;
                if file.is_file() {
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;
                    let output_path = format!("voice_files/{:02}-{:02}.wav", hour, minute);
                    std::fs::write(&output_path, buffer)?;
                }
            }
        }
        bar.inc(minutes_per_hour.len() as u64);
    }

    bar.finish();
    println!("すべての音声ファイルが正常に生成・保存されました！");

    Ok(())
}
