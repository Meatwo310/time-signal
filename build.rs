extern crate embed_resource;

fn main() {
    embed_resource::compile("tray-icon.rc", embed_resource::NONE).manifest_optional().unwrap();
}
