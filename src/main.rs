mod voicevox;

use crate::voicevox::VoicevoxClient;
use anyhow::{Context, Result};
use chrono::{Local, Timelike};
use clap::{Parser, Subcommand};
use cron_tab::Cron;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read};
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
        /// The speaker ID to use for the voice generation.
        /// Leave empty to list all speakers.
        speaker_id: Option<u32>,

        /// The URL of the VOICEVOX server.
        #[arg(short, long, default_value = "http://127.0.0.1:50021/")]
        url: String,
    },
    Run {
    }
}

fn handle_gen(speaker_id: Option<u32>, url: String) -> Result<()> {
    let client = VoicevoxClient::new(Url::parse(&url)
        .context("Invalid URL provided for VOICEVOX server")?
    );

    client.check_version()?;

    let speakers = client.list_speakers()?;

    if speaker_id.is_none() {
        println!("\nList of speakers:");
        for speaker in speakers {
            println!("\n{}:", speaker.name);
            for style in speaker.styles {
                println!("{:4}. {}", style.id, style.name);
            }
        }
        return Ok(());
    }

    let speaker_id = speaker_id.context("Speaker ID should be provided here")?;
    let speaker_and_style = speakers.iter()
        .find_map(|speaker| speaker.styles.iter()
            .find(|style| style.id == speaker_id)
            .map(|style| (speaker.name.as_str(), style.name.as_str()))
        ).with_context(|| format!("Speaker ID {} not found", speaker_id))?;

    println!(
        "{}. {} ({})",
        speaker_id,
        speaker_and_style.0,
        speaker_and_style.1,
    );

    if client.is_initialized_speaker(speaker_id)? {
        println!("Speaker is ready!");
    } else {
        println!("Initializing speaker...");
        client.initialize_speaker(speaker_id)?;
        println!("Speaker initialized successfully!");
    }

    generate_voice_files(&client, speaker_id)?;

    Ok(())
}

fn generate_voice_files(client: &VoicevoxClient, speaker_id: u32) -> Result<()> {
    println!("Generating queries...");

    let mut queries: HashMap<u32, HashMap<u32, String>> = HashMap::new();

    for hour in 0..24 {
        let mut minute_queries: HashMap<u32, String> = HashMap::new();
        for minute in [0, 15, 30, 45] {
            let hour_text = if hour == 0 { "零" } else { &hour.to_string() };
            let text = if minute == 0 {
                format!("{}時です", hour_text)
            } else {
                format!("{}時{}分です", hour_text, minute)
            };
            let query = client.audio_query(&text, speaker_id)?;
            minute_queries.insert(minute, query);
        }
        queries.insert(hour, minute_queries);
    }

    println!("Generated {} queries in total", queries.len() * 4);

    std::fs::create_dir_all("voice_files")?;

    for hour in 0..24 {
        println!("Generating audio files for {}時...", hour);

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
    }

    println!("All voice files generated and saved successfully!");

    Ok(())
}

fn handle_run() -> Result<()> {
    let mut cron = Cron::new(Local);
    // https://github.com/tuyentv96/rust-crontab?tab=readme-ov-file#-cron-expression-format
    // ┌───────────── second (0 - 59)
    // │ ┌─────────── minute (0 - 59)
    // │ │ ┌───────── hour (0 - 23)
    // │ │ │ ┌─────── day of month (1 - 31)
    // │ │ │ │ ┌───── month (1 - 12)
    // │ │ │ │ │ ┌─── day of week (0 - 6) (Sunday to Saturday)
    // │ │ │ │ │ │ ┌─ year (1970 - 3000)
    // │ │ │ │ │ │ │
    // * * * * * * *
    cron.add_fn("0 */15 * * * * *", || {
    // cron.add_fn("*/10 * * * * * *", || {
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute() / 15 * 15;

        let filename = format!("voice_files/{:02}-{:02}.wav", hour, minute);
        let file = File::open(&filename)
            .expect(&format!("Failed to open {}", filename));

        let mut handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
        handle.log_on_drop(false);
        let sink = rodio::play(handle.mixer(), file).unwrap();
        sink.sleep_until_end();
    })?;

    cron.start();
    println!("Cron job started!");
    loop {
        sleep(Duration::from_secs(3600));
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command.unwrap_or(Commands::Run {}) {
        Commands::Gen { speaker_id, url } => handle_gen(speaker_id, url)?,
        Commands::Run { .. } => handle_run()?,
    }
    Ok(())
}
