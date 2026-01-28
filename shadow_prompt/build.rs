// Build script for Windows resource embedding (EXE icon)

fn main() {
    #[cfg(target_os = "windows")]
    {
        embed_resource::compile("resources.rc", embed_resource::NONE);
    }
}
