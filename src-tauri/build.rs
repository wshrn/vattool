use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::Path;

const ICON_URL: &str = "https://mirrors.tuna.tsinghua.edu.cn/static/img/favicon.png";
const ICON_DIR: &str = "icons";
const ICON_PNG: &str = "icons/icon.png";
const ICON_ICO: &str = "icons/icon.ico";
const ICON_ICNS: &str = "icons/icon.icns";
const FALLBACK_ICON_BASE64: &str = "iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAAGXRFWHRTb2Z0d2FyZQBBZG9iZSBJbWFnZVJlYWR5ccllPAAAABh0RVh0Q3JlYXRpb24gVGltZQAwNS8xMi8xN6W2N/sAAABiSURBVHja7NoxAQAgEMLA+nc4hQ4MsHMh6EBH7rV1CQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAZw45AAGlOfnIAAAAAElFTkSuQmCC";

fn main() {
    if let Err(error) = ensure_icons() {
        eprintln!("警告: 构建阶段处理图标失败: {error}");
    }
    tauri_build::build()
}

fn ensure_icons() -> Result<(), String> {
    fs::create_dir_all(ICON_DIR).map_err(|err| format!("无法创建图标目录: {err}"))?;

    if Path::new(ICON_PNG).exists() && Path::new(ICON_ICO).exists() && Path::new(ICON_ICNS).exists() {
        return Ok(());
    }

    let png_bytes = download_icon().unwrap_or_else(|| decode_fallback());
    write_png(&png_bytes)?;
    write_ico(&png_bytes)?;
    write_icns(&png_bytes)?;
    Ok(())
}

fn download_icon() -> Option<Vec<u8>> {
    match ureq::get(ICON_URL).timeout_connect(10_000).call() {
        Ok(response) => {
            if response.status() != 200 {
                eprintln!("警告: 国内镜像图标下载失败，状态码 {}", response.status());
                return None;
            }
            let mut reader = response.into_reader();
            let mut buffer = Vec::new();
            if reader.read_to_end(&mut buffer).is_ok() {
                Some(buffer)
            } else {
                None
            }
        }
        Err(error) => {
            eprintln!("警告: 请求国内图标镜像失败: {error}");
            None
        }
    }
}

fn decode_fallback() -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(FALLBACK_ICON_BASE64)
        .expect("内置图标数据损坏")
}

fn write_png(data: &[u8]) -> Result<(), String> {
    File::create(ICON_PNG)
        .and_then(|mut file| file.write_all(data))
        .map_err(|err| format!("写入 PNG 图标失败: {err}"))
}

fn write_ico(data: &[u8]) -> Result<(), String> {
    let image = image::load_from_memory(data).map_err(|err| format!("解析 PNG 图标失败: {err}"))?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let file = File::create(ICON_ICO).map_err(|err| format!("创建 ICO 文件失败: {err}"))?;
    let mut writer = BufWriter::new(file);
    let mut encoder = image::codecs::ico::IcoEncoder::new(&mut writer);
    encoder
        .encode(&rgba, width as u32, height as u32, image::ColorType::Rgba8)
        .map_err(|err| format!("编码 ICO 图标失败: {err}"))?;
    writer.flush().map_err(|err| format!("写入 ICO 图标失败: {err}"))
}

fn write_icns(data: &[u8]) -> Result<(), String> {
    let icon = icns::Icon::from_png(data).map_err(|err| format!("解析 PNG 生成 ICNS 失败: {err}"))?;
    let mut family = icns::IconFamily::new();
    family.add_icon(&icon).map_err(|err| format!("添加 ICNS 图标失败: {err}"))?;
    let file = File::create(ICON_ICNS).map_err(|err| format!("创建 ICNS 文件失败: {err}"))?;
    let mut writer = BufWriter::new(file);
    family
        .write(&mut writer)
        .map_err(|err| format!("写入 ICNS 文件失败: {err}"))?;
    writer.flush().map_err(|err| format!("刷新 ICNS 文件失败: {err}"))
}
