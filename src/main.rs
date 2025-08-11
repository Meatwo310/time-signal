mod voicevox;

use anyhow::Result;
use clap::{Parser, Subcommand};
use url::Url;
use voicevox::check_voicevox_version;
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
        #[arg(short, long, default_value = "http://127.0.0.1:50021/")]
        url: String,
    },
    Run {
    }
}
fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command.unwrap_or(Commands::Run {}) {
        Commands::Gen { url } => {
            check_voicevox_version(&Url::parse(&url)?)?;
        }
        Commands::Run { .. } => {
            println!("Running!");
        }
    }
    Ok(())
}
