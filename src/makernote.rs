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

use crate::byte_order::ByteOrder;
use crate::entry::Entry;
use crate::error::Result;
use crate::tag::Tag;
use crate::value::Value;

/// Identifies the camera vendor and format variant for a MakerNote block.
///
/// Detection follows the same signature logic as libexif: magic-byte prefixes
/// are checked first (Fuji, Olympus, Sanyo, Epson, Nikon v2), then a Make-tag
/// string match is used for vendors that have no in-band magic (Canon). The
/// enum variants that carry no data represent formats whose MakerNote block is
/// a plain IFD with no header prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MakerNoteKind {
    Canon,
    Casio,
    Fuji,
    /// Epson uses the same block layout as Olympus v1.
    Epson,
    /// Nikon v1: bare IFD at a fixed offset; no embedded TIFF header.
    NikonV1,
    /// Nikon v2: contains a self-contained TIFF block with its own byte-order marker.
    NikonV2,
    /// Nikon v0: older variant identified by a 0x001b leading byte pair.
    NikonV0,
    OlympusV1,
    OlympusV2,
    PentaxV1,
    PentaxV2,
    PentaxV3,
    Sanyo,
    Unknown,
}

/// A parsed MakerNote: the vendor variant plus the decoded tag entries.
#[derive(Debug, Clone)]
pub struct MakerNote {
    pub kind: MakerNoteKind,
    pub entries: Vec<Entry>,
    /// Byte order used inside the MakerNote block. Usually matches the outer
    /// EXIF byte order, but Nikon v2 embeds its own TIFF header with its own marker.
    pub byte_order: ByteOrder,
}

impl MakerNote {
    /// Detect and parse the raw MakerNote byte slice.
    ///
    /// `make_tag` is the value of the EXIF Make tag (e.g. "Canon") and is used
    /// to disambiguate vendors whose MakerNote carries no magic prefix.
    /// `outer_order` is the byte order of the surrounding EXIF block.
    pub fn parse(data: &[u8], make_tag: Option<&str>, outer_order: ByteOrder) -> Result<Self> {
        let kind = detect_kind(data, make_tag);
        parse_by_kind(data, kind, outer_order)
    }

    /// Return all entries as an iterator of (tag, value) references.
    pub fn iter(&self) -> impl Iterator<Item = (&Tag, &Value)> {
        self.entries.iter().map(|e| (&e.tag, &e.value))
    }
}

/// Detect the MakerNote variant from the raw byte slice and optional Make string.
fn detect_kind(data: &[u8], make_tag: Option<&str>) -> MakerNoteKind {
    if data.len() >= 8 && &data[..8] == b"FUJIFILM" {
        return MakerNoteKind::Fuji;
    }

    if data.len() >= 8 && &data[..8] == b"OLYMPUS\0" {
        return MakerNoteKind::OlympusV2;
    }
    if data.len() >= 6 && &data[..5] == b"OLYMP" {
        return MakerNoteKind::OlympusV1;
    }
    if data.len() >= 6 && &data[..5] == b"SANYO" {
        return MakerNoteKind::Sanyo;
    }
    if data.len() >= 6 && &data[..5] == b"EPSON" {
        return MakerNoteKind::Epson;
    }

    if data.len() >= 6 && &data[..5] == b"Nikon" {
        return match data.get(6).copied() {
            Some(1) => MakerNoteKind::NikonV1,
            Some(2) => MakerNoteKind::NikonV2,
            _ => MakerNoteKind::Unknown,
        };
    }

    // Nikon v0: older bodies write two bytes 0x00 0x1B with no text header.
    // Confirm with the Make tag to avoid false positives.
    if data.len() >= 2 && data[0] == 0x00 && data[1] == 0x1b {
        let make = make_tag.unwrap_or("");
        if make.eq_ignore_ascii_case("nikon") {
            return MakerNoteKind::NikonV0;
        }
    }

    // Pentax v3 and Casio v2 share the "AOC" prefix; the following two bytes
    // distinguish them: "II"/"MM" means Pentax v3, anything else is Pentax v2
    // (which uses Casio v2 tag definitions).
    if data.len() >= 6 && &data[..3] == b"AOC" {
        let b4 = data[4];
        let b5 = data[5];
        if (b4 == b'I' && b5 == b'I') || (b4 == b'M' && b5 == b'M') {
            return MakerNoteKind::PentaxV3;
        }
        return MakerNoteKind::PentaxV2;
    }

    // Casio v2 uses a "QVC" prefix.
    if data.len() >= 4 && &data[..3] == b"QVC" {
        return MakerNoteKind::Casio;
    }

    // Pentax v1: bare IFD, first two bytes are entry count in big-endian.
    if data.len() >= 2 && data[0] == 0x00 && data[1] == 0x1b {
        let make = make_tag.unwrap_or("");
        if make.eq_ignore_ascii_case("pentax") || make.eq_ignore_ascii_case("asahi") {
            return MakerNoteKind::PentaxV1;
        }
    }

    // Canon has no magic prefix; detection is purely by Make string.
    if let Some(make) = make_tag
        && make.eq_ignore_ascii_case("canon") {
            return MakerNoteKind::Canon;
        }

    MakerNoteKind::Unknown
}

