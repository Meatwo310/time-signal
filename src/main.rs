mod voicevox;

use crate::voicevox::VoicevoxClient;
use anyhow::{Context, Result};
use chrono::{Local, Timelike};
use clap::{Parser, Subcommand};
use cron_tab::Cron;
use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressStyle};
use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::thread::sleep;
use std::time::Duration;
use url::Url;
use zip::ZipArchive;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand)]
enum Commands {
    Gen {
        /// 音声生成に使用するスタイルID。
        /// 空の場合はすべてのスタイルを一覧表示します
        speaker_id: Option<u32>,

        /// VOICEVOXサーバーのURL
        #[arg(short, long, default_value = "http://127.0.0.1:50021/")]
        url: String,
    },
    Run {
        #[arg(short, long, default_value = "0 */15 * * * *")]
        cron_spec: String,
    }
}

fn handle_gen(speaker_id: Option<u32>, url: String) -> Result<()> {
    let client = VoicevoxClient::new(Url::parse(&url)
        .context("VOICEVOXサーバーのURLが不正です")?
    );

    let required = VersionReq::parse(">=0.24.0")?;
    let current = Version::parse(&client.get_version()?)?;

    if required.matches(&current) {
        println!("VOICEVOX: {current}");
    } else {
        println!(
            "警告: VOICEVOX {current} は必要なバージョン {required} を満たしていません",
        );
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
    let speaker_and_style = speakers.iter()
        .find_map(|speaker| speaker.styles.iter()
            .find(|style| style.id == speaker_id)
            .map(|style| (&speaker.name, &style.name))
        ).with_context(|| format!("スタイルID {speaker_id} が見つかりません"))?;

    println!(
        "{}. {} ({})",
        speaker_id,
        speaker_and_style.0,
        speaker_and_style.1,
    );

    if !client.is_initialized_speaker(speaker_id)? {
        println!("スタイルを初期化中...");
        client.initialize_speaker(speaker_id)?;
        println!("スタイルの初期化が完了しました！");
    }

    generate_voice_files(&client, speaker_id)?;

    Ok(())
}

fn create_progress_bar(length: u64, message: &str) -> Result<ProgressBar> {
    let bar = ProgressBar::new(length)
        .with_style(
            ProgressStyle::with_template(&format!("{} [{{bar:24}}] {{pos:>2}}/{{len:>2}}", message))?
                .progress_chars("#..")
        )
        .with_finish(ProgressFinish::AndLeave);
    bar.force_draw();
    Ok(bar)
}

fn generate_voice_files(client: &VoicevoxClient, speaker_id: u32) -> Result<()> {
    let mut queries: HashMap<u32, HashMap<u32, String>> = HashMap::new();

    let bar = create_progress_bar(24 * 4, "クエリを生成")?;

    for hour in 0..24 {
        let mut minute_queries: HashMap<u32, String> = HashMap::new();
        for minute in [0, 15, 30, 45] {
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

    let bar = create_progress_bar(24 * 4, "ボイスを生成")?;

    for hour in 0..24 {
        let hour_queries = queries.get(&hour).unwrap();
        let query_vec: Vec<String> = [0, 15, 30, 45]
            .iter()
            .map(|minute| hour_queries.get(minute).unwrap().clone())
            .collect();
        let zip_data = client.multi_synthesis(&query_vec, speaker_id)?;

        let cursor = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(cursor)?;

        let minutes = [0, 15, 30, 45];
        for (i, &minute) in minutes.iter().enumerate() {
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
        bar.inc(4);
    }

    bar.finish();
    println!("すべての音声ファイルが正常に生成・保存されました！");

    Ok(())
}

fn handle_run(cron_spec: &str) -> Result<()> {
    let mut cron = Cron::new(Local);
    // https://github.com/tuyentv96/rust-crontab?tab=readme-ov-file#-cron-expression-format
    // ┌───────────── 秒 (0 - 59)
    // │ ┌─────────── 分 (0 - 59)
    // │ │ ┌───────── 時 (0 - 23)
    // │ │ │ ┌─────── 日 (1 - 31)
    // │ │ │ │ ┌───── 月 (1 - 12)
    // │ │ │ │ │ ┌─── 曜日 (0 - 6) (日曜日から土曜日)
    // │ │ │ │ │ │ ┌─ 年 (1970 - 3000)
    // │ │ │ │ │ │ │
    // * * * * * * *
    cron.add_fn(cron_spec, || {
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute() / 15 * 15;

        let filename = format!("voice_files/{:02}-{:02}.wav", hour, minute);
        let file = File::open(&filename)
            .expect(&format!("ファイル {filename} を開けませんでした"));

        let mut handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
        handle.log_on_drop(false);
        let sink = rodio::play(handle.mixer(), file).unwrap();
        sink.sleep_until_end();
    })?;

    cron.start();
    println!("cronスケジューラを開始しました！");
    loop {
        sleep(Duration::from_secs(3600));
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command.unwrap_or(Commands::Run {cron_spec: "0 */15 * * * *".to_string()}) {
        Commands::Gen { speaker_id, url } => handle_gen(speaker_id, url)?,
        Commands::Run { cron_spec } => handle_run(&cron_spec)?,
    }
    Ok(())
}
