use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let master_path = PathBuf::from("images/NoteSquirrelIcon.png");

    // Load master PNG
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
    // Linux: PNGs + one default runtime icon
    // ----------------------------------------------------------------
    {
        let sizes = [16, 32, 48, 128, 256];
        for &size in &sizes {
            let resized = image::imageops::resize(
                &master, size, size, image::imageops::FilterType::Lanczos3,
            );
            resized.save(out_dir.join(format!("icon-{}.png", size))).unwrap();
        }

        // Save one standard PNG for runtime (256px)
        let runtime_icon_path = out_dir.join("window-icon.png");
        master.save(&runtime_icon_path).unwrap();

        // Tell Cargo to pass this path to the app as an env var
        println!("cargo:rustc-env=APP_ICON={}", runtime_icon_path.display());
    }

    println!("cargo:rerun-if-changed=images/NoteSquirrelIcon.png");
}
