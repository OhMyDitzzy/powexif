/// examples/read_tags.rs
///
/// Reads every EXIF entry from a JPEG file and prints them to stdout.
///
/// Run with:
///   cargo run --example read_tags -- path/to/photo.jpg
use powexif::{ExifData, Ifd};

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: read_tags <file.jpg>");
        std::process::exit(1);
    });

    let exif = match ExifData::read_jpeg(&path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading {path}: {e}");
            std::process::exit(1);
        }
    };

    println!("Byte order: {:?}", exif.byte_order);
    println!();

    let ifd_order = [
        Ifd::Ifd0,
        Ifd::Ifd1,
        Ifd::ExifIfd,
        Ifd::GpsIfd,
        Ifd::InteropIfd,
    ];

    for ifd in ifd_order {
        if let Some(entries) = exif.ifds.get(&ifd) {
            if entries.is_empty() {
                continue;
            }
            println!("[{}]", ifd.name());
            for entry in entries {
                println!(
                    "  {:?} ({}) = {}",
                    entry.tag,
                    entry.value.format().name(),
                    entry.value
                );
            }
            println!();
        }
    }
}
