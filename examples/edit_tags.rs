use powexif::{ExifData, Ifd, Tag, Value};
/// examples/edit_tags.rs
///
/// Demonstrates setting, reading back, and removing an EXIF tag
/// on a copy of the input file (leaves the original untouched).
///
/// Run with:
///   cargo run --example edit_tags -- path/to/photo.jpg
use std::fs;

fn main() {
    let src = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: edit_tags <file.jpg>");
        std::process::exit(1);
    });

    // Work on a copy so we don't modify the original.
    let dst = format!("{src}.edited.jpg");
    fs::copy(&src, &dst).expect("failed to copy file");

    // --- Load ---
    let mut exif = ExifData::read_jpeg(&dst).expect("failed to read EXIF");

    // --- Set a string tag ---
    exif.set(
        Ifd::Ifd0,
        Tag::Artist,
        Value::Ascii("powexif example".to_string()),
    );
    exif.set(
        Ifd::Ifd0,
        Tag::Copyright,
        Value::Ascii("2026 My Name".to_string()),
    );

    // --- Set a rational tag (e.g. X resolution = 72/1) ---
    use powexif::value::Rational;
    exif.set(
        Ifd::Ifd0,
        Tag::XResolution,
        Value::Rational(vec![Rational::new(72, 1)]),
    );

    // --- Save ---
    exif.save_jpeg(&dst).expect("failed to save EXIF");
    println!("Saved edited file to {dst}");

    // --- Read back and verify ---
    let exif2 = ExifData::read_jpeg(&dst).expect("failed to re-read");
    if let Some(entry) = exif2.get(Ifd::Ifd0, Tag::Artist) {
        println!("Artist tag = {}", entry.value);
    }

    // --- Remove the tag ---
    let mut exif3 = ExifData::read_jpeg(&dst).expect("failed to read for removal");
    let removed = exif3.remove(Ifd::Ifd0, Tag::Artist);
    println!("Artist removed: {removed}");
    exif3.save_jpeg(&dst).expect("failed to save after removal");

    // Confirm it's gone
    let exif4 = ExifData::read_jpeg(&dst).expect("failed to re-read after removal");
    match exif4.get(Ifd::Ifd0, Tag::Artist) {
        Some(_) => println!("Artist still present (unexpected)"),
        None => println!("Artist tag successfully removed"),
    }
}
