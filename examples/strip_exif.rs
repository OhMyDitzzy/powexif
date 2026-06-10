use powexif::ExifData;
/// examples/strip_exif.rs
///
/// Strips all EXIF data from a JPEG by parsing with ExifData and saving
/// without any IFD entries. Useful for removing metadata before sharing.
///
/// Run with:
///   cargo run --example strip_exif -- input.jpg output.jpg
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: strip_exif <input.jpg> <output.jpg>");
        std::process::exit(1);
    }

    let (src, dst) = (&args[1], &args[2]);

    // Copy first so we have the JPEG structure intact.
    fs::copy(src, dst).expect("failed to copy input file");

    let mut exif = ExifData::read_jpeg(dst).unwrap_or_else(|e| {
        eprintln!("Failed to read EXIF from {src}: {e}");
        std::process::exit(1);
    });

    // Clear every IFD.
    exif.ifds.clear();
    exif.maker_note = None;

    exif.save_jpeg(dst).expect("failed to save stripped file");

    let before = fs::metadata(src).map(|m| m.len()).unwrap_or(0);
    let after = fs::metadata(dst).map(|m| m.len()).unwrap_or(0);

    println!("Stripped EXIF from {src}.");
    println!("  Before: {} bytes", before);
    println!("  After:  {} bytes", after);
    println!("  Saved:  {} bytes", before.saturating_sub(after));
    println!("  Output: {dst}");
}
