use std::{
    fs,
    path::Path,
};

fn main() -> anyhow::Result<()> {
    let out_dir = Path::new("src-tauri/icons");
    if !out_dir.exists() {
        fs::create_dir_all(out_dir)?;
    }

    let icon_path = out_dir.join("app-icon.png");
    let icon_url = "https://mirrors.tuna.tsinghua.edu.cn/static/img/pypi.png";

    println!("cargo:rerun-if-changed={}", icon_path.display());

    if !icon_path.exists() {
        download_icon(&icon_path, icon_url)?;
    }

    Ok(())
}

fn download_icon(path: &Path, url: &str) -> anyhow::Result<()> {
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        anyhow::bail!("无法下载图标文件: {}", response.status());
    }
    let bytes = response.bytes()?;
    fs::write(path, &bytes)?;
    Ok(())
}
