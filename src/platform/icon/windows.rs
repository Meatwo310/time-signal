use tray_item::IconSource;

pub fn get_icon_source() -> anyhow::Result<IconSource> {
    Ok(IconSource::Resource("tray-default"))
}
