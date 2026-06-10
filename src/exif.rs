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

use std::collections::HashMap;
use std::io::{self, Read};
use std::path::Path;

use crate::byte_order::ByteOrder;
use crate::entry::Entry;
use crate::error::{ExifError, Result};
use crate::ifd::Ifd;
use crate::makernote::MakerNote;
use crate::tag::Tag;
use crate::value::Value;

/// The "Exif\0\0" magic bytes that follow the APP1 marker length in a JPEG.
const EXIF_HEADER: &[u8] = b"Exif\x00\x00";

/// JPEG markers used while scanning for the APP1 segment.
const MARKER_SOI: u8 = 0xD8;
const MARKER_APP1: u8 = 0xE1;
const MARKER_SOS: u8 = 0xDA;

/// TIFF magic number that follows the byte-order marker.
const TIFF_MAGIC: u16 = 0x002A;

/// Complete EXIF data parsed from a JPEG or raw TIFF buffer.
///
/// Entries are stored per-IFD so that callers can enumerate or modify specific
/// sections without re-parsing. The raw JPEG bytes before and after the APP1
/// segment are kept so that `save_jpeg` can reconstruct a valid file.
#[derive(Debug, Clone)]
pub struct ExifData {
    /// Byte order declared in the TIFF header inside the EXIF block.
    pub byte_order: ByteOrder,

    /// Entries keyed by the IFD they belong to.
    pub ifds: HashMap<Ifd, Vec<Entry>>,

    /// Parsed MakerNote, present when the MakerNote tag exists and the vendor
    /// was recognized.
    pub maker_note: Option<MakerNote>,

    /// Raw JPEG bytes that precede the APP1 segment (SOI + any earlier markers).
    /// Empty when parsing a raw TIFF buffer rather than a JPEG.
    jpeg_prefix: Vec<u8>,

    /// Raw JPEG bytes that follow the APP1 segment.
    /// Empty when parsing a raw TIFF buffer rather than a JPEG.
    jpeg_suffix: Vec<u8>,
}

