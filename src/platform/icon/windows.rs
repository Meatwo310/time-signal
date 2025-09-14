use anyhow::Result;
use tray_item::IconSource;

pub fn get_icon_source() -> Result<IconSource> {
    Ok(IconSource::Resource("tray-default"))
}
