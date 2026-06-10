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
use crate::error::{ExifError, Result};
use crate::format::DataFormat;
use crate::tag::Tag;
use crate::value::{Rational, SRational, Value};

/// A single IFD entry: one tag with its decoded value.
#[derive(Debug, Clone)]
pub struct Entry {
    pub tag: Tag,
    pub value: Value,
}

impl Entry {
    pub fn new(tag: Tag, value: Value) -> Self {
        Self { tag, value }
    }

    /// Parse one 12-byte IFD entry from `ifd_block`.
    ///
    /// `base` is the start of the entire TIFF/EXIF block (offset 0 = byte order marker).
    /// Offsets embedded in the entry are relative to `base`.
    pub fn parse(ifd_block: &[u8], base: &[u8], order: ByteOrder) -> Result<Self> {
        if ifd_block.len() < 12 {
            return Err(ExifError::UnexpectedEnd {
                needed: 12,
                available: ifd_block.len(),
            });
        }

        let tag_code = order.read_u16(&ifd_block[0..2]);
        let format_code = order.read_u16(&ifd_block[2..4]);
        let count = order.read_u32(&ifd_block[4..8]) as usize;
        let value_or_offset = &ifd_block[8..12];

        let format = DataFormat::from_u16(format_code)?;
        let total_bytes = count * format.component_size();

        // If the value fits in 4 bytes it is stored inline, otherwise the 4-byte
        // field is an offset into the TIFF block pointing to the actual data.
        let data: &[u8] = if total_bytes <= 4 {
            &value_or_offset[..total_bytes.min(4)]
        } else {
            let offset = order.read_u32(value_or_offset) as usize;
            let end = offset
                .checked_add(total_bytes)
                .ok_or(ExifError::OffsetOutOfBounds {
                    offset,
                    available: base.len(),
                })?;
            if end > base.len() {
                return Err(ExifError::OffsetOutOfBounds {
                    offset: end,
                    available: base.len(),
                });
            }
            &base[offset..end]
        };

        let value = decode_value(format, count, data, order)?;
        Ok(Entry {
            tag: Tag::from_u16(tag_code),
            value,
        })
    }

    /// Serialize this entry into the 12-byte fixed IFD record, appending any
    /// overflow data to `heap`. `heap_offset` is the byte position (relative to
    /// the TIFF base) where `heap` starts, so we can embed the correct absolute
    /// offset when the value exceeds 4 bytes.
    pub fn serialize(&self, order: ByteOrder, heap_offset: usize, heap: &mut Vec<u8>) -> [u8; 12] {
        let mut rec = [0u8; 12];
        let tag_code = self.tag.code();
        let format = self.value.format();
        let count = self.value.component_count() as u32;

        rec[0..2].copy_from_slice(&order.write_u16(tag_code));
        rec[2..4].copy_from_slice(&order.write_u16(format as u16));
        rec[4..8].copy_from_slice(&order.write_u32(count));

        let payload = encode_value(&self.value, order);

        if payload.len() <= 4 {
            // Pad with zeros so the 4-byte slot is always fully written.
            rec[8..8 + payload.len()].copy_from_slice(&payload);
        } else {
            let offset = (heap_offset + heap.len()) as u32;
            rec[8..12].copy_from_slice(&order.write_u32(offset));
            heap.extend_from_slice(&payload);
            // IFD entries must start on a word boundary per TIFF spec.
            if !heap.len().is_multiple_of(2) {
                heap.push(0);
            }
        }

        rec
    }
}

