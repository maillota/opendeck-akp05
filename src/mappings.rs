use mirajazz::types::{ImageFormat, ImageMirroring, ImageMode, ImageRotation};

// Must be unique between all the plugins, 2 characters long and match `DeviceNamespace` field in `manifest.json`
pub const DEVICE_NAMESPACE: &str = "az";

pub const ROW_COUNT: usize = 2;
pub const COL_COUNT: usize = 5;
pub const KEY_COUNT: usize = 10;
pub const ENCODER_COUNT: usize = 4;

pub const IMAGE_FORMAT: ImageFormat = ImageFormat {
    mode: ImageMode::JPEG,
    size: (100, 100),
    rotation: ImageRotation::Rot180,
    mirror: ImageMirroring::None,
};

#[derive(Debug, Clone)]
pub enum Kind {
    AKP05,
}

pub const AJAZZ_VID: u16 = 0x0300;
pub const AKP05_PID: u16 = 0x3004;

impl Kind {
    /// Matches devices VID+PID pairs to correct kinds
    pub fn from_vid_pid(vid: u16, pid: u16) -> Option<Self> {
        match vid {
            AJAZZ_VID => match pid {
                AKP05_PID => Some(Kind::AKP05),
                _ => None,
            },

            _ => None,
        }
    }

    /// Returns true for devices that emitting two events per key press, instead of one
    /// Currently only one device does that
    pub fn supports_both_states(&self) -> bool {
        match &self {
            Self::AKP05 => true,
            _ => false,
        }
    }

    /// There is no point relying on manufacturer/device names reported by the USB stack,
    /// so we return custom names for all the kinds of devices
    pub fn human_name(&self) -> String {
        match &self {
            Self::AKP05 => "Ajazz AKP05",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct CandidateDevice {
    pub id: String,
    pub vid: u16,
    pub pid: u16,
    pub serial: String,
    pub kind: Kind,
}
