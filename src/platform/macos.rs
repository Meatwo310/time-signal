use crate::platform::icon::get_icon_source;

pub fn run_tray() -> anyhow::Result<()> {
    let mut tray = TrayItem::new("Time Signal", get_icon_source())?;

    tray.add_label("Time Signal")?;

    let mut inner = tray.inner_mut();
    inner.add_quit_item("Quit");
    inner.display();

    Ok(())
}
