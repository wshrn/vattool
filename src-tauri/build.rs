use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    const ICON_URL: &str = "https://mirrors.tuna.tsinghua.edu.cn/github-release/tauri-apps/icons/LatestRelease/app-icon.ico";
    let icon_path = Path::new("icons/app-icon.ico");

    if !icon_path.exists() {
        println!("cargo:warning=downloading application icon from Tsinghua mirror");
        let bytes = reqwest::blocking::get(ICON_URL)
            .context("failed to download icon from Tsinghua mirror")?
            .bytes()
            .context("unable to read icon bytes")?;
        if let Some(parent) = icon_path.parent() {
            fs::create_dir_all(parent).context("failed to create icon directory")?;
        }
        fs::write(icon_path, &bytes).context("failed to write icon file")?;
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icons/app-icon.ico");
    Ok(())
}
