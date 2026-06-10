/*
   Copyright 2026 OhMyDitzzy

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use colored::Colorize;
use prettytable::{Cell, Row, Table, format, row};

use powexif::interpret::{ComputedFields, interpret};
use powexif::value::{Rational, SRational};
use powexif::{ExifData, Ifd, Tag, Value};

#[derive(Parser)]
#[command(
    name = "powexif",
    version,
    about = "Parse, edit, and save EXIF metadata from JPEG or TIFF files",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Display all EXIF tags in a file.
    Show {
        /// Path to the JPEG or TIFF file.
        file: PathBuf,

        /// Show only the specified IFD (ifd0, ifd1, exif, gps, interop).
        #[arg(long, value_name = "IFD")]
        ifd: Option<String>,

        /// Also display MakerNote entries when present.
        #[arg(long)]
        makernote: bool,
    },

    /// Set or replace one EXIF tag.
    Set {
        file: PathBuf,
        #[arg(long, default_value = "ifd0")]
        ifd: String,
        #[arg(long)]
        tag: String,
        #[arg(long, default_value = "string")]
        r#type: String,
        value: String,
    },

    /// Remove one EXIF tag.
    Remove {
        file: PathBuf,
        #[arg(long, default_value = "ifd0")]
        ifd: String,
        #[arg(long)]
        tag: String,
    },

    /// Copy EXIF data from one file into another.
    Copy {
        source: PathBuf,
        destination: PathBuf,
    },

    /// Strip all EXIF data from a JPEG file.
    Strip { file: PathBuf },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Show {
            file,
            ifd,
            makernote,
        } => cmd_show(&file, ifd.as_deref(), makernote),
        Command::Set {
            file,
            ifd,
            tag,
            r#type,
            value,
        } => cmd_set(&file, &ifd, &tag, &r#type, &value),
        Command::Remove { file, ifd, tag } => cmd_remove(&file, &ifd, &tag),
        Command::Copy {
            source,
            destination,
        } => cmd_copy(&source, &destination),
        Command::Strip { file } => cmd_strip(&file),
    };

    if let Err(e) = result {
        eprintln!("{} {e}", "error:".red().bold());
        process::exit(1);
    }
}

fn cmd_show(path: &PathBuf, ifd_filter: Option<&str>, show_makernote: bool) -> anyhow::Result<()> {
    let exif = load_file(path)?;
    let filter = ifd_filter.map(parse_ifd_name).transpose()?;

    print_file_info(path, &exif);

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);

    table.set_titles(row![
        bFc -> "IFD",
        bFc -> "Tag",
        bFc -> "Format",
        bFc -> "Value"
    ]);

    for (ifd, entry) in exif.all_entries() {
        if let Some(f) = filter
            && ifd != f {
                continue;
            }
        match entry.tag {
            Tag::ExifIfdPointer | Tag::GpsInfoIfdPointer | Tag::InteroperabilityIfdPointer => {
                continue;
            }
            _ => {}
        }

        let ifd_str = ifd_colored(ifd);
        let tag_str = entry.tag.name().bold().to_string();
        let fmt_str = format_colored(entry.value.format().name());
        let val_str = value_display(entry.tag, &entry.value);

        table.add_row(Row::new(vec![
            Cell::new(&ifd_str),
            Cell::new(&tag_str),
            Cell::new(&fmt_str),
            Cell::new(&val_str),
        ]));
    }

    let computed = ComputedFields::compute(&exif);
    add_computed_row(&mut table, "Aperture", &computed.aperture);
    add_computed_row(&mut table, "Shutter Speed", &computed.shutter_speed);
    add_computed_row(&mut table, "Image Size", &computed.image_size);
    add_computed_row(&mut table, "Megapixels", &computed.megapixels);
    add_computed_row(
        &mut table,
        "Scale Factor To 35mm",
        &computed.scale_factor_35mm,
    );
    add_computed_row(
        &mut table,
        "Circle Of Confusion",
        &computed.circle_of_confusion,
    );
    add_computed_row(&mut table, "Field Of View", &computed.field_of_view);
    add_computed_row(
        &mut table,
        "Hyperfocal Distance",
        &computed.hyperfocal_distance,
    );
    add_computed_row(&mut table, "Light Value", &computed.light_value);

    table.printstd();

    if show_makernote {
        if let Some(mn) = &exif.maker_note {
            println!(
                "\n{} ({})",
                "MakerNote".cyan().bold(),
                format!("{:?}", mn.kind).dimmed()
            );
            let mut mn_table = Table::new();
            mn_table.set_format(*format::consts::FORMAT_BOX_CHARS);
            mn_table.set_titles(row![bFc -> "Tag", bFc -> "Format", bFc -> "Value"]);
            for (tag, value) in mn.iter() {
                mn_table.add_row(Row::new(vec![
                    Cell::new(&tag.name().bold().to_string()),
                    Cell::new(&format_colored(value.format().name())),
                    Cell::new(&value.to_string()),
                ]));
            }
            mn_table.printstd();
        } else {
            println!("\n{}", "No MakerNote found.".dimmed());
        }
    }

    Ok(())
}

fn print_file_info(path: &PathBuf, exif: &ExifData) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    table.set_titles(row![bFy -> "File Info", bFy -> "Value"]);

    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        table.add_row(row!["File Name", name]);
    }

    if let Ok(meta) = std::fs::metadata(path) {
        let size = meta.len();
        let size_str = if size >= 1_048_576 {
            format!("{:.1} MB", size as f64 / 1_048_576.0)
        } else if size >= 1024 {
            format!("{:.0} kB", size as f64 / 1024.0)
        } else {
            format!("{} bytes", size)
        };
        table.add_row(row!["File Size", size_str]);
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let (file_type, mime) = match ext.as_str() {
        "jpg" | "jpeg" => ("JPEG", "image/jpeg"),
        "tif" | "tiff" => ("TIFF", "image/tiff"),
        _ => ("Unknown", "application/octet-stream"),
    };
    table.add_row(row!["File Type", file_type]);
    table.add_row(row!["MIME Type", mime]);

    let byte_order_str = match exif.byte_order {
        powexif::ByteOrder::LittleEndian => "Little-endian (Intel, II)",
        powexif::ByteOrder::BigEndian => "Big-endian (Motorola, MM)",
    };
    table.add_row(row!["Exif Byte Order", byte_order_str]);

    table.printstd();
    println!();
}

fn add_computed_row(table: &mut Table, label: &str, value: &Option<String>) {
    if let Some(v) = value {
        table.add_row(Row::new(vec![
            Cell::new(&"[computed]".dimmed().to_string()),
            Cell::new(&label.italic().to_string()),
            Cell::new(&"—".dimmed().to_string()),
            Cell::new(v),
        ]));
    }
}

fn ifd_colored(ifd: Ifd) -> String {
    match ifd {
        Ifd::Ifd0 => ifd.name().cyan().to_string(),
        Ifd::Ifd1 => ifd.name().blue().to_string(),
        Ifd::ExifIfd => ifd.name().green().to_string(),
        Ifd::GpsIfd => ifd.name().yellow().to_string(),
        Ifd::InteropIfd => ifd.name().magenta().to_string(),
    }
}

fn format_colored(name: &str) -> String {
    match name {
        "ASCII" => name.yellow().to_string(),
        "SHORT" | "LONG" | "SSHORT" | "SLONG" | "BYTE" | "SBYTE" => name.blue().to_string(),
        "RATIONAL" | "SRATIONAL" => name.cyan().to_string(),
        "UNDEFINED" => name.dimmed().to_string(),
        _ => name.to_string(),
    }
}

fn value_display(tag: Tag, value: &Value) -> String {
    if let Some(human) = interpret(tag, value) {
        human
    } else {
        value.to_string()
    }
}

fn cmd_set(
    path: &PathBuf,
    ifd_str: &str,
    tag_str: &str,
    type_str: &str,
    raw_value: &str,
) -> anyhow::Result<()> {
    let mut exif = load_file(path)?;
    let ifd = parse_ifd_name(ifd_str)?;
    let tag = parse_tag_name(tag_str);
    let value = parse_value(type_str, raw_value)?;

    exif.set(ifd, tag, value);
    save_file(path, &exif)?;
    println!("{} {tag_str} in {}.", "Set".green().bold(), ifd.name());
    Ok(())
}

fn cmd_remove(path: &PathBuf, ifd_str: &str, tag_str: &str) -> anyhow::Result<()> {
    let mut exif = load_file(path)?;
    let ifd = parse_ifd_name(ifd_str)?;
    let tag = parse_tag_name(tag_str);

    if exif.remove(ifd, tag) {
        save_file(path, &exif)?;
        println!(
            "{} {tag_str} from {}.",
            "Removed".green().bold(),
            ifd.name()
        );
    } else {
        println!(
            "{} {tag_str} not found in {}.",
            "Warning:".yellow().bold(),
            ifd.name()
        );
    }
    Ok(())
}

fn cmd_copy(source: &PathBuf, dest: &PathBuf) -> anyhow::Result<()> {
    let src_exif = load_file(source)?;

    let dest_bytes = std::fs::read(dest)?;
    let mut dest_exif = ExifData::from_jpeg_bytes(&dest_bytes)
        .map_err(|e| anyhow::anyhow!("destination file: {e}"))?;

    dest_exif.ifds = src_exif.ifds;
    dest_exif.maker_note = src_exif.maker_note;
    dest_exif.byte_order = src_exif.byte_order;

    let out = dest_exif.to_jpeg_bytes()?;
    std::fs::write(dest, out)?;
    println!(
        "{} EXIF from {} to {}.",
        "Copied".green().bold(),
        source.display(),
        dest.display()
    );
    Ok(())
}

fn cmd_strip(path: &PathBuf) -> anyhow::Result<()> {
    let bytes = std::fs::read(path)?;
    let stripped = strip_app1(&bytes)?;
    std::fs::write(path, stripped)?;
    println!(
        "{} EXIF from {}.",
        "Stripped".green().bold(),
        path.display()
    );
    Ok(())
}

fn strip_app1(jpeg: &[u8]) -> anyhow::Result<Vec<u8>> {
    if jpeg.len() < 2 || jpeg[0] != 0xFF || jpeg[1] != 0xD8 {
        return Err(anyhow::anyhow!("not a valid JPEG file"));
    }

    let mut out = Vec::with_capacity(jpeg.len());
    out.extend_from_slice(&jpeg[..2]);

    let mut pos = 2;
    let mut stripped = false;

    while pos + 4 <= jpeg.len() {
        if jpeg[pos] != 0xFF {
            break;
        }
        let marker = jpeg[pos + 1];
        let seg_len = u16::from_be_bytes([jpeg[pos + 2], jpeg[pos + 3]]) as usize;
        let seg_end = pos + 2 + seg_len;

        if seg_end > jpeg.len() {
            break;
        }

        let payload_start = pos + 4;
        let is_exif_app1 = marker == 0xE1
            && seg_len >= 8
            && &jpeg[payload_start..payload_start.min(payload_start + 6)] == b"Exif\x00\x00";

        if is_exif_app1 && !stripped {
            stripped = true;
        } else {
            out.extend_from_slice(&jpeg[pos..seg_end]);
        }

        pos = seg_end;

        if marker == 0xDA {
            out.extend_from_slice(&jpeg[pos..]);
            return Ok(out);
        }
    }

    if !stripped {
        return Err(anyhow::anyhow!("no EXIF APP1 segment found"));
    }

    out.extend_from_slice(&jpeg[pos..]);
    Ok(out)
}

fn load_file(path: &PathBuf) -> anyhow::Result<ExifData> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let data = if ext == "tif" || ext == "tiff" {
        ExifData::read_tiff(path)?
    } else {
        ExifData::read_jpeg(path)?
    };
    Ok(data)
}

fn save_file(path: &PathBuf, exif: &ExifData) -> anyhow::Result<()> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "tif" || ext == "tiff" {
        let bytes = exif.to_tiff_bytes()?;
        std::fs::write(path, bytes)?;
    } else {
        exif.save_jpeg(path)?;
    }
    Ok(())
}

fn parse_ifd_name(s: &str) -> anyhow::Result<Ifd> {
    match s.to_ascii_lowercase().as_str() {
        "ifd0" | "0" => Ok(Ifd::Ifd0),
        "ifd1" | "1" => Ok(Ifd::Ifd1),
        "exif" => Ok(Ifd::ExifIfd),
        "gps" => Ok(Ifd::GpsIfd),
        "interop" => Ok(Ifd::InteropIfd),
        other => Err(anyhow::anyhow!(
            "unknown IFD '{other}'; valid values: ifd0, ifd1, exif, gps, interop"
        )),
    }
}

fn parse_tag_name(s: &str) -> Tag {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X"))
        && let Ok(code) = u16::from_str_radix(hex, 16) {
            return Tag::from_u16(code);
        }

    let lower = s.to_ascii_lowercase();
    let known: &[(Tag, &str)] = &[
        (Tag::Make, "make"),
        (Tag::Model, "model"),
        (Tag::DateTime, "datetime"),
        (Tag::DateTimeOriginal, "datetimeoriginal"),
        (Tag::DateTimeDigitized, "datetimedigitized"),
        (Tag::ImageDescription, "imagedescription"),
        (Tag::Artist, "artist"),
        (Tag::Copyright, "copyright"),
        (Tag::Software, "software"),
        (Tag::Orientation, "orientation"),
        (Tag::XResolution, "xresolution"),
        (Tag::YResolution, "yresolution"),
        (Tag::ResolutionUnit, "resolutionunit"),
        (Tag::ExposureTime, "exposuretime"),
        (Tag::FNumber, "fnumber"),
        (Tag::IsoSpeedRatings, "isospeedratings"),
        (Tag::FocalLength, "focallength"),
        (Tag::Flash, "flash"),
        (Tag::MeteringMode, "meteringmode"),
        (Tag::ExposureProgram, "exposureprogram"),
        (Tag::WhiteBalance, "whitebalance"),
        (Tag::ExposureMode, "exposuremode"),
        (Tag::PixelXDimension, "pixelxdimension"),
        (Tag::PixelYDimension, "pixelydimension"),
        (Tag::GpsLatitudeRef, "gpslatituderef"),
        (Tag::GpsLatitude, "gpslatitude"),
        (Tag::GpsLongitudeRef, "gpslongituderef"),
        (Tag::GpsLongitude, "gpslongitude"),
        (Tag::GpsAltitudeRef, "gpsaltituderef"),
        (Tag::GpsAltitude, "gpsaltitude"),
        (Tag::GpsDateStamp, "gpsdatestamp"),
        (Tag::UserComment, "usercomment"),
        (Tag::LensMake, "lensmake"),
        (Tag::LensModel, "lensmodel"),
        (Tag::LensSerialNumber, "lensserialnumber"),
        (Tag::BodySerialNumber, "bodyserialnumber"),
        (Tag::CameraOwnerName, "cameraownername"),
        (Tag::ImageUniqueId, "imageuniqueid"),
        (Tag::SceneCaptureType, "scenecapturetype"),
        (Tag::Contrast, "contrast"),
        (Tag::Saturation, "saturation"),
        (Tag::Sharpness, "sharpness"),
    ];

    for (tag, name) in known {
        if *name == lower {
            return *tag;
        }
    }

    Tag::Unknown(0)
}

fn parse_value(type_str: &str, raw: &str) -> anyhow::Result<Value> {
    match type_str.to_ascii_lowercase().as_str() {
        "string" | "ascii" => Ok(Value::Ascii(raw.to_owned())),
        "u16" | "short" => {
            let v: u16 = raw
                .parse()
                .map_err(|_| anyhow::anyhow!("not a valid u16: '{raw}'"))?;
            Ok(Value::Short(vec![v]))
        }
        "u32" | "long" => {
            let v: u32 = raw
                .parse()
                .map_err(|_| anyhow::anyhow!("not a valid u32: '{raw}'"))?;
            Ok(Value::Long(vec![v]))
        }
        "rational" => {
            let (n, d) = split_rational(raw)?;
            let num: u32 = n
                .parse()
                .map_err(|_| anyhow::anyhow!("numerator not a u32"))?;
            let den: u32 = d
                .parse()
                .map_err(|_| anyhow::anyhow!("denominator not a u32"))?;
            Ok(Value::Rational(vec![Rational::new(num, den)]))
        }
        "srational" => {
            let (n, d) = split_rational(raw)?;
            let num: i32 = n
                .parse()
                .map_err(|_| anyhow::anyhow!("numerator not an i32"))?;
            let den: i32 = d
                .parse()
                .map_err(|_| anyhow::anyhow!("denominator not an i32"))?;
            Ok(Value::SRational(vec![SRational::new(num, den)]))
        }
        other => Err(anyhow::anyhow!(
            "unknown type '{other}'; valid: string, u16, u32, rational, srational"
        )),
    }
}

fn split_rational(s: &str) -> anyhow::Result<(&str, &str)> {
    s.split_once('/')
        .ok_or_else(|| anyhow::anyhow!("rational value must be in N/D form, got '{s}'"))
}