impl ExifData {
    /// Parse EXIF data from a JPEG file on disk.
    pub fn read_jpeg<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        Self::from_jpeg_bytes(&bytes)
    }

    /// Parse EXIF data from a raw TIFF file on disk.
    pub fn read_tiff<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        Self::from_tiff_bytes(&bytes)
    }

    /// Parse EXIF data from in-memory JPEG bytes.
    ///
    /// The function locates the APP1 segment, verifies the "Exif\0\0" header,
    /// parses the embedded TIFF block, and stores the surrounding JPEG bytes so
    /// the file can be reconstructed with `save_jpeg` after edits.
    pub fn from_jpeg_bytes(jpeg: &[u8]) -> Result<Self> {
        if jpeg.len() < 2 || jpeg[0] != 0xFF || jpeg[1] != MARKER_SOI {
            return Err(ExifError::InvalidHeader);
        }

        let (prefix, tiff_block, suffix) = extract_app1_tiff(jpeg)?;
        let mut data = parse_tiff_block(tiff_block)?;
        data.jpeg_prefix = prefix;
        data.jpeg_suffix = suffix;
        Ok(data)
    }

    /// Parse EXIF data from a raw TIFF byte buffer.
    pub fn from_tiff_bytes(tiff: &[u8]) -> Result<Self> {
        parse_tiff_block(tiff)
    }

    /// Save the modified EXIF data back into a JPEG file.
    ///
    /// The serialized TIFF block is wrapped in a new APP1 segment, then
    /// concatenated with the original JPEG prefix (SOI + earlier markers)
    /// and the original JPEG suffix (everything after the old APP1).
    pub fn save_jpeg<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let bytes = self.to_jpeg_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Serialize the modified EXIF data as JPEG bytes.
    pub fn to_jpeg_bytes(&self) -> Result<Vec<u8>> {
        let tiff = self.to_tiff_bytes()?;

        // APP1 payload = "Exif\0\0" + tiff block.
        let payload_len = EXIF_HEADER.len() + tiff.len();
        // The 2-byte length field counts itself.
        let segment_len = (payload_len + 2) as u16;

        let mut out =
            Vec::with_capacity(self.jpeg_prefix.len() + 4 + payload_len + self.jpeg_suffix.len());
        out.extend_from_slice(&self.jpeg_prefix);
        out.push(0xFF);
        out.push(MARKER_APP1);
        out.push((segment_len >> 8) as u8);
        out.push(segment_len as u8);
        out.extend_from_slice(EXIF_HEADER);
        out.extend_from_slice(&tiff);
        out.extend_from_slice(&self.jpeg_suffix);
        Ok(out)
    }

    /// Serialize the EXIF data as a self-contained TIFF block.
    ///
    /// The layout is: byte-order marker (2 bytes) + 0x002A (2 bytes) +
    /// IFD0 offset (4 bytes) + IFD0 entries + heap data.
    /// IFD1 (thumbnail) is appended after IFD0 if present.
    pub fn to_tiff_bytes(&self) -> Result<Vec<u8>> {
        serialize_tiff(self)
    }

    /// Look up an entry in the specified IFD by tag.
    pub fn get(&self, ifd: Ifd, tag: Tag) -> Option<&Entry> {
        self.ifds.get(&ifd)?.iter().find(|e| e.tag == tag)
    }

    /// Look up a mutable reference to an entry in the specified IFD.
    pub fn get_mut(&mut self, ifd: Ifd, tag: Tag) -> Option<&mut Entry> {
        self.ifds.get_mut(&ifd)?.iter_mut().find(|e| e.tag == tag)
    }

    /// Insert or replace an entry in the specified IFD.
    pub fn set(&mut self, ifd: Ifd, tag: Tag, value: Value) {
        let entries = self.ifds.entry(ifd).or_default();
        if let Some(existing) = entries.iter_mut().find(|e| e.tag == tag) {
            existing.value = value;
        } else {
            entries.push(Entry::new(tag, value));
        }
    }

    /// Remove a tag from the specified IFD. Returns true if it was present.
    pub fn remove(&mut self, ifd: Ifd, tag: Tag) -> bool {
        if let Some(entries) = self.ifds.get_mut(&ifd) {
            let before = entries.len();
            entries.retain(|e| e.tag != tag);
            return entries.len() < before;
        }
        false
    }

    /// Return all entries in the specified IFD, or an empty slice if absent.
    pub fn entries(&self, ifd: Ifd) -> &[Entry] {
        self.ifds.get(&ifd).map(Vec::as_slice).unwrap_or_default()
    }

    /// Iterate over every (IFD, Entry) pair across all IFDs.
    pub fn all_entries(&self) -> impl Iterator<Item = (Ifd, &Entry)> {
        let order = [
            Ifd::Ifd0,
            Ifd::Ifd1,
            Ifd::ExifIfd,
            Ifd::GpsIfd,
            Ifd::InteropIfd,
        ];
        order.into_iter().flat_map(|ifd| {
            self.ifds
                .get(&ifd)
                .into_iter()
                .flat_map(move |v| v.iter().map(move |e| (ifd, e)))
        })
    }
}

/// Locate the APP1 segment in a JPEG buffer and split it into three parts:
/// the prefix (SOI through the byte before APP1), the raw TIFF block inside
/// APP1, and the suffix (everything after APP1).
fn extract_app1_tiff(jpeg: &[u8]) -> Result<(Vec<u8>, &[u8], Vec<u8>)> {
    let mut pos = 2; // skip SOI (0xFF 0xD8)

    while pos + 4 <= jpeg.len() {
        if jpeg[pos] != 0xFF {
            return Err(ExifError::InvalidHeader);
        }
        let marker = jpeg[pos + 1];

        // SOS marks the start of image data; no APP1 found before it.
        if marker == MARKER_SOS {
            return Err(ExifError::NoExifSegment);
        }

        // Segment length is big-endian and includes the 2-byte length field itself.
        if pos + 4 > jpeg.len() {
            break;
        }
        let seg_len = u16::from_be_bytes([jpeg[pos + 2], jpeg[pos + 3]]) as usize;
        let seg_payload_start = pos + 4;
        let seg_payload_end = pos + 2 + seg_len;

        if seg_payload_end > jpeg.len() {
            return Err(ExifError::OffsetOutOfBounds {
                offset: seg_payload_end,
                available: jpeg.len(),
            });
        }

        if marker == MARKER_APP1 {
            let payload = &jpeg[seg_payload_start..seg_payload_end];
            if payload.len() >= 6 && &payload[..6] == EXIF_HEADER {
                // The TIFF block starts right after the "Exif\0\0" header.
                let tiff = &payload[6..];
                let prefix = jpeg[..pos].to_vec();
                let suffix = jpeg[seg_payload_end..].to_vec();
                return Ok((prefix, tiff, suffix));
            }
        }

        pos = seg_payload_end;
    }

    Err(ExifError::NoExifSegment)
}

