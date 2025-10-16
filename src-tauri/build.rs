use std::{fs, path::Path, time::Duration};

fn main() {
    let icon_path = Path::new("icons/app-icon.ico");
    if icon_path.exists() {
        println!("cargo:rerun-if-changed=icons/app-icon.ico");
    } else {
        download_icon(icon_path);
    }
    tauri_build::build()
}

fn download_icon(path: &Path) {
    const ICON_URL: &str = "https://mirrors.tuna.tsinghua.edu.cn/github-release/tauri-apps/tauri/latest/download/tauri.ico";
    println!("cargo:warning=attempting to download icon from {ICON_URL}");

    let parent = path.parent().expect("icon path parent");
    if let Err(err) = fs::create_dir_all(parent) {
        panic!("failed to create icon directory: {err}");
    }

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(30))
        .build();

    match agent.get(ICON_URL).call() {
        Ok(response) => {
            if response.status() >= 400 {
                panic!("failed to download icon: HTTP {}", response.status());
            }
            let mut reader = response.into_reader();
            let mut bytes = Vec::new();
            if let Err(err) = std::io::copy(&mut reader, &mut bytes) {
                panic!("failed to read icon data: {err}");
            }
            if let Err(err) = fs::write(path, &bytes) {
                panic!("failed to write icon file: {err}");
            }
        }
        Err(err) => panic!("failed to download icon: {err}"),
    }
}
