fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        #[cfg(target_os = "windows")]
        {
            let mut res = winres::WindowsResource::new();
            res.compile().unwrap();
        }
    }
}