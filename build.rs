use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use resvg::{tiny_skia, usvg};

const APP_ICON: &str = "ui/icons/app-icon.svg";
/// Sizes Explorer, the taskbar and Alt+Tab pick from, across display scales.
const ICON_SIZES: [u32; 8] = [16, 20, 24, 32, 40, 48, 64, 256];

fn main() {
    slint_build::compile("ui/app.slint").expect("Slint UI should compile");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed={APP_ICON}");
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
        let icon = out_dir.join("app.ico");
        write_ico(Path::new(APP_ICON), &icon);
        // The window's taskbar icon comes from app.slint; this one is what Explorer shows for the .exe.
        winresource::WindowsResource::new()
            .set_icon(icon.to_str().expect("OUT_DIR is valid Unicode"))
            .compile()
            .expect("the Windows SDK resource compiler should embed the app icon");
    }
}

/// Renders the SVG at every icon size and packs the PNGs into one .ico file.
fn write_ico(svg: &Path, ico: &Path) {
    let data = fs::read(svg).expect("app icon SVG should be readable");
    let tree = usvg::Tree::from_data(&data, &usvg::Options::default()).expect("app icon SVG should parse");

    let images: Vec<(u32, Vec<u8>)> = ICON_SIZES
        .iter()
        .map(|&size| {
            let mut pixmap = tiny_skia::Pixmap::new(size, size).expect("icon sizes are non-zero");
            let scale = size as f32 / tree.size().width();
            resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
            (size, pixmap.encode_png().expect("rendered icon should encode as PNG"))
        })
        .collect();

    // ICO layout: 6-byte header, one 16-byte directory entry per image, then the PNG payloads.
    let count = images.len() as u16;
    let mut offset = 6 + 16 * u32::from(count);
    let mut file = Vec::new();
    file.extend_from_slice(&[0, 0]);
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&count.to_le_bytes());
    for (size, png) in &images {
        // 256 is stored as 0 in the one-byte width and height fields.
        let side = if *size >= 256 { 0 } else { *size as u8 };
        file.extend_from_slice(&[side, side, 0, 0]);
        file.extend_from_slice(&1u16.to_le_bytes());
        file.extend_from_slice(&32u16.to_le_bytes());
        file.extend_from_slice(&(png.len() as u32).to_le_bytes());
        file.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for (_, png) in &images {
        file.extend_from_slice(png);
    }
    fs::write(ico, file).expect("OUT_DIR should be writable");
}
