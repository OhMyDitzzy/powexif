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

use crate::tag::Tag;
use crate::value::Value;

/// Returns a human-readable interpretation of a tag value when one is available, Returns None for tags whose raw
/// numeric value is already the most useful representation.
pub fn interpret(tag: Tag, value: &Value) -> Option<String> {
    let n = value.as_u16().map(|v| v as u32).or_else(|| value.as_u32());

    match tag {
        Tag::Orientation => n.map(|v| match v {
            1 => "Horizontal (normal)".into(),
            2 => "Mirror horizontal".into(),
            3 => "Rotate 180".into(),
            4 => "Mirror vertical".into(),
            5 => "Mirror horizontal and rotate 270 CW".into(),
            6 => "Rotate 90 CW".into(),
            7 => "Mirror horizontal and rotate 90 CW".into(),
            8 => "Rotate 270 CW".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::ResolutionUnit => n.map(|v| match v {
            1 => "None".into(),
            2 => "inches".into(),
            3 => "cm".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::YCbCrPositioning => n.map(|v| match v {
            1 => "Centered".into(),
            2 => "Co-sited".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::Compression => n.map(|v| match v {
            1 => "Uncompressed".into(),
            2 => "CCITT 1D".into(),
            3 => "T4/Group 3 Fax".into(),
            4 => "T6/Group 4 Fax".into(),
            5 => "LZW".into(),
            6 => "JPEG (old-style)".into(),
            7 => "JPEG".into(),
            8 => "Adobe Deflate".into(),
            9 => "JBIG B&W".into(),
            10 => "JBIG Color".into(),
            32773 => "PackBits".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::ColorSpace => n.map(|v| match v {
            1 => "sRGB".into(),
            2 => "Adobe RGB".into(),
            0xFFFF => "Uncalibrated".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::ExposureProgram => n.map(|v| match v {
            0 => "Not Defined".into(),
            1 => "Manual".into(),
            2 => "Program AE".into(),
            3 => "Aperture-priority AE".into(),
            4 => "Shutter speed priority AE".into(),
            5 => "Creative (Slow speed)".into(),
            6 => "Action (High speed)".into(),
            7 => "Portrait".into(),
            8 => "Landscape".into(),
            9 => "Bulb".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::MeteringMode => n.map(|v| match v {
            0 => "Unknown".into(),
            1 => "Average".into(),
            2 => "Center-weighted average".into(),
            3 => "Spot".into(),
            4 => "Multi-spot".into(),
            5 => "Multi-segment".into(),
            6 => "Partial".into(),
            255 => "Other".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::LightSource => n.map(|v| match v {
            0 => "Unknown".into(),
            1 => "Daylight".into(),
            2 => "Fluorescent".into(),
            3 => "Tungsten (Incandescent)".into(),
            4 => "Flash".into(),
            9 => "Fine Weather".into(),
            10 => "Cloudy".into(),
            11 => "Shade".into(),
            12 => "Daylight Fluorescent".into(),
            13 => "Day White Fluorescent".into(),
            14 => "Cool White Fluorescent".into(),
            15 => "White Fluorescent".into(),
            17 => "Standard Light A".into(),
            18 => "Standard Light B".into(),
            19 => "Standard Light C".into(),
            20 => "D55".into(),
            21 => "D65".into(),
            22 => "D75".into(),
            23 => "D50".into(),
            24 => "ISO Studio Tungsten".into(),
            255 => "Other".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::Flash => n.map(|v| {
            let fired = v & 0x1 != 0;
            let ret = (v >> 1) & 0x3;
            let mode = (v >> 3) & 0x3;
            let func = (v >> 5) & 0x1;
            let redeye = (v >> 6) & 0x1;

            if func == 1 {
                return "No Flash Function".into();
            }
            let mut s = if fired { "Fired" } else { "No Flash" }.to_string();
            if fired {
                match ret {
                    1 => s.push_str(", Return not detected"),
                    2 => s.push_str(", Return detected"),
                    _ => {}
                }
            }
            match mode {
                1 => s.push_str(", Compulsory Flash"),
                2 => s.push_str(", Compulsory Flash Off"),
                3 => s.push_str(", Auto"),
                _ => {}
            }
            if redeye == 1 {
                s.push_str(", Red-eye reduction");
            }
            s
        }),

        Tag::WhiteBalance => n.map(|v| match v {
            0 => "Auto".into(),
            1 => "Manual".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::ExposureMode => n.map(|v| match v {
            0 => "Auto".into(),
            1 => "Manual".into(),
            2 => "Auto Bracket".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::SceneCaptureType => n.map(|v| match v {
            0 => "Standard".into(),
            1 => "Landscape".into(),
            2 => "Portrait".into(),
            3 => "Night".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::SensitivityType => n.map(|v| match v {
            0 => "Unknown".into(),
            1 => "Standard Output Sensitivity".into(),
            2 => "Recommended Exposure Index".into(),
            3 => "ISO Speed".into(),
            4 => "Standard Output Sensitivity and Recommended Exposure Index".into(),
            5 => "Standard Output Sensitivity and ISO Speed".into(),
            6 => "Recommended Exposure Index and ISO Speed".into(),
            7 => "Standard Output Sensitivity, Recommended Exposure Index and ISO Speed".into(),
            _ => format!("Unknown ({})", v),
        }),

        Tag::ExposureTime => {
            if let Value::Rational(v) = value {
                v.first().map(|r| {
                    let num = r.numerator;
                    let den = r.denominator;
                    if den == 0 {
                        return "0".into();
                    }
                    if num == 0 {
                        return "0".into();
                    }
                    if num == 1 {
                        format!("1/{}", den)
                    } else {
                        // Simplify: e.g. 70005/1000000 → 1/14
                        let g = gcd(num as u64, den as u64) as u32;
                        let snum = num / g;
                        let sden = den / g;
                        if snum == 1 {
                            format!("1/{}", sden)
                        } else {
                            format!("{}/{}", snum, sden)
                        }
                    }
                })
            } else {
                None
            }
        }

        Tag::FNumber => {
            if let Value::Rational(v) = value {
                v.first()
                    .and_then(|r| r.as_f64())
                    .map(|f| format!("{:.1}", f))
            } else {
                None
            }
        }

        Tag::FocalLength => {
            if let Value::Rational(v) = value {
                v.first()
                    .and_then(|r| r.as_f64())
                    .map(|f| format!("{:.1} mm", f))
            } else {
                None
            }
        }

        Tag::DigitalZoomRatio => {
            if let Value::Rational(v) = value {
                v.first()
                    .and_then(|r| r.as_f64())
                    .map(|f| format!("{:.0}", f))
            } else {
                None
            }
        }

        Tag::ExposureBiasValue => {
            if let Value::SRational(v) = value {
                v.first().and_then(|r| r.as_f64()).map(|f| {
                    if f == 0.0 {
                        "0".into()
                    } else {
                        format!("{:.1}", f)
                    }
                })
            } else {
                None
            }
        }

        Tag::BrightnessValue => {
            if let Value::SRational(v) = value {
                v.first().and_then(|r| r.as_f64()).map(|f| {
                    if f == 0.0 {
                        "0".into()
                    } else {
                        format!("{:.4}", f)
                    }
                })
            } else {
                None
            }
        }

        Tag::ComponentsConfiguration => {
            if let Value::Undefined(bytes) = value {
                let parts: Vec<&str> = bytes
                    .iter()
                    .map(|&b| match b {
                        0 => "-",
                        1 => "Y",
                        2 => "Cb",
                        3 => "Cr",
                        4 => "R",
                        5 => "G",
                        6 => "B",
                        _ => "?",
                    })
                    .collect();
                Some(parts.join(", "))
            } else {
                None
            }
        }

        Tag::FlashPixVersion => {
            if let Value::Undefined(bytes) = value
                && bytes.len() == 4 {
                    let s: String = bytes.iter().map(|&b| b as char).collect();
                    return Some(s);
                }
            None
        }

        Tag::UserComment => {
            if let Value::Undefined(bytes) = value {
                if bytes.len() >= 8 {
                    let charset = &bytes[..8];
                    let text_bytes = &bytes[8..];
                    let text: String = text_bytes
                        .iter()
                        .filter(|&&b| b != 0)
                        .map(|&b| b as char)
                        .collect();
                    if charset.starts_with(b"ASCII\0\0\0") || charset.iter().all(|&b| b == 0) {
                        return Some(text.trim().to_string());
                    }
                }
                Some(String::new())
            } else {
                None
            }
        }

        _ => None,
    }
}

/// Compute greatest common divisor via Euclidean algorithm.
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Computed/derived fields that exiftool shows but which aren't raw EXIF tags.
pub struct ComputedFields {
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub image_size: Option<String>,
    pub megapixels: Option<String>,
    pub scale_factor_35mm: Option<String>,
    pub circle_of_confusion: Option<String>,
    pub field_of_view: Option<String>,
    pub hyperfocal_distance: Option<String>,
    pub light_value: Option<String>,
}

impl ComputedFields {
    pub fn compute(exif: &crate::ExifData) -> Self {
        use crate::ifd::Ifd;

        let get_rational = |ifd: Ifd, tag: Tag| -> Option<f64> {
            exif.get(ifd, tag)?.value.as_rational()?.as_f64()
        };
        let get_u16 = |ifd: Ifd, tag: Tag| -> Option<u32> {
            let v = exif.get(ifd, tag)?;
            v.value
                .as_u16()
                .map(|x| x as u32)
                .or_else(|| v.value.as_u32())
        };

        let fnumber = get_rational(Ifd::ExifIfd, Tag::FNumber);
        let focal_mm = get_rational(Ifd::ExifIfd, Tag::FocalLength);
        let focal_35 = get_u16(Ifd::ExifIfd, Tag::FocalLengthIn35mmFilm).map(|v| v as f64);
        let exposure_time_r = exif.get(Ifd::ExifIfd, Tag::ExposureTime).and_then(|e| {
            if let Value::Rational(ref v) = e.value {
                v.first().copied()
            } else {
                None
            }
        });
        let width = get_u16(Ifd::ExifIfd, Tag::PixelXDimension)
            .or_else(|| get_u16(Ifd::Ifd0, Tag::ImageWidth));
        let height = get_u16(Ifd::ExifIfd, Tag::PixelYDimension)
            .or_else(|| get_u16(Ifd::Ifd0, Tag::ImageLength));

        let aperture = fnumber.map(|f| format!("{:.1}", f));

        let shutter_speed = exposure_time_r.and_then(|r| {
            if r.denominator == 0 {
                return None;
            }
            let num = r.numerator as u64;
            let den = r.denominator as u64;
            let g = gcd(num, den);
            let snum = num / g;
            let sden = den / g;
            Some(if snum == 1 {
                format!("1/{}", sden)
            } else {
                format!("{}/{}", snum, sden)
            })
        });

        let image_size = match (width, height) {
            (Some(w), Some(h)) => Some(format!("{}x{}", w, h)),
            _ => None,
        };

        let megapixels = match (width, height) {
            (Some(w), Some(h)) => Some(format!("{:.1}", (w as f64 * h as f64) / 1_000_000.0)),
            _ => None,
        };

        // Scale factor: ratio of 35mm equivalent to actual focal length
        let scale_factor_35mm = match (focal_mm, focal_35) {
            (Some(f), Some(f35)) if f > 0.0 => Some(format!("{:.1}", f35 / f)),
            _ => None,
        };

        // Circle of confusion: sensor diagonal / 1500 (common approximation)
        // For known sensor size via scale factor: CoC = 0.043mm / scale_factor
        let coc = match (focal_mm, focal_35) {
            (Some(f), Some(f35)) if f > 0.0 => {
                let scale = f35 / f;
                if scale > 0.0 {
                    Some(0.043 / scale)
                } else {
                    None
                }
            }
            _ => None,
        };
        let circle_of_confusion = coc.map(|c| format!("{:.3} mm", c));

        // Field of view: 2 * atan(diagonal_35mm / (2 * focal_35mm)) in degrees
        // 35mm frame diagonal = 43.27mm
        let field_of_view = focal_35.map(|f35| {
            if f35 <= 0.0 {
                return "0 deg".into();
            }
            let half_angle = (43.27f64 / (2.0 * f35)).atan();
            format!("{:.1} deg", half_angle.to_degrees() * 2.0)
        });

        // Hyperfocal distance: f^2 / (N * c) + f  where f=focal mm, N=f-number, c=CoC mm
        let hyperfocal_distance = match (focal_mm, fnumber, coc) {
            (Some(f), Some(n), Some(c)) if n > 0.0 && c > 0.0 => {
                let f_m = f / 1000.0; // convert mm to m
                let c_m = c / 1000.0;
                let h = (f_m * f_m) / (n * c_m) + f_m;
                Some(format!("{:.2} m", h))
            }
            _ => None,
        };

        // Light value: LV = log2(N^2 / t) where N=f-number, t=exposure time in seconds
        let light_value = match (fnumber, exposure_time_r) {
            (Some(n), Some(r)) if r.denominator != 0 => {
                let t = r.numerator as f64 / r.denominator as f64;
                if t > 0.0 {
                    let lv = (n * n / t).log2();
                    Some(format!("{:.1}", lv))
                } else {
                    None
                }
            }
            _ => None,
        };

        ComputedFields {
            aperture,
            shutter_speed,
            image_size,
            megapixels,
            scale_factor_35mm,
            circle_of_confusion,
            field_of_view,
            hyperfocal_distance,
            light_value,
        }
    }
}
