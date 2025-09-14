mod voicevox;

use crate::voicevox::VoicevoxClient;
use anyhow::{Context, Result};
use chrono::{Local, Timelike};
use clap::{Parser, Subcommand};
use cron_tab::Cron;
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use regex::Regex;
use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::fs::File;
use std::io::{stdout, Cursor, Read, Write};
use std::path::Path;
use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};
use url::Url;
use user_idle::UserIdle;
use zip::ZipArchive;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand)]
enum Commands {
    /// 音声ファイルを事前生成します。VOICEVOXサーバーが必要です。
    Gen {
        /// 音声生成に使用するスタイルID。
        /// 空の場合はすべてのスタイルを一覧表示します
        speaker_id: Option<u32>,

        /// VOICEVOXサーバーのURL
        #[arg(short, long, default_value = "http://127.0.0.1:50021/")]
        url: String,

        /// 時報の間隔。15分を指定すると、毎時0分、15分、30分、45分に音声が生成されます。
        #[arg(short, long, default_value = "15")]
        interval: u8,
    },
    /// 一定間隔で時報を再生します。音声ファイルを事前に生成する必要があります。
    Run {
        /// 時報の間隔。15分を指定すると、毎時0分、15分、30分、45分に音声が再生されます。
        #[arg(short, long, default_value = "15")]
        interval: u8,

        /// 指定分以上操作がない場合、時報をスキップします。
        #[arg(short='t', long, default_value = "10")]
        idle_timeout: u64,
    }
}

enum Message {
    Quit,
}

fn validate_interval(interval: u8) -> Result<()> {
    if interval == 0 || interval > 60 {
        anyhow::bail!("intervalは1から60の間で指定してください。指定された値: {interval}");
    }
    Ok(())
}

fn handle_gen(speaker_id: Option<u32>, url: String, interval: u8) -> Result<()> {
    validate_interval(interval)?;

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
    let (speaker, style) = client.find_speaker_and_style(speaker_id, &speakers)?;

    println!(
        "{}. {} ({})",
        style.id,
        speaker.name,
        style.name,
    );

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
            ProgressStyle::with_template(&format!("{} [{{bar:24}}] {{pos:>2}}/{{len:>2}}", message))?
                .progress_chars("#..")
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

fn check_voice_files(interval: u8) -> Result<()> {
    let voice_files = Path::new("voice_files");
    let pattern = Regex::new(r"^([01]\d|2[0-3])-([0-5]\d)\.wav$")?;

    let expected_file_count = 24 * ((59 / interval + 1) as usize);
    let file_count = match voice_files.read_dir() {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().to_str().map(String::from))
            .filter(|name| pattern.is_match(name))
            .count(),
        Err(_) => 0,
    };

    if !voice_files.exists() || !voice_files.is_dir() || file_count == 0 {
        println!("警告: 音声ファイルが存在しません。genコマンドを実行して音声ファイルを生成してください。");
    } else if file_count < expected_file_count {
        println!("警告: 音声ファイルが不足しています。genコマンドを実行してすべての音声ファイルを生成してください。");
    }

    Ok(())
}

fn get_idle_minutes() -> u64 {
    UserIdle::get_time().map(|u| u.as_minutes()).unwrap_or(0)
}

#[cfg(target_os = "windows")]
fn get_icon_source() -> Result<IconSource> {
    Ok(IconSource::Resource("tray-default"))
}

/// [`tray_item::IconSource`]側のcfg属性による制約のため、`unix`ではなく`macos`と`linux`を指定
#[cfg(any(target_os = "macos", all(target_os = "linux", feature = "ksni")))]
fn get_icon_source() -> Result<IconSource> {
    let cursor = Cursor::new(include_bytes!("../icons/time-signal.png"));
    let decoder = png::Decoder::new(cursor);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf)?;
    let bytes = &buf[..info.buffer_size()];

    Ok(IconSource::Data {
        data: bytes.to_vec(),
        height: 256,
        width: 256,
    })
}


fn handle_run(interval: u8, idle_timeout: u64) -> Result<()> {
    validate_interval(interval)?;
    check_voice_files(interval)?;

    let mut cron = Cron::new(Local);
    let cron_spec = format!("0 */{interval} * * * *");

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
    cron.add_fn(&cron_spec, move || {
        let now = Local::now();
        let hour = now.hour();
        let minute = (now.minute() as u8) / interval * interval;

        if idle_timeout == 0 || get_idle_minutes() < idle_timeout {
            println!("{:02}:{:02}です", hour, minute);
        } else {
            println!("{:02}:{:02} - スキップ", hour, minute);
            return;
        }

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

    let mut tray = TrayItem::new(
        "Time Signal",
        IconSource::Resource("tray-default")
    ).context("トレイアイコンの作成に失敗しました")?;

    tray.add_label("Time Signal is running.")?;

    tray.inner_mut().add_separator()?;

    let (tx, rx) = mpsc::sync_channel(1);

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })?;

    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                println!("終了しています...");
                break;
            }
            Err(e) => {
                println!("エラー: {e}");
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command.unwrap_or(Commands::Run {interval: 15, idle_timeout: 10}) {
        Commands::Gen { speaker_id, url, interval } => handle_gen(speaker_id, url, interval)?,
        Commands::Run { interval, idle_timeout } => handle_run(interval, idle_timeout)?,
    }
    Ok(())
}
