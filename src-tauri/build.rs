use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let icon_dir = manifest_dir.join("icons");
    let icon_path = icon_dir.join("icon.png");

    println!("cargo:rerun-if-changed=build.rs");

    if let Err(err) = fs::create_dir_all(&icon_dir) {
        eprintln!("Failed to create icon directory: {err}");
        return;
    }

    if icon_path.exists() {
        return;
    }

    let icon_url = "https://mirrors.cloud.tencent.com/pypi/static/images/logo-small.8d3180ef.png";

    match reqwest::blocking::get(icon_url) {
        Ok(response) => {
            if let Ok(bytes) = response.bytes() {
                if let Ok(mut file) = fs::File::create(&icon_path) {
                    if file.write_all(&bytes).is_err() {
                        eprintln!("Failed to write icon file");
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to download icon: {err}");
        }
    }

    tauri_build::build()
}
