# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned
- Support for reading/writing XMP metadata
- PNG chunk support
- `--output` flag for non-destructive edits (write to a new file)

---

## [0.0.1] - 2026-06-10

### Added
- Initial release
- Parse EXIF metadata from JPEG and TIFF files
- Write-back support: set, remove, and save EXIF tags
- `show` subcommand with colored table output and optional IFD filter
- `set` subcommand for string, u16, u32, rational, and srational value types
- `remove` subcommand to delete individual tags
- `copy` subcommand to transfer EXIF from one file to another
- `strip` subcommand to remove all EXIF data from a JPEG
- MakerNote parsing for common camera vendors
- Computed fields: aperture, shutter speed, image size, megapixels, scale factor to 35 mm, circle of confusion, field of view, hyperfocal distance, and light value
- Human-readable interpretation for orientation, metering mode, exposure program, white balance, flash, and other enumerated tags
- Full EXIF 2.3 / TIFF 6.0 tag coverage
- Unknown tags preserved as `Tag::Unknown(u16)` on round-trip
- Pure Rust — no C dependencies

[Unreleased]: https://github.com/OhMyDitzzy/powexif/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/OhMyDitzzy/powexif/releases/tag/v0.0.1