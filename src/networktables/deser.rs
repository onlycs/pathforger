use core::f64;
use serde::Deserialize;
use std::{
    hash::{Hash, Hasher},
    time::Duration,
};

mod deser {
    use serde::de::Deserializer;
    use serde::Deserialize;
    use std::time::Duration;

    pub fn duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let micros = i64::deserialize(deserializer)?;
        Ok(Duration::from_micros(micros as u64))
    }

    pub fn u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(i64::deserialize(deserializer)? as u64)
    }

    pub fn u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(i32::deserialize(deserializer)? as u16)
    }

    pub fn ficudial_id<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = i32::deserialize(deserializer)?;

        if id == -1 {
            Ok(None)
        } else {
            Ok(Some(id as u32))
        }
    }

    pub fn vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        // i cannot be bothered.
        let copier = || unsafe { std::mem::transmute_copy::<D, D>(&deserializer) };

        let len = i8::deserialize(copier())? as usize;
        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            vec.push(T::deserialize(copier())?);
        }

        Ok(vec)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct Quaternion {
    w: f64,
    x: f64,
    y: f64,
    z: f64,
}

impl Quaternion {
    pub fn roll(&self) -> f64 {
        let Self { w, x, y, z } = self;

        return f64::atan2(2.0 * (w * x + y * z), 1.0 - 2.0 * (x * x + y * y));
    }

    pub fn pitch(&self) -> f64 {
        let Self { w, x, y, z } = self;
        let ratio = 2.0 * (w * y - z * x);

        if ratio.abs() >= 1.0 {
            return f64::copysign(f64::consts::FRAC_PI_2, ratio);
        } else {
            return f64::asin(ratio);
        }
    }

    pub fn yaw(&self) -> f64 {
        let Self { w, x, y, z } = self;

        return f64::atan2(2.0 * (w * z + x * y), 1.0 - 2.0 * (y * y + z * z));
    }
}

impl Eq for Quaternion {}
impl Hash for Quaternion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.w.to_bits().hash(state);
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.z.to_bits().hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct Translate3D {
    x: f64,
    y: f64,
    z: f64,
}

impl Eq for Translate3D {}
impl Hash for Translate3D {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.z.to_bits().hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct TargetCorner {
    pub x: f64,
    pub y: f64,
}

impl Eq for TargetCorner {}
impl Hash for TargetCorner {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct DetectedObject {
    #[serde(deserialize_with = "deser::u64")]
    pub id: u64,
    pub confidence: f32,
}

impl Eq for DetectedObject {}
impl Hash for DetectedObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.confidence.to_bits().hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub struct TargetTransforms {
    pub best: Transform3D,
    pub alt: Transform3D,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub struct Transform3D {
    translation: Translate3D,
    rotation: Quaternion,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PNPResult {
    pub best: Transform3D,
    pub alt: Transform3D,
    pub error: f64,
    pub alt_error: f64,
    pub ambiguity: f64,
}

impl Eq for PNPResult {}
impl Hash for PNPResult {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.best.hash(state);
        self.alt.hash(state);
        self.error.to_bits().hash(state);
        self.alt_error.to_bits().hash(state);
        self.ambiguity.to_bits().hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub struct PhotonPipelineMetadata {
    #[serde(deserialize_with = "deser::u64")]
    pub seqid: u64,
    #[serde(deserialize_with = "deser::duration")]
    pub capture_time: Duration,
    #[serde(deserialize_with = "deser::duration")]
    pub publish_time: Duration,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PhotonTrackedTarget {
    pub yaw: f64,
    pub pitch: f64,
    pub area: f64,
    pub skew: f64,
    #[serde(deserialize_with = "deser::ficudial_id")]
    pub fiducial_id: Option<u32>,
    #[serde(flatten)]
    pub detected: DetectedObject,
    #[serde(flatten)]
    pub to_target: TargetTransforms,
    #[serde(deserialize_with = "deser::vec")]
    pub area_rect_corners: Vec<TargetCorner>,
    #[serde(deserialize_with = "deser::vec")]
    pub detected_corners: Vec<TargetCorner>,
}

impl Eq for PhotonTrackedTarget {}
impl Hash for PhotonTrackedTarget {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.yaw.to_bits().hash(state);
        self.pitch.to_bits().hash(state);
        self.area.to_bits().hash(state);
        self.skew.to_bits().hash(state);
        self.fiducial_id.hash(state);
        self.detected.hash(state);
        self.to_target.hash(state);
        self.area_rect_corners.hash(state);
        self.detected_corners.hash(state);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub struct MultiTargetPNP {
    pub pnp: PNPResult,
    #[serde(deserialize_with = "deser::u16")]
    pub num_fiducials: u16,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PhotonRes {
    pub metadata: PhotonPipelineMetadata,
    pub target: Vec<PhotonTrackedTarget>,
    pub pnp: Option<MultiTargetPNP>,
}
