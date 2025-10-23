use crate::platform::run_tray;
use anyhow::Result;
use chrono::{Local, Timelike};
use cron_tab::Cron;
use regex::Regex;
use std::fs::File;
use std::path::Path;
use user_idle::UserIdle;

pub fn validate_interval(interval: u8) -> Result<()> {
    if interval == 0 || interval > 60 {
        anyhow::bail!("intervalは1から60の間で指定してください。指定された値: {interval}");
    }
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

pub fn handle_run(interval: u8, idle_timeout: u64, cli: bool) -> Result<()> {
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

    if cli {
        std::thread::park();
        Ok(())
    } else {
        println!("システムトレイへ常駐します");
        run_tray()
    }
}
