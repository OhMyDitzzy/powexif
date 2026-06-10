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

use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, ExifError>;

/// All errors that can occur while parsing, editing, or saving EXIF data.
#[derive(Debug)]
pub enum ExifError {
    /// The data does not start with a valid JPEG SOI or EXIF header.
    InvalidHeader,
    /// The TIFF byte-order marker ("II" or "MM") was not found.
    InvalidByteOrder,
    /// An offset or length value points outside the bounds of the data buffer.
    OffsetOutOfBounds { offset: usize, available: usize },
    /// A required slice was too short to read the expected number of bytes.
    UnexpectedEnd { needed: usize, available: usize },
    /// A tag value uses a data format code that is not defined in the EXIF spec.
    UnknownFormat(u16),
    /// The EXIF APP1 segment was not found in the JPEG file.
    NoExifSegment,
    /// A string value contains bytes that are not valid UTF-8.
    InvalidUtf8(std::string::FromUtf8Error),
    /// An IO operation failed (file read/write).
    Io(io::Error),
    /// The caller tried to set a value whose type does not match the tag's expected format.
    FormatMismatch { expected: String, got: String },
}

impl fmt::Display for ExifError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExifError::InvalidHeader => write!(f, "invalid EXIF header"),
            ExifError::InvalidByteOrder => write!(f, "invalid byte order marker"),
            ExifError::OffsetOutOfBounds { offset, available } => {
                write!(
                    f,
                    "offset {offset} is out of bounds (data size: {available})"
                )
            }
            ExifError::UnexpectedEnd { needed, available } => {
                write!(f, "need {needed} bytes but only {available} remain")
            }
            ExifError::UnknownFormat(code) => write!(f, "unknown data format code: {code}"),
            ExifError::NoExifSegment => write!(f, "no EXIF APP1 segment found in JPEG"),
            ExifError::InvalidUtf8(e) => write!(f, "invalid UTF-8 in ASCII tag: {e}"),
            ExifError::Io(e) => write!(f, "I/O error: {e}"),
            ExifError::FormatMismatch { expected, got } => {
                write!(f, "format mismatch: expected {expected}, got {got}")
            }
        }
    }
}

impl std::error::Error for ExifError {}

impl From<io::Error> for ExifError {
    fn from(e: io::Error) -> Self {
        ExifError::Io(e)
    }
}

impl From<std::string::FromUtf8Error> for ExifError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        ExifError::InvalidUtf8(e)
    }
}
