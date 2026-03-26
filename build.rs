//! Build script – embeds the app icon as a Windows resource into the .exe

fn main() {
    // Embed the icon so it appears in the taskbar and file explorer
    let _ = embed_resource::compile("macro_paste.rc", embed_resource::NONE);
}
