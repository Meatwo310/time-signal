use std::io::Cursor;
use tray_item::IconSource;

pub fn get_icon_source() -> anyhow::Result<IconSource> {
    let cursor = Cursor::new(include_bytes!("../../../icons/time-signal.png"));
    let decoder = png::Decoder::new(cursor);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf)?;
    let bytes = &buf[..info.buffer_size()];

    Ok(IconSource::Data {
        data: bytes.to_vec(),
        height: 256,
        width: 256,
    })
}
