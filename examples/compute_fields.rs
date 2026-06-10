/// examples/computed_fields.rs
///
/// Shows how to access computed/derived EXIF fields such as aperture,
/// shutter speed, field of view, and hyperfocal distance.
///
/// Run with:
///   cargo run --example computed_fields -- path/to/photo.jpg
use powexif::ExifData;
use powexif::interpret::ComputedFields;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: computed_fields <file.jpg>");
        std::process::exit(1);
    });

    let exif = ExifData::read_jpeg(&path).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    let c = ComputedFields::compute(&exif);

    fn show(label: &str, value: &Option<String>) {
        match value {
            Some(v) => println!("  {label:<26} {v}"),
            None => println!("  {label:<26} (not available)"),
        }
    }

    println!("Computed fields for {path}:");
    show("Aperture", &c.aperture);
    show("Shutter Speed", &c.shutter_speed);
    show("Image Size", &c.image_size);
    show("Megapixels", &c.megapixels);
    show("Scale Factor to 35mm", &c.scale_factor_35mm);
    show("Circle of Confusion", &c.circle_of_confusion);
    show("Field of View", &c.field_of_view);
    show("Hyperfocal Distance", &c.hyperfocal_distance);
    show("Light Value", &c.light_value);
}
