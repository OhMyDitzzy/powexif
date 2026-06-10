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

use crate::error::{ExifError, Result};

/// The type of data stored in a tag value, as defined by the EXIF/TIFF spec.
///
/// Each tag entry in an IFD declares the format of its value using one of these
/// codes. The format determines how many bytes each component occupies and how
/// to interpret the raw bytes (e.g., as a signed integer vs an unsigned rational).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum DataFormat {
    /// Unsigned 8-bit integer. 1 byte per component.
    Byte = 1,
    /// ASCII text string, null-terminated. 1 byte per character.
    Ascii = 2,
    /// Unsigned 16-bit integer. 2 bytes per component.
    Short = 3,
    /// Unsigned 32-bit integer. 4 bytes per component.
    Long = 4,
    /// Unsigned rational: two u32 values (numerator / denominator). 8 bytes per component.
    Rational = 5,
    /// Signed 8-bit integer. 1 byte per component.
    SByte = 6,
    /// Uninterpreted byte sequence. 1 byte per component.
    Undefined = 7,
    /// Signed 16-bit integer. 2 bytes per component.
    SShort = 8,
    /// Signed 32-bit integer. 4 bytes per component.
    SLong = 9,
    /// Signed rational: two i32 values (numerator / denominator). 8 bytes per component.
    SRational = 10,
    /// 32-bit IEEE 754 float. 4 bytes per component.
    Float = 11,
    /// 64-bit IEEE 754 float. 8 bytes per component.
    Double = 12,
}

impl DataFormat {
    /// Parse a raw u16 format code into a `DataFormat`.
    pub fn from_u16(code: u16) -> Result<Self> {
        match code {
            1 => Ok(DataFormat::Byte),
            2 => Ok(DataFormat::Ascii),
            3 => Ok(DataFormat::Short),
            4 => Ok(DataFormat::Long),
            5 => Ok(DataFormat::Rational),
            6 => Ok(DataFormat::SByte),
            7 => Ok(DataFormat::Undefined),
            8 => Ok(DataFormat::SShort),
            9 => Ok(DataFormat::SLong),
            10 => Ok(DataFormat::SRational),
            11 => Ok(DataFormat::Float),
            12 => Ok(DataFormat::Double),
            _ => Err(ExifError::UnknownFormat(code)),
        }
    }

    /// Returns the byte size of a single component for this format.
    pub fn component_size(self) -> usize {
        match self {
            DataFormat::Byte | DataFormat::Ascii | DataFormat::SByte | DataFormat::Undefined => 1,
            DataFormat::Short | DataFormat::SShort => 2,
            DataFormat::Long | DataFormat::SLong | DataFormat::Float => 4,
            DataFormat::Rational | DataFormat::SRational | DataFormat::Double => 8,
        }
    }

    /// Returns a short name for the format, used in display output.
    pub fn name(self) -> &'static str {
        match self {
            DataFormat::Byte => "BYTE",
            DataFormat::Ascii => "ASCII",
            DataFormat::Short => "SHORT",
            DataFormat::Long => "LONG",
            DataFormat::Rational => "RATIONAL",
            DataFormat::SByte => "SBYTE",
            DataFormat::Undefined => "UNDEFINED",
            DataFormat::SShort => "SSHORT",
            DataFormat::SLong => "SLONG",
            DataFormat::SRational => "SRATIONAL",
            DataFormat::Float => "FLOAT",
            DataFormat::Double => "DOUBLE",
        }
    }
}

impl std::fmt::Display for DataFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
