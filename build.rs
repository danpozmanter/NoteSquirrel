use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let master_path = PathBuf::from("images/NoteSquirrelIcon.png");

    let master = image::open(&master_path)
        .expect("failed to open images/NoteSquirrelIcon.png")
        .into_rgba8();

    // ----------------------------------------------------------------
    // Windows: ICO
    // ----------------------------------------------------------------
    {
        let sizes = [16, 32, 48, 256];
        let mut icons = Vec::new();
        for &size in &sizes {
            let resized = image::imageops::resize(
                &master, size, size, image::imageops::FilterType::Lanczos3,
            );
            let icon_image = ico::IconImage::from_rgba_data(size, size, resized.into_raw());
            let icon = ico::IconDirEntry::encode(&icon_image)
                .expect("encode ico");
            icons.push(icon);
        }
        let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
        for icon in icons {
            dir.add_entry(icon);
        }
        let file = File::create(out_dir.join("icon.ico")).unwrap();
        dir.write(BufWriter::new(file)).unwrap();
    }

    // ----------------------------------------------------------------
    // macOS: ICNS
    // ----------------------------------------------------------------
    {
        use icns::{IconFamily, Image, PixelFormat};
        let mut family = IconFamily::new();
        let sizes = [16, 32, 64, 128, 256];
        for &size in &sizes {
            let resized = image::imageops::resize(
                &master, size, size, image::imageops::FilterType::Lanczos3,
            );
            let image = Image::from_data(
                PixelFormat::RGBA,
                size,
                size,
                resized.into_raw(),
            ).unwrap();
            family.add_icon(&image).unwrap();
        }
        let file = File::create(out_dir.join("icon.icns")).unwrap();
        family.write(BufWriter::new(file)).unwrap();
    }

    // ----------------------------------------------------------------
    // Linux: Embed icon data directly into binary
    // ----------------------------------------------------------------
    {
        let sizes = [16, 32, 48, 128, 256];
        for &size in &sizes {
            let resized = image::imageops::resize(
                &master, size, size, image::imageops::FilterType::Lanczos3,
            );
            resized.save(out_dir.join(format!("icon-{}.png", size))).unwrap();
        }

        let embedded_icon_path = out_dir.join("embedded_icon.rs");
        let mut embedded_file = File::create(&embedded_icon_path).unwrap();
        use std::io::Write;

        writeln!(embedded_file, "// Auto-generated embedded icon data").unwrap();
        writeln!(embedded_file, "pub const ICON_RGBA: &[u8] = &[").unwrap();

        let (width, height) = master.dimensions();
        let rgba_data = master.into_raw();
        for (i, byte) in rgba_data.iter().enumerate() {
            if i % 16 == 0 {
                write!(embedded_file, "\n    ").unwrap();
            }
            write!(embedded_file, "0x{:02x}, ", byte).unwrap();
        }

        writeln!(embedded_file, "\n];").unwrap();
        writeln!(embedded_file, "pub const ICON_WIDTH: u32 = {};", width).unwrap();
        writeln!(embedded_file, "pub const ICON_HEIGHT: u32 = {};", height).unwrap();

        println!("cargo:rustc-env=EMBEDDED_ICON_FILE={}", embedded_icon_path.display());
    }

    println!("cargo:rerun-if-changed=images/NoteSquirrelIcon.png");
}
