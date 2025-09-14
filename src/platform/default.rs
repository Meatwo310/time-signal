use crate::platform::icon::get_icon_source;
use anyhow::Context;
use std::sync::mpsc;
use tray_item::TrayItem;

enum Message {
    Quit,
}

pub fn run_tray() -> anyhow::Result<()> {
    let mut tray = TrayItem::new(
        "Time Signal",
        get_icon_source()?
    ).context("トレイアイコンの作成に失敗しました")?;

    tray.add_label("Time Signal")?;
    tray.inner_mut().add_separator()?;

    let (tx, rx) = mpsc::sync_channel(1);
    let quit_tx = tx.clone();
    tray.add_menu_item("終了", move || {
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