/// Parse a raw TIFF block (starting with the byte-order marker) into ExifData.
fn parse_tiff_block(tiff: &[u8]) -> Result<ExifData> {
    if tiff.len() < 8 {
        return Err(ExifError::UnexpectedEnd {
            needed: 8,
            available: tiff.len(),
        });
    }

    let order = match &tiff[0..2] {
        b"II" => ByteOrder::LittleEndian,
        b"MM" => ByteOrder::BigEndian,
        _ => return Err(ExifError::InvalidByteOrder),
    };

    let magic = order.read_u16(&tiff[2..4]);
    if magic != TIFF_MAGIC {
        return Err(ExifError::InvalidHeader);
    }

    let ifd0_offset = order.read_u32(&tiff[4..8]) as usize;

    let mut data = ExifData {
        byte_order: order,
        ifds: HashMap::new(),
        maker_note: None,
        jpeg_prefix: Vec::new(),
        jpeg_suffix: Vec::new(),
    };

    // Parse IFD0.
    let next_ifd_offset = parse_ifd_into(&mut data, tiff, ifd0_offset, order, Ifd::Ifd0)?;

    // IFD0 may contain pointers to sub-IFDs. Follow them now that the IFD0
    // entries are loaded.
    follow_sub_ifds(&mut data, tiff, order);

    // IFD1 (thumbnail) is linked by a 4-byte offset after the last IFD0 entry.
    if next_ifd_offset != 0 {
        parse_ifd_into(&mut data, tiff, next_ifd_offset, order, Ifd::Ifd1)?;
    }

    // Parse MakerNote if present.
    let make_str = data
        .ifds
        .get(&Ifd::Ifd0)
        .and_then(|entries| entries.iter().find(|e| e.tag == Tag::Make))
        .and_then(|e| e.value.as_str())
        .map(str::trim)
        .map(str::to_owned);

    let mn_raw = data
        .ifds
        .get(&Ifd::ExifIfd)
        .and_then(|entries| entries.iter().find(|e| e.tag == Tag::MakerNote))
        .and_then(|e| {
            if let crate::value::Value::Undefined(ref bytes) = e.value {
                Some(bytes.clone())
            } else {
                None
            }
        });

    if let Some(raw) = mn_raw {
        let mn = MakerNote::parse(&raw, make_str.as_deref(), order)?;
        if !mn.entries.is_empty() {
            data.maker_note = Some(mn);
        }
    }

    Ok(data)
}

/// Parse all IFD entries starting at `ifd_offset` in `tiff` and store them
/// in `data.ifds[ifd]`. Returns the offset of the next linked IFD (0 = none).
fn parse_ifd_into(
    data: &mut ExifData,
    tiff: &[u8],
    ifd_offset: usize,
    order: ByteOrder,
    ifd: Ifd,
) -> Result<usize> {
    if ifd_offset + 2 > tiff.len() {
        return Ok(0);
    }

    let count = order.read_u16(&tiff[ifd_offset..ifd_offset + 2]) as usize;
    let entries_end = ifd_offset + 2 + count * 12;

    if entries_end + 4 > tiff.len() {
        // File is truncated; load as many entries as fit.
    }

    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let start = ifd_offset + 2 + i * 12;
        let end = start + 12;
        if end > tiff.len() {
            break;
        }
        match Entry::parse(&tiff[start..end], tiff, order) {
            Ok(e) => entries.push(e),
            Err(_) => continue, // skip malformed entries rather than failing
        }
    }

    data.ifds.insert(ifd, entries);

    // The 4 bytes after the last entry record contain the next IFD offset.
    if entries_end + 4 <= tiff.len() {
        Ok(order.read_u32(&tiff[entries_end..entries_end + 4]) as usize)
    } else {
        Ok(0)
    }
}

