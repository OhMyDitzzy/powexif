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

/// Identifies which IFD (Image File Directory) a tag belongs to.
///
/// A JPEG with EXIF data contains multiple IFDs. IFD0 holds primary image
/// metadata, IFD1 holds thumbnail metadata, and the sub-IFDs (Exif, GPS,
/// Interop) are referenced by pointer tags inside IFD0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ifd {
    /// Primary image directory. Contains the most common tags (Make, Model,
    /// DateTime, orientation, etc.) and pointers to ExifIfd and GpsIfd.
    Ifd0,
    /// Thumbnail image directory. Usually contains a small embedded JPEG.
    Ifd1,
    /// Sub-IFD for extended camera settings: ISO, shutter speed, aperture, etc.
    ExifIfd,
    /// Sub-IFD for GPS coordinates and related information.
    GpsIfd,
    /// Sub-IFD for interoperability metadata (mostly used in DCF-compliant cameras).
    InteropIfd,
}

impl Ifd {
    /// Returns a human-readable name for display purposes.
    pub fn name(self) -> &'static str {
        match self {
            Ifd::Ifd0 => "IFD0",
            Ifd::Ifd1 => "IFD1",
            Ifd::ExifIfd => "Exif IFD",
            Ifd::GpsIfd => "GPS IFD",
            Ifd::InteropIfd => "Interoperability IFD",
        }
    }
}

impl std::fmt::Display for Ifd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
