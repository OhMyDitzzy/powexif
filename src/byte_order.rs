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

/// Byte order used when interpreting multi-byte integers in EXIF data.
///
/// EXIF files can be encoded in either big-endian (Motorola) or
/// little-endian (Intel) byte order. The TIFF header at the start
/// of the EXIF block tells us which one to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    /// Big-endian: most significant byte first. Indicated by "MM" in the header.
    BigEndian,
    /// Little-endian: least significant byte first. Indicated by "II" in the header.
    LittleEndian,
}

impl ByteOrder {
    /// Read a u16 from a 2-byte slice using this byte order.
    pub fn read_u16(self, b: &[u8]) -> u16 {
        match self {
            ByteOrder::BigEndian => u16::from_be_bytes([b[0], b[1]]),
            ByteOrder::LittleEndian => u16::from_le_bytes([b[0], b[1]]),
        }
    }

    /// Read a u32 from a 4-byte slice using this byte order.
    pub fn read_u32(self, b: &[u8]) -> u32 {
        match self {
            ByteOrder::BigEndian => u32::from_be_bytes([b[0], b[1], b[2], b[3]]),
            ByteOrder::LittleEndian => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        }
    }

    /// Read an i16 from a 2-byte slice using this byte order.
    pub fn read_i16(self, b: &[u8]) -> i16 {
        match self {
            ByteOrder::BigEndian => i16::from_be_bytes([b[0], b[1]]),
            ByteOrder::LittleEndian => i16::from_le_bytes([b[0], b[1]]),
        }
    }

    /// Read an i32 from a 4-byte slice using this byte order.
    pub fn read_i32(self, b: &[u8]) -> i32 {
        match self {
            ByteOrder::BigEndian => i32::from_be_bytes([b[0], b[1], b[2], b[3]]),
            ByteOrder::LittleEndian => i32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        }
    }

    /// Write a u16 into a 2-byte array using this byte order.
    pub fn write_u16(self, v: u16) -> [u8; 2] {
        match self {
            ByteOrder::BigEndian => v.to_be_bytes(),
            ByteOrder::LittleEndian => v.to_le_bytes(),
        }
    }

    /// Write a u32 into a 4-byte array using this byte order.
    pub fn write_u32(self, v: u32) -> [u8; 4] {
        match self {
            ByteOrder::BigEndian => v.to_be_bytes(),
            ByteOrder::LittleEndian => v.to_le_bytes(),
        }
    }

    /// Write an i16 into a 2-byte array using this byte order.
    pub fn write_i16(self, v: i16) -> [u8; 2] {
        match self {
            ByteOrder::BigEndian => v.to_be_bytes(),
            ByteOrder::LittleEndian => v.to_le_bytes(),
        }
    }

    /// Write an i32 into a 4-byte array using this byte order.
    pub fn write_i32(self, v: i32) -> [u8; 4] {
        match self {
            ByteOrder::BigEndian => v.to_be_bytes(),
            ByteOrder::LittleEndian => v.to_le_bytes(),
        }
    }
}
