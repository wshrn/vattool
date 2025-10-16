use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

fn main() {
    ensure_icon();
    tauri_build::build()
}

fn ensure_icon() {
    let icon_path = Path::new("src-tauri/icons/app-icon.png");
    if icon_path.exists() {
        return;
    }

    if let Some(parent) = icon_path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!("Failed to create icon directory: {err}");
            return;
        }
    }

    let url = "https://mirrors.tuna.tsinghua.edu.cn/static/img/favicon.png";
    match ureq::get(url).call() {
        Ok(response) => {
            if let Ok(mut reader) = response.into_reader() {
                let mut data = Vec::new();
                if let Err(err) = reader.read_to_end(&mut data) {
                    eprintln!("Failed to download icon: {err}");
                    return;
                }
                if let Err(err) = fs::File::create(icon_path).and_then(|mut file| file.write_all(&data)) {
                    eprintln!("Failed to write icon file: {err}");
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to fetch icon: {err}");
        }
    }
}
