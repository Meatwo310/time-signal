extern crate embed_resource;

#[cfg(windows)]
fn main() {
    embed_resource::compile("tray-icon.rc", embed_resource::NONE).manifest_optional().unwrap();
}

#[cfg(not(windows))]
fn main() {}
