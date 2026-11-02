fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=resources.rc");
        println!("cargo:rerun-if-changed=app.manifest");
        println!("cargo:rerun-if-changed=app-asinvoker.manifest");

        // `admin` feature -> embed the requireAdministrator manifest instead of the default
        // asInvoker one. resources.rc branches on this define.
        if std::env::var_os("CARGO_FEATURE_ADMIN").is_some() {
            embed_resource::compile("resources.rc", ["ADMIN=1"]);
        } else {
            embed_resource::compile("resources.rc", embed_resource::NONE);
        }
    }
}