fn decode_value(format: DataFormat, count: usize, data: &[u8], order: ByteOrder) -> Result<Value> {
    let cs = format.component_size();
    let needed = count * cs;
    if data.len() < needed {
        return Err(ExifError::UnexpectedEnd {
            needed,
            available: data.len(),
        });
    }

    Ok(match format {
        DataFormat::Byte => Value::Byte(data[..count].to_vec()),
        DataFormat::Ascii => {
            // Strip the mandatory null terminator before handing to Rust strings.
            let raw = &data[..count];
            let without_null = raw.strip_suffix(&[0]).unwrap_or(raw);
            let s = String::from_utf8(without_null.to_vec())?;
            Value::Ascii(s)
        }
        DataFormat::Short => {
            let v = (0..count).map(|i| order.read_u16(&data[i * 2..])).collect();
            Value::Short(v)
        }
        DataFormat::Long => {
            let v = (0..count).map(|i| order.read_u32(&data[i * 4..])).collect();
            Value::Long(v)
        }
        DataFormat::Rational => {
            let v = (0..count)
                .map(|i| {
                    let off = i * 8;
                    Rational::new(
                        order.read_u32(&data[off..]),
                        order.read_u32(&data[off + 4..]),
                    )
                })
                .collect();
            Value::Rational(v)
        }
        DataFormat::SByte => Value::SByte(data[..count].iter().map(|&b| b as i8).collect()),
        DataFormat::Undefined => Value::Undefined(data[..count].to_vec()),
        DataFormat::SShort => {
            let v = (0..count).map(|i| order.read_i16(&data[i * 2..])).collect();
            Value::SShort(v)
        }
        DataFormat::SLong => {
            let v = (0..count).map(|i| order.read_i32(&data[i * 4..])).collect();
            Value::SLong(v)
        }
        DataFormat::SRational => {
            let v = (0..count)
                .map(|i| {
                    let off = i * 8;
                    SRational::new(
                        order.read_i32(&data[off..]),
                        order.read_i32(&data[off + 4..]),
                    )
                })
                .collect();
            Value::SRational(v)
        }
        DataFormat::Float => {
            let v = (0..count)
                .map(|i| f32::from_bits(order.read_u32(&data[i * 4..])))
                .collect();
            Value::Float(v)
        }
        DataFormat::Double => {
            let v = (0..count)
                .map(|i| {
                    let off = i * 8;
                    let hi = order.read_u32(&data[off..]) as u64;
                    let lo = order.read_u32(&data[off + 4..]) as u64;
                    let bits = match order {
                        ByteOrder::BigEndian => (hi << 32) | lo,
                        ByteOrder::LittleEndian => (lo << 32) | hi,
                    };
                    f64::from_bits(bits)
                })
                .collect();
            Value::Double(v)
        }
    })
}

fn encode_value(value: &Value, order: ByteOrder) -> Vec<u8> {
    match value {
        Value::Byte(v) => v.clone(),
        Value::Ascii(s) => {
            let mut b = s.as_bytes().to_vec();
            b.push(0); // null terminator required by spec
            b
        }
        Value::Short(v) => v.iter().flat_map(|&n| order.write_u16(n)).collect(),
        Value::Long(v) => v.iter().flat_map(|&n| order.write_u32(n)).collect(),
        Value::Rational(v) => v
            .iter()
            .flat_map(|r| {
                let mut b = order.write_u32(r.numerator).to_vec();
                b.extend_from_slice(&order.write_u32(r.denominator));
                b
            })
            .collect(),
        Value::SByte(v) => v.iter().map(|&b| b as u8).collect(),
        Value::Undefined(v) => v.clone(),
        Value::SShort(v) => v.iter().flat_map(|&n| order.write_i16(n)).collect(),
        Value::SLong(v) => v.iter().flat_map(|&n| order.write_i32(n)).collect(),
        Value::SRational(v) => v
            .iter()
            .flat_map(|r| {
                let mut b = order.write_i32(r.numerator).to_vec();
                b.extend_from_slice(&order.write_i32(r.denominator));
                b
            })
            .collect(),
        Value::Float(v) => v
            .iter()
            .flat_map(|&f| order.write_u32(f.to_bits()))
            .collect(),
        Value::Double(v) => v
            .iter()
            .flat_map(|&d| {
                let bits = d.to_bits();
                match order {
                    ByteOrder::BigEndian => {
                        let hi = (bits >> 32) as u32;
                        let lo = bits as u32;
                        let mut b = order.write_u32(hi).to_vec();
                        b.extend_from_slice(&order.write_u32(lo));
                        b
                    }
                    ByteOrder::LittleEndian => {
                        let lo = bits as u32;
                        let hi = (bits >> 32) as u32;
                        let mut b = order.write_u32(lo).to_vec();
                        b.extend_from_slice(&order.write_u32(hi));
                        b
                    }
                }
            })
            .collect(),
    }
}
