# powexif

A fast, lightweight, and pure-Rust library and command-line tool designed for parsing, editing, and saving EXIF metadata from JPEG and TIFF images.

[![Crates.io](https://img.shields.io/crates/v/powexif.svg)](https://crates.io/crates/powexif)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/OhMyDitzzy/powexif/release.yml?branch=main)](https://github.com/OhMyDitzzy/powexif/actions)

## Overview
**powexif** provides a robust, zero-dependency (no `libexif` or C-bindings) solution for handling image metadata in Rust. Whether you need to build a high-performance photo processing pipeline, a privacy tool to strip tracking data, or simply want a quick CLI tool to inspect your shots, `powexif` offers a safe and idiomatic Rust API alongside a feature-rich command-line interface.

### Why powexif?
- [x] **Safety First:** Built entirely in pure Rust, eliminating common memory-safety vulnerabilities associated with traditional C-based metadata parsers.
- [x] **Smart Computations:** Beyond basic tag reading, it automatically interprets raw metadata into human-readable photography metrics like Field of View (FoV), Hyperfocal Distance, and Aperture.
- [x] **Non-Destructive:** Unknown or vendor-specific tags are preserved during write operations, ensuring data integrity on round-trips.

## Features

- **Full IFD Support:** Read and write across all standard EXIF directories (`IFD0`, `IFD1`, `Exif IFD`, `GPS IFD`, `Interop IFD`).
- **Vendor Insights:** Built-in MakerNote support for common camera manufacturers.
- **Computed Optics:** On-the-fly calculation for `aperture`, `shutter_speed`, `megapixels`, `field_of_view`, `hyperfocal_distance`, and more.
- **Privacy Utilities:** Instantly strip the entire `APP1` segment or selectively clear precise GPS location tags.
- **Metadata Cloning:** Easily duplicate EXIF structures from a source image directly into a destination file.
- **Beautiful CLI:** Colored, structured terminal tables for quick metadata inspection.

## Installation

### As a Command-Line Tool

Make sure you have Cargo installed, then run:
```bash
cargo install powexif
```

### As a Dependency

Add this to your `Cargo.toml`:

```toml
[dependencies]
powexif = "<version>" # Replace with the latest version
```

Or use the Cargo CLI:
```bash
cargo add powexif
```

## CLI Usage
powexif comes with an intuitive CLI for rapid metadata manipulation.

### Show all EXIF tags
```bash
# View all embedded EXIF tags in a structured table
powexif show photo.jpg

# Filter inspection to a specific directory (e.g., GPS metadata)
powexif show photo.jpg --ifd gps

# Include vendor-specific MakerNote data
powexif show photo.jpg --makernote
```

Valid --ifd selectors: ifd0, ifd1, exif, gps, interop.

### Modifying & Deleting Tags

```bash
# Update or insert a string metadata tag
powexif set photo.jpg --tag artist --type string "Jane Doe"

# Set numeric or complex types (e.g., Orientation, Aperture)
powexif set photo.jpg --tag orientation --type u16 1
powexif set photo.jpg --tag fnumber --type rational 28/10

# Remove a specific tag entirely
powexif remove photo.jpg --ifd gps --tag gpslongitude
```

Supported `--type` values: `string` (or `ascii`), `u16` (or `short`), `u32` (or `long`), `rational`, `srational`.

### Privacy & Maintenance
```bash
# Strip all EXIF data entirely before sharing online
powexif strip photo.jpg

# Clone metadata structure from one asset to another
powexif copy source.jpg destination.jpg
```

### Copy EXIF from one file to another

```bash
powexif copy source.jpg destination.jpg
```

This replaces all EXIF data in `destination.jpg` with the EXIF data from `source.jpg`.

---

### Strip all EXIF data

```bash
powexif strip photo.jpg
```

Removes the APP1 EXIF segment entirely. Useful for sharing photos without embedded metadata.

---

## Library Usage

### Parse EXIF from a JPEG file

```rust
use powexif::{ExifData, Ifd, Tag};

fn main() -> powexif::Result<()> {
    // Load and parse from file path
    let exif = ExifData::read_jpeg("photo.jpg")?;

    // Safe retrieval of specific standard tags
    if let Some(entry) = exif.get(Ifd::Ifd0, Tag::Make) {
        println!("Camera Manufacturer: {}", entry.value);
    }
    Ok(())
}
```

### Set and save a tag

```rust
use powexif::{ExifData, Ifd, Tag, Value};

fn main() -> powexif::Result<()> {
    let mut exif = ExifData::read_jpeg("photo.jpg")?;

    // Insert or update metadata values safely
    exif.set(Ifd::Ifd0, Tag::Artist, Value::Ascii("Jane Doe".to_string()));
    
    // Save modifications back to the asset
    exif.save_jpeg("photo.jpg")?;
    Ok(())
}
```

### Computed fields
You don't need to manually calculate complex optical math. powexif computes them for you:

```rust
use powexif::{ExifData, interpret::ComputedFields};

fn main() -> powexif::Result<()> {
    let exif = ExifData::read_jpeg("photo.jpg")?;
    let computed = ComputedFields::compute(&exif);

    if let Some(fov) = &computed.field_of_view {
        println!("Calculated Field of View: {}", fov);
    }
    if let Some(hyperfocal) = &computed.hyperfocal_distance {
        println!("Hyperfocal Distance: {}m", hyperfocal);
    }
    Ok(())
}

```

Available computed fields: `aperture`, `shutter_speed`, `image_size`, `megapixels`, `scale_factor_35mm`, `circle_of_confusion`, `field_of_view`, `hyperfocal_distance`, `light_value`.

## Supported Tags

powexif covers all standard EXIF 2.3 / TIFF 6.0 tags, organized across five IFDs:

| IFD | Contents |
|-----|----------|
| `ifd0` | Primary image info (Make, Model, DateTime, Orientation, etc.) |
| `ifd1` | Thumbnail image info |
| `exif` | Capture settings (ExposureTime, FNumber, ISO, FocalLength, etc.) |
| `gps` | GPS coordinates and metadata |
| `interop` | Interoperability information |

*Note:* Unrecognized or non-standard tags are safely preserved as Tag::Unknown(u16) during read/write cycles, ensuring your files don't lose custom metadata.

## Error Handling

All fallible operations return `powexif::Result<T>`, which is an alias for `std::result::Result<T, ExifError>`. The `ExifError` type covers:

- `InvalidHeader` — not a valid JPEG SOI or EXIF header
- `InvalidByteOrder` — TIFF byte-order marker not found
- `NoExifSegment` — no EXIF APP1 segment in the JPEG
- `OffsetOutOfBounds` — a parsed offset points outside the buffer
- `UnexpectedEnd` — not enough bytes to read the expected data
- `UnknownFormat` — unrecognized EXIF data format code
- `FormatMismatch` — value type does not match what the tag expects
- `InvalidUtf8` — ASCII tag contains non-UTF-8 bytes
- `Io` — underlying file I/O error

## License

powexif is Licensed under the [Apache License, Version 2.0](LICENSE).