/// Parse entries from the MakerNote block according to the detected variant.
fn parse_by_kind(data: &[u8], kind: MakerNoteKind, outer_order: ByteOrder) -> Result<MakerNote> {
    match kind {
        MakerNoteKind::Fuji => parse_fuji(data),

        MakerNoteKind::OlympusV1 | MakerNoteKind::Sanyo | MakerNoteKind::Epson => {
            parse_olympus_v1(data, outer_order, kind)
        }

        MakerNoteKind::OlympusV2 => parse_olympus_v2(data),

        MakerNoteKind::NikonV2 => parse_nikon_v2(data),

        MakerNoteKind::NikonV1 => parse_nikon_v1(data, outer_order),

        // Nikon v0 is a bare IFD with no header, same as Pentax v1.
        MakerNoteKind::NikonV0 => parse_bare_ifd(data, 0, outer_order, kind),

        MakerNoteKind::Canon => parse_bare_ifd(data, 0, outer_order, kind),

        MakerNoteKind::PentaxV1 => parse_bare_ifd(data, 0, outer_order, kind),

        // Pentax v2/v3 and Casio v2 skip the 4-byte magic + 2-byte padding.
        MakerNoteKind::PentaxV2 | MakerNoteKind::PentaxV3 | MakerNoteKind::Casio => {
            parse_bare_ifd(data, 6, outer_order, kind)
        }

        MakerNoteKind::Unknown => Ok(MakerNote {
            kind: MakerNoteKind::Unknown,
            entries: Vec::new(),
            byte_order: outer_order,
        }),
    }
}

/// Fuji MakerNote layout: "FUJIFILM" (8 bytes) + u32-LE offset to IFD.
///
/// The IFD offsets inside are relative to the start of the MakerNote block,
/// not the TIFF base. Fuji always uses little-endian regardless of the outer
/// EXIF byte order.
fn parse_fuji(data: &[u8]) -> Result<MakerNote> {
    let order = ByteOrder::LittleEndian;
    if data.len() < 12 {
        return Ok(MakerNote {
            kind: MakerNoteKind::Fuji,
            entries: Vec::new(),
            byte_order: order,
        });
    }
    let ifd_offset = order.read_u32(&data[8..12]) as usize;
    let entries = parse_ifd_entries(data, ifd_offset, data, order)?;
    Ok(MakerNote {
        kind: MakerNoteKind::Fuji,
        entries,
        byte_order: order,
    })
}

/// Olympus v1 / Sanyo / Epson layout: 6-byte magic, 2-byte version, IFD entries.
///
/// Byte order is encoded at offset 6: 0x01 in the first byte = little-endian,
/// 0x01 in the second byte = big-endian.
fn parse_olympus_v1(data: &[u8], outer_order: ByteOrder, kind: MakerNoteKind) -> Result<MakerNote> {
    let order = if data.len() >= 8 {
        if data[6] == 1 {
            ByteOrder::LittleEndian
        } else if data[7] == 1 {
            ByteOrder::BigEndian
        } else {
            outer_order
        }
    } else {
        outer_order
    };
    let ifd_offset = 8;
    let entries = parse_ifd_entries(data, ifd_offset, data, order)?;
    Ok(MakerNote {
        kind,
        entries,
        byte_order: order,
    })
}

