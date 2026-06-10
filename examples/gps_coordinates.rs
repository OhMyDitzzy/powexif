/// examples/gps_coordinates.rs
///
/// Reads GPS latitude and longitude from a JPEG and converts to decimal degrees.
///
/// Run with:
///   cargo run --example gps_coordinates -- path/to/photo.jpg
use powexif::{ExifData, Ifd, Tag, Value};

fn dms_to_decimal(value: &Value, reference: &str) -> Option<f64> {
    let rationals = match value {
        Value::Rational(v) if v.len() >= 3 => v,
        _ => return None,
    };

    let deg = rationals[0].as_f64()?;
    let min = rationals[1].as_f64()?;
    let sec = rationals[2].as_f64()?;

    let decimal = deg + min / 60.0 + sec / 3600.0;

    if reference == "S" || reference == "W" {
        Some(-decimal)
    } else {
        Some(decimal)
    }
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: gps_coordinates <file.jpg>");
        std::process::exit(1);
    });

    let exif = ExifData::read_jpeg(&path).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    let lat_ref = exif
        .get(Ifd::GpsIfd, Tag::GpsLatitudeRef)
        .and_then(|e| match &e.value {
            Value::Ascii(s) => Some(s.trim_end_matches('\0').to_string()),
            _ => None,
        });

    let lon_ref = exif
        .get(Ifd::GpsIfd, Tag::GpsLongitudeRef)
        .and_then(|e| match &e.value {
            Value::Ascii(s) => Some(s.trim_end_matches('\0').to_string()),
            _ => None,
        });

    let lat = exif
        .get(Ifd::GpsIfd, Tag::GpsLatitude)
        .zip(lat_ref.as_deref())
        .and_then(|(e, r)| dms_to_decimal(&e.value, r));

    let lon = exif
        .get(Ifd::GpsIfd, Tag::GpsLongitude)
        .zip(lon_ref.as_deref())
        .and_then(|(e, r)| dms_to_decimal(&e.value, r));

    match (lat, lon) {
        (Some(lat), Some(lon)) => {
            println!("GPS coordinates: {lat:.6}, {lon:.6}");
            println!("Google Maps: https://www.google.com/maps?q={lat:.6},{lon:.6}");
        }
        _ => println!("No GPS data found in {path}"),
    }
}
