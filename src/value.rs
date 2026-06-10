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

use crate::format::DataFormat;

/// A rational number as stored in EXIF: numerator and denominator are both unsigned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    pub numerator: u32,
    pub denominator: u32,
}

impl Rational {
    pub fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// Returns the value as an f64, or None if the denominator is zero.
    pub fn as_f64(self) -> Option<f64> {
        if self.denominator == 0 {
            None
        } else {
            Some(self.numerator as f64 / self.denominator as f64)
        }
    }
}

impl std::fmt::Display for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else if let Some(v) = self.as_f64() {
            write!(f, "{}/{} ({:.4})", self.numerator, self.denominator, v)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

/// A signed rational number as stored in EXIF: both numerator and denominator are signed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SRational {
    pub numerator: i32,
    pub denominator: i32,
}

impl SRational {
    pub fn new(numerator: i32, denominator: i32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// Returns the value as an f64, or None if the denominator is zero.
    pub fn as_f64(self) -> Option<f64> {
        if self.denominator == 0 {
            None
        } else {
            Some(self.numerator as f64 / self.denominator as f64)
        }
    }
}

impl std::fmt::Display for SRational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else if let Some(v) = self.as_f64() {
            write!(f, "{}/{} ({:.4})", self.numerator, self.denominator, v)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

/// A typed value decoded from a raw EXIF tag entry.
///
/// EXIF allows multiple components per tag (e.g., GPS coordinates use three
/// Rational values). All multi-component values are stored as Vec.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Byte(Vec<u8>),
    Ascii(String),
    Short(Vec<u16>),
    Long(Vec<u32>),
    Rational(Vec<Rational>),
    SByte(Vec<i8>),
    Undefined(Vec<u8>),
    SShort(Vec<i16>),
    SLong(Vec<i32>),
    SRational(Vec<SRational>),
    Float(Vec<f32>),
    Double(Vec<f64>),
}

impl Value {
    /// Returns the `DataFormat` that corresponds to this value variant.
    pub fn format(&self) -> DataFormat {
        match self {
            Value::Byte(_) => DataFormat::Byte,
            Value::Ascii(_) => DataFormat::Ascii,
            Value::Short(_) => DataFormat::Short,
            Value::Long(_) => DataFormat::Long,
            Value::Rational(_) => DataFormat::Rational,
            Value::SByte(_) => DataFormat::SByte,
            Value::Undefined(_) => DataFormat::Undefined,
            Value::SShort(_) => DataFormat::SShort,
            Value::SLong(_) => DataFormat::SLong,
            Value::SRational(_) => DataFormat::SRational,
            Value::Float(_) => DataFormat::Float,
            Value::Double(_) => DataFormat::Double,
        }
    }

    /// Returns the number of components in this value.
    pub fn component_count(&self) -> usize {
        match self {
            Value::Byte(v) => v.len(),
            // ASCII component count includes the null terminator per the spec.
            Value::Ascii(s) => s.len() + 1,
            Value::Short(v) => v.len(),
            Value::Long(v) => v.len(),
            Value::Rational(v) => v.len(),
            Value::SByte(v) => v.len(),
            Value::Undefined(v) => v.len(),
            Value::SShort(v) => v.len(),
            Value::SLong(v) => v.len(),
            Value::SRational(v) => v.len(),
            Value::Float(v) => v.len(),
            Value::Double(v) => v.len(),
        }
    }

    /// Convenience: returns the first Short value, or None.
    pub fn as_u16(&self) -> Option<u16> {
        if let Value::Short(v) = self {
            v.first().copied()
        } else {
            None
        }
    }

    /// Convenience: returns the first Long value, or None.
    pub fn as_u32(&self) -> Option<u32> {
        if let Value::Long(v) = self {
            v.first().copied()
        } else {
            None
        }
    }

    /// Convenience: returns the ASCII string value, or None.
    pub fn as_str(&self) -> Option<&str> {
        if let Value::Ascii(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    /// Convenience: returns the first Rational value, or None.
    pub fn as_rational(&self) -> Option<Rational> {
        if let Value::Rational(v) = self {
            v.first().copied()
        } else {
            None
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Byte(v) => {
                let parts: Vec<String> = v.iter().map(|b| b.to_string()).collect();
                write!(f, "{}", parts.join(", "))
            }
            Value::Ascii(s) => write!(f, "{}", s),
            Value::Short(v) => {
                let parts: Vec<String> = v.iter().map(|n| n.to_string()).collect();
                write!(f, "{}", parts.join(", "))
            }
            Value::Long(v) => {
                let parts: Vec<String> = v.iter().map(|n| n.to_string()).collect();
                write!(f, "{}", parts.join(", "))
            }
            Value::Rational(v) => {
                let parts: Vec<String> = v.iter().map(|r| r.to_string()).collect();
                write!(f, "{}", parts.join(", "))
            }
            Value::SByte(v) => {
                let parts: Vec<String> = v.iter().map(|b| b.to_string()).collect();
                write!(f, "{}", parts.join(", "))
            }
            Value::Undefined(v) => write!(f, "{} bytes (undefined)", v.len()),
            Value::SShort(v) => {
                let parts: Vec<String> = v.iter().map(|n| n.to_string()).collect();
                write!(f, "{}", parts.join(", "))
            }
            Value::SLong(v) => {
                let parts: Vec<String> = v.iter().map(|n| n.to_string()).collect();
                write!(f, "{}", parts.join(", "))
            }
            Value::SRational(v) => {
                let parts: Vec<String> = v.iter().map(|r| r.to_string()).collect();
                write!(f, "{}", parts.join(", "))
            }
            Value::Float(v) => {
                let parts: Vec<String> = v.iter().map(|n| format!("{:.6}", n)).collect();
                write!(f, "{}", parts.join(", "))
            }
            Value::Double(v) => {
                let parts: Vec<String> = v.iter().map(|n| format!("{:.6}", n)).collect();
                write!(f, "{}", parts.join(", "))
            }
        }
    }
}
