mod voicevox;

use anyhow::Result;
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

fn main() -> Result<()> {
    check_voicevox_version()?;
    Ok(())
}
