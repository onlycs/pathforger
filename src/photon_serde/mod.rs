mod builtin;

#[cfg(test)]
mod test;

use crate::prelude::*;
use std::{
    hash::{Hash, Hasher},
    io::Cursor,
    time::Duration,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeserializeError {}

pub trait Deserialize: Sized {
    fn deserialize(data: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError>;
}

pub fn deserialize<D: Deserialize>(data: &[u8]) -> Result<D, DeserializeError> {
    D::deserialize(&mut Cursor::new(data))
}

trait FpHash {
    fn hash(&self, h: &mut impl Hasher);
}

impl FpHash for f64 {
    fn hash(&self, h: &mut impl Hasher) {
        if self.is_nan() {
            f64::NAN.to_le_bytes().hash(h);
        } else {
            self.to_le_bytes().hash(h);
        }
    }
}

impl FpHash for f32 {
    fn hash(&self, h: &mut impl Hasher) {
        if self.is_nan() {
            f32::NAN.to_le_bytes().hash(h);
        } else {
            self.to_le_bytes().hash(h);
        }
    }
}

impl FpHash for Angle {
    fn hash(&self, h: &mut impl Hasher) {
        self.get::<radian>().hash(h);
    }
}

impl FpHash for Length {
    fn hash(&self, h: &mut impl Hasher) {
        self.get::<meter>().hash(h);
    }
}

macro_rules! define_types {
    // main entrypoint
    ($(
        $([$manual:ident ( $($eq:ident)?, $($hash:ident)? )])?
        $(#[$struct_attr:meta])*
        pub struct $struct:ident {
            $(pub $field:ident: $ty:ty),* $(,)?
        }
    )*) => {
        $(
            $(#[$struct_attr])*
            pub struct $struct {
                $(pub $field: $ty),*
            }

            impl Deserialize for $struct {
                #[allow(unused_assignments)]
                fn deserialize(data: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
                    Ok(Self {
                        $($field: <$ty as Deserialize>::deserialize(data)?),*
                    })
                }
            }

            define_types!(impl $( $manual $($eq)? )? $struct);
            define_types!(impl $( $manual $($hash)? )? $struct $($field:$ty)*);
        )*
    };

    // Eq+Hash manual impls
    (impl manual Eq $struct:ident) => {
        impl Eq for $struct {}
    };
    (impl manual Hash $struct:ident $($field:ident : $ty:ty)*) => {
        impl Hash for $struct {
            fn hash<H: Hasher>(&self, hasher: &mut H) {
                $(self.$field.hash(hasher));*
            }
        }
    };

    // catch other internal uses
    (impl $_a:ident $_b:ident $($_c:ident : $_d:ty)*) => {};
    (impl $_a:ident $_b:ident) => {};
    (impl $_a:ident $($_b:ident : $_c:ty)*) => {};
    (impl $_a:ident) => {};
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FiducialId(pub Option<u32>);

impl Deserialize for FiducialId {
    fn deserialize(data: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let value = i32::deserialize(data)?;

        if value == -1 {
            return Ok(Self(None));
        }

        Ok(Self(Some(value as u32)))
    }
}

// photon types
define_types! {
    [manual(Eq, Hash)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Quaternion {
        pub w: f64,
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }

    [manual(Eq, Hash)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Translate3d {
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Transform3d {
        pub translation: Translate3d,
        pub rotation: Quaternion,
    }

    [manual(Eq, Hash)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DetectedObject {
        pub id: u64,
        pub confidence: f32,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct TargetTransforms {
        pub best: Transform3d,
        pub alt: Transform3d,
    }

    [manual(Eq, Hash)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct TargetCorner {
        pub x: f64,
        pub y: f64,
    }

    [manual(Eq, Hash)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PNPResult {
        pub best: Transform3d,
        pub alt: Transform3d,
        pub error: f64,
        pub alt_error: f64,
        pub ambiguity: f64,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct PhotonPipelineMetadata {
        pub seqid: u64,
        pub capture_time: Duration,
        pub publish_time: Duration,
        pub last_handshake: Duration,
    }

    [manual(Eq, Hash)]
    #[derive(Clone, Debug, PartialEq)]
    pub struct PhotonTrackedTarget {
        pub yaw: f64,
        pub pitch: f64,
        pub area: f64,
        pub skew: f64,
        pub fiducial_id: FiducialId,
        pub detected: DetectedObject,
        pub to_target: TargetTransforms,
        pub ambiguity: f64,
        pub area_rect_corners: Vec<TargetCorner>,
        pub detected_corners: Vec<TargetCorner>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct MultiTargetPNP {
        pub pnp: PNPResult,
        pub num_fiducials: u16,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct PhotonResult {
        pub metadata: PhotonPipelineMetadata,
        pub targets: Vec<PhotonTrackedTarget>,
        pub pnp: Option<MultiTargetPNP>,
    }
}

// robot types
define_types! {
    [manual(Eq, Hash)]
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Rotate2d {
        pub angle: Angle,
    }

    [manual(Eq, Hash)]
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Translate2d {
        pub x: Length,
        pub y: Length,
    }

    [manual(Eq, Hash)]
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Pose2d {
        pub translate: Translate2d,
        pub rotate: Rotate2d,
    }
}

pub mod prelude {
    pub use super::*;
}