/// Olympus v2 layout: "OLYMPUS\0" (8 bytes) + "II"/"MM" (2 bytes) + u16 0x2A + u32 IFD offset.
fn parse_olympus_v2(data: &[u8]) -> Result<MakerNote> {
    if data.len() < 16 {
        return Ok(MakerNote {
            kind: MakerNoteKind::OlympusV2,
            entries: Vec::new(),
            byte_order: ByteOrder::LittleEndian,
        });
    }
    let order = if &data[8..10] == b"II" {
        ByteOrder::LittleEndian
    } else {
        ByteOrder::BigEndian
    };
    let ifd_offset = order.read_u32(&data[12..16]) as usize;
    let entries = parse_ifd_entries(data, ifd_offset, data, order)?;
    Ok(MakerNote {
        kind: MakerNoteKind::OlympusV2,
        entries,
        byte_order: order,
    })
}

/// Nikon v2 layout: "Nikon\0" (6) + version byte (1) + unknown byte (1) +
/// embedded TIFF block starting at offset 10 (byte-order marker, 0x002A, IFD offset).
///
/// The embedded TIFF is self-contained; all internal offsets are relative to
/// the start of that inner TIFF, not to the outer EXIF block.
fn parse_nikon_v2(data: &[u8]) -> Result<MakerNote> {
    // The embedded TIFF starts at byte 10.
    let tiff_start = 10;
    if data.len() < tiff_start + 8 {
        return Ok(MakerNote {
            kind: MakerNoteKind::NikonV2,
            entries: Vec::new(),
            byte_order: ByteOrder::LittleEndian,
        });
    }
    let tiff = &data[tiff_start..];
    let order = if &tiff[0..2] == b"II" {
        ByteOrder::LittleEndian
    } else {
        ByteOrder::BigEndian
    };
    let ifd_offset = order.read_u32(&tiff[4..8]) as usize;
    let entries = parse_ifd_entries(tiff, ifd_offset, tiff, order)?;
    Ok(MakerNote {
        kind: MakerNoteKind::NikonV2,
        entries,
        byte_order: order,
    })
}

/// Nikon v1 layout: "Nikon\0" (6 bytes) + version (1 byte) + IFD entries.
///
/// Some D1H/D1X bodies omit the extra 3 bytes before the entry count, so we
/// try to detect where the IFD actually starts by checking which offset gives
/// a plausible entry count.
fn parse_nikon_v1(data: &[u8], order: ByteOrder) -> Result<MakerNote> {
    // Try at offset 8 first (version byte + 0x00 + 0x01 + 0x00 preamble).
    // Fall back to offset 7 if the count at 8 looks invalid.
    let ifd_offset = if data.len() >= 10 && order.read_u16(&data[8..10]) < 256 {
        8
    } else {
        7
    };
    let entries = parse_ifd_entries(data, ifd_offset, data, order)?;
    Ok(MakerNote {
        kind: MakerNoteKind::NikonV1,
        entries,
        byte_order: order,
    })
}

/// Parse a plain IFD at `ifd_offset` within `data`.
///
/// Used for Canon, Pentax, Casio, and Nikon v0, all of which have no embedded
/// TIFF header; offsets are relative to the start of the MakerNote data slice.
fn parse_bare_ifd(
    data: &[u8],
    ifd_offset: usize,
    order: ByteOrder,
    kind: MakerNoteKind,
) -> Result<MakerNote> {
    let entries = parse_ifd_entries(data, ifd_offset, data, order)?;
    Ok(MakerNote {
        kind,
        entries,
        byte_order: order,
    })
}

/// Read all entries from an IFD starting at `ifd_offset` within `ifd_block`.
///
/// `value_base` is the slice that embedded offsets (in the value-or-offset field)
/// are relative to. For most formats this is the same as `ifd_block`, but Nikon
/// v2 uses an inner TIFF, so `value_base` is that inner buffer.
fn parse_ifd_entries(
    ifd_block: &[u8],
    ifd_offset: usize,
    value_base: &[u8],
    order: ByteOrder,
) -> Result<Vec<Entry>> {
    if ifd_offset + 2 > ifd_block.len() {
        return Ok(Vec::new());
    }

    let count = order.read_u16(&ifd_block[ifd_offset..ifd_offset + 2]) as usize;
    let mut entries = Vec::with_capacity(count);
    let records_start = ifd_offset + 2;

    for i in 0..count {
        let entry_start = records_start + i * 12;
        let entry_end = entry_start + 12;
        if entry_end > ifd_block.len() {
            break;
        }
        let slice = &ifd_block[entry_start..entry_end];
        match Entry::parse(slice, value_base, order) {
            Ok(e) => entries.push(e),
            // Skip entries whose format is unrecognized rather than aborting the
            // whole parse, since MakerNote blocks frequently contain undocumented tags.
            Err(_) => continue,
        }
    }

    Ok(entries)
}