/// Scan IFD0 for pointer tags and parse the sub-IFDs they reference.
fn follow_sub_ifds(data: &mut ExifData, tiff: &[u8], order: ByteOrder) {
    let pointers: Vec<(Tag, usize)> = data
        .ifds
        .get(&Ifd::Ifd0)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|e| match e.tag {
                    Tag::ExifIfdPointer
                    | Tag::GpsInfoIfdPointer
                    | Tag::InteroperabilityIfdPointer => {
                        e.value.as_u32().map(|offset| (e.tag, offset as usize))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();

    for (tag, offset) in pointers {
        let target = match tag {
            Tag::ExifIfdPointer => Ifd::ExifIfd,
            Tag::GpsInfoIfdPointer => Ifd::GpsIfd,
            Tag::InteroperabilityIfdPointer => Ifd::InteropIfd,
            _ => continue,
        };
        let _ = parse_ifd_into(data, tiff, offset, order, target);
    }

    // ExifIFD may itself contain an Interoperability pointer.
    let interop_ptr = data
        .ifds
        .get(&Ifd::ExifIfd)
        .and_then(|entries| {
            entries
                .iter()
                .find(|e| e.tag == Tag::InteroperabilityIfdPointer)
        })
        .and_then(|e| e.value.as_u32())
        .map(|v| v as usize);

    if let Some(offset) = interop_ptr
        && !data.ifds.contains_key(&Ifd::InteropIfd) {
            let _ = parse_ifd_into(data, tiff, offset, order, Ifd::InteropIfd);
        }
}

/// Serialize ExifData into a self-contained TIFF block.
///
/// Layout: byte-order (2) + 0x002A (2) + IFD0-offset (4) + IFD0-body +
/// sub-IFD bodies + IFD1-body. The heap (values wider than 4 bytes) for each
/// IFD is packed directly after that IFD's entry records.
fn serialize_tiff(data: &ExifData) -> Result<Vec<u8>> {
    let order = data.byte_order;

    // The TIFF header occupies the first 8 bytes, and IFD0 starts right after.
    let ifd0_offset: u32 = 8;

    let mut tiff: Vec<u8> = Vec::new();

    // Write byte-order marker and TIFF magic.
    match order {
        ByteOrder::LittleEndian => tiff.extend_from_slice(b"II"),
        ByteOrder::BigEndian => tiff.extend_from_slice(b"MM"),
    }
    tiff.extend_from_slice(&order.write_u16(TIFF_MAGIC));
    tiff.extend_from_slice(&order.write_u32(ifd0_offset));

    // We need to compute sub-IFD offsets before writing IFD0, because IFD0
    // entries for ExifIfdPointer etc. must already contain the correct offsets.
    // TODO: build each IFD body in memory, then assemble in order.

    let ifd_order = [Ifd::ExifIfd, Ifd::GpsIfd, Ifd::InteropIfd, Ifd::Ifd1];

    // Compute the size of IFD0 so we know where sub-IFDs begin.
    let ifd0_entries_raw = data
        .ifds
        .get(&Ifd::Ifd0)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let ifd0_entry_count = ifd0_entries_raw.len();
    // IFD body = 2 (count) + count*12 (records) + 4 (next-IFD pointer) + heap.
    let ifd0_records_size = 2 + ifd0_entry_count * 12 + 4;

    // Build sub-IFD bodies first so we know their sizes and can calculate offsets.
    let _sub_ifd_bodies: Vec<(Ifd, Vec<u8>, Vec<u8>)> = Vec::new(); // (ifd, records, heap)
    let _running_offset = 8 + ifd0_records_size; // will be updated after ifd0 heap

    // Placeholder: we will determine heap sizes after a dry-run of IFD0 serialization.
    // Simpler approach: serialize IFD0 without pointer-updating entries first to measure
    // IFD0 heap size, then compute sub-IFD offsets, then do a real serialization pass.

    // Serialize each sub-IFD body (records + heap) independently.
    let mut bodies: HashMap<Ifd, (Vec<u8>, Vec<u8>)> = HashMap::new();
    for &ifd in &ifd_order {
        if let Some(entries) = data.ifds.get(&ifd) {
            if entries.is_empty() {
                continue;
            }
            let (rec, heap) = serialize_ifd_body(entries, order, 0 /* placeholder offset */);
            bodies.insert(ifd, (rec, heap));
        }
    }

    // Compute actual byte offsets for each sub-IFD in the final stream.
    // IFD0 starts at byte 8. After IFD0 we lay out sub-IFDs in ifd_order order.
    // We need IFD0's heap size to know where sub-IFDs start, but IFD0's heap
    // depends on no large pointer values, so we can measure it now.
    let (ifd0_rec_placeholder, ifd0_heap) =
        serialize_ifd_body(ifd0_entries_raw, order, 8 + ifd0_records_size);

    let ifd0_total = ifd0_rec_placeholder.len() + ifd0_heap.len();
    let mut next_offset = 8 + ifd0_total;

    // Assign offsets to sub-IFDs and reserialize them with the correct heap_offset.
    let mut sub_offsets: HashMap<Ifd, usize> = HashMap::new();
    let mut ordered_bodies: Vec<(Ifd, Vec<u8>, Vec<u8>)> = Vec::new();

    for &ifd in &ifd_order {
        if !bodies.contains_key(&ifd) {
            continue;
        }
        sub_offsets.insert(ifd, next_offset);
        let entries = data.ifds.get(&ifd).unwrap();
        let entry_count = entries.len();
        let records_region = 2 + entry_count * 12 + 4;
        let heap_offset = next_offset + records_region;
        let (rec, heap) = serialize_ifd_body(entries, order, heap_offset);
        next_offset += rec.len() + heap.len();
        ordered_bodies.push((ifd, rec, heap));
    }

    // Now serialize IFD0 with the correct sub-IFD pointer values injected.
    let ifd0_entries_updated: Vec<Entry> = ifd0_entries_raw
        .iter()
        .map(|e| {
            let patched_value = match e.tag {
                Tag::ExifIfdPointer => sub_offsets
                    .get(&Ifd::ExifIfd)
                    .map(|&off| Value::Long(vec![off as u32])),
                Tag::GpsInfoIfdPointer => sub_offsets
                    .get(&Ifd::GpsIfd)
                    .map(|&off| Value::Long(vec![off as u32])),
                Tag::InteroperabilityIfdPointer => sub_offsets
                    .get(&Ifd::InteropIfd)
                    .map(|&off| Value::Long(vec![off as u32])),
                _ => None,
            };
            if let Some(v) = patched_value {
                Entry::new(e.tag, v)
            } else {
                e.clone()
            }
        })
        .collect();

    let ifd0_heap_start = 8 + ifd0_records_size;
    let (ifd0_rec, ifd0_heap) = serialize_ifd_body(&ifd0_entries_updated, order, ifd0_heap_start);

    // IFD1 offset: if IFD1 was included, it is the last body; find its offset.
    let ifd1_offset = sub_offsets.get(&Ifd::Ifd1).copied().unwrap_or(0);

    // Patch the next-IFD pointer at the end of IFD0 records.
    // The next-IFD pointer is the last 4 bytes of the records region.
    let mut ifd0_rec_mut = ifd0_rec;
    let ptr_pos = ifd0_rec_mut.len() - 4;
    ifd0_rec_mut[ptr_pos..ptr_pos + 4].copy_from_slice(&order.write_u32(ifd1_offset as u32));

    // Assemble the final TIFF buffer.
    tiff.extend_from_slice(&ifd0_rec_mut);
    tiff.extend_from_slice(&ifd0_heap);

    for (_ifd, rec, heap) in &ordered_bodies {
        tiff.extend_from_slice(rec);
        tiff.extend_from_slice(heap);
    }

    Ok(tiff)
}

/// Serialize a list of entries into IFD records and a heap.
///
/// Returns `(records, heap)` where records is the count (2 bytes) + entry
/// records (n * 12 bytes) + next-IFD pointer (4 bytes, zeroed here — callers
/// patch this after computing IFD1's position). The heap contains overflow
/// value data for entries whose serialized value exceeds 4 bytes.
///
/// `heap_base_offset` is the absolute byte position in the final TIFF stream
/// where `heap` will be placed, so that offset pointers embedded in the
/// records are correct.
fn serialize_ifd_body(
    entries: &[Entry],
    order: ByteOrder,
    heap_base_offset: usize,
) -> (Vec<u8>, Vec<u8>) {
    let count = entries.len() as u16;
    let mut records: Vec<u8> = Vec::with_capacity(2 + entries.len() * 12 + 4);
    let mut heap: Vec<u8> = Vec::new();

    records.extend_from_slice(&order.write_u16(count));

    for entry in entries {
        let rec = entry.serialize(order, heap_base_offset, &mut heap);
        records.extend_from_slice(&rec);
    }

    // Next-IFD offset placeholder (zeroed; caller patches if needed).
    records.extend_from_slice(&order.write_u32(0));

    (records, heap)
}

/// Read all bytes from a type implementing `Read`.
pub fn read_all<R: Read>(mut reader: R) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}
