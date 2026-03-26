//! Build script – platform-specific build steps

fn main() {
    // Windows: embed the app icon as a resource into the .exe
    #[cfg(target_os = "windows")]
    {
        let _ = embed_resource::compile("macro_paste.rc", embed_resource::NONE);
    }
}
