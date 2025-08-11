mod voicevox;

use crate::voicevox::VoicevoxClient;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use url::Url;
use zip::ZipArchive;
// fn main() {
//     let mut cron = Cron::new(Local);
//
//     // https://github.com/tuyentv96/rust-crontab?tab=readme-ov-file#-cron-expression-format
//     // ┌───────────── second (0 - 59)
//     // │ ┌─────────── minute (0 - 59)
//     // │ │ ┌───────── hour (0 - 23)
//     // │ │ │ ┌─────── day of month (1 - 31)
//     // │ │ │ │ ┌───── month (1 - 12)
//     // │ │ │ │ │ ┌─── day of week (0 - 6) (Sunday to Saturday)
//     // │ │ │ │ │ │ ┌─ year (1970 - 3000)
//     // │ │ │ │ │ │ │
//     // * * * * * * *
//     cron.add_fn("* */15 * * * * *", || {
//         println!("HELLO WORLD");
//     }).unwrap();
//
//     cron.start();
//     sleep(Duration::from_secs(20));
//     cron.stop();
// }

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
            let text = format!("{}時{}分です", hour, minute);
            let query = client.audio_query(&text, speaker_id)?;
            minute_queries.insert(minute, query);
        }
        queries.insert(hour, minute_queries);
    }

    println!("Generated {} queries in total", queries.len() * 4);

    // voice_filesディレクトリを作成
    std::fs::create_dir_all("voice_files")?;

    // 時間ごとにボイスファイルを生成
    for hour in 0..24 {
        println!("Generating audio files for {}時...", hour);

        let hour_queries = queries.get(&hour)
            .context("Hour queries not found")?;

        let query_vec: Vec<String> = [0, 15, 30, 45]
            .iter()
            .map(|minute| hour_queries.get(minute).unwrap().clone())
            .collect();

        let zip_data = client.multi_synthesis(&query_vec, speaker_id)?;

        // ZIPデータをメモリ上で展開
        let cursor = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(cursor)?;

        // 時間ごとのサブディレクトリを作成
        let hour_dir = format!("voice_files/{:02}", hour);
        std::fs::create_dir_all(&hour_dir)?;

        // ZIPファイル内の各ファイルを展開して保存
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;

            if file.is_file() {
                let file_name = file.name().to_string();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;

                let output_path = format!("{}/{}", hour_dir, file_name);
                std::fs::write(&output_path, buffer)?;
            }
        }
    }

    println!("All voice files generated and saved successfully!");

    Ok(())
}

fn handle_run() -> Result<()> {
    println!("Running!");
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command.unwrap_or(Commands::Run {}) {
        Commands::Gen { speaker_id, url } => handle_gen(speaker_id, url)?,
        Commands::Run { .. } => handle_run()?,
    }
    Ok(())
}
