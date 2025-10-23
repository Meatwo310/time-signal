mod gen;
mod platform;
mod run;
mod voicevox;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
        #[arg(short = 't', long, default_value = "10")]
        idle_timeout: u64,

        /// CLIモードで実行します。トレイアイコンは表示されません。
        #[arg(long)]
        cli: bool,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command.unwrap_or(Commands::Run {
        interval: 15,
        idle_timeout: 10,
        cli: false,
    }) {
        Commands::Gen {
            speaker_id,
            url,
            interval,
        } => gen::handle_gen(speaker_id, url, interval)?,
        Commands::Run {
            interval,
            idle_timeout,
            cli,
        } => run::handle_run(interval, idle_timeout, cli)?,
    }
    Ok(())
}
