use std::time::Duration;

pub trait ByteParser {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self;
}

impl ByteParser for f64 {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let value = f64::from_le_bytes([
            bytes[*ptr],
            bytes[*ptr + 1],
            bytes[*ptr + 2],
            bytes[*ptr + 3],
            bytes[*ptr + 4],
            bytes[*ptr + 5],
            bytes[*ptr + 6],
            bytes[*ptr + 7],
        ]);
        *ptr += 8;
        value
    }
}

impl ByteParser for i64 {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let value = i64::from_le_bytes([
            bytes[*ptr],
            bytes[*ptr + 1],
            bytes[*ptr + 2],
            bytes[*ptr + 3],
            bytes[*ptr + 4],
            bytes[*ptr + 5],
            bytes[*ptr + 6],
            bytes[*ptr + 7],
        ]);
        *ptr += 8;
        value
    }
}

impl ByteParser for u64 {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        i64::parse_bytes(bytes, ptr) as u64
    }
}

impl ByteParser for i32 {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let value = i32::from_le_bytes([
            bytes[*ptr],
            bytes[*ptr + 1],
            bytes[*ptr + 2],
            bytes[*ptr + 3],
        ]);
        *ptr += 4;
        value
    }
}

impl ByteParser for f32 {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let value = f32::from_le_bytes([
            bytes[*ptr],
            bytes[*ptr + 1],
            bytes[*ptr + 2],
            bytes[*ptr + 3],
        ]);
        *ptr += 4;
        value
    }
}

impl ByteParser for bool {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let value = bytes[*ptr] != 0;
        *ptr += 1;
        value
    }
}

impl<T: ByteParser> ByteParser for Option<T> {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let has_value = bool::parse_bytes(bytes, ptr);
        if has_value {
            Some(T::parse_bytes(bytes, ptr))
        } else {
            None
        }
    }
}

impl<T: ByteParser> ByteParser for Vec<T> {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let len = bytes[*ptr] as usize;
        *ptr += 1;

        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::parse_bytes(bytes, ptr));
        }

        vec
    }
}

pub struct Translate3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl ByteParser for Translate3D {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let x = f64::parse_bytes(bytes, ptr);
        let y = f64::parse_bytes(bytes, ptr);
        let z = f64::parse_bytes(bytes, ptr);

        Translate3D { x, y, z }
    }
}

pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl ByteParser for Quaternion {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let w = f64::parse_bytes(bytes, ptr);
        let x = f64::parse_bytes(bytes, ptr);
        let y = f64::parse_bytes(bytes, ptr);
        let z = f64::parse_bytes(bytes, ptr);

        Self { w, x, y, z }
    }
}

pub struct Transform3D {
    pub translation: Translate3D,
    pub rotation: Quaternion,
}

impl ByteParser for Transform3D {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let translation = Translate3D::parse_bytes(bytes, ptr);
        let rotation = Quaternion::parse_bytes(bytes, ptr);

        Transform3D {
            translation,
            rotation,
        }
    }
}

pub struct PhotonPipelineMetadata {
    pub seqid: u64,
    pub capture_time: Duration,
    pub publish_time: Duration,
}

impl ByteParser for PhotonPipelineMetadata {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let seqid = u64::parse_bytes(bytes, ptr);
        let capture_us = u64::parse_bytes(bytes, ptr);
        let publish_us = u64::parse_bytes(bytes, ptr);

        PhotonPipelineMetadata {
            seqid: seqid as u64,
            capture_time: Duration::from_micros(capture_us as u64),
            publish_time: Duration::from_micros(publish_us as u64),
        }
    }
}

pub struct TargetCorner {
    pub x: f64,
    pub y: f64,
}

impl ByteParser for TargetCorner {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let x = f64::parse_bytes(bytes, ptr);
        let y = f64::parse_bytes(bytes, ptr);

        TargetCorner { x, y }
    }
}

pub struct PhotonTrackedTarget {
    pub yaw: f64,
    pub pitch: f64,
    pub area: f64,
    pub skew: f64,
    pub fiducial_id: i32,
    pub object_detect_id: i32,
    pub object_detect_conf: f32,
    pub best_camera_to_target: Transform3D,
    pub alt_camera_to_target: Transform3D,
    pub min_area_rect_corners: Vec<TargetCorner>,
    pub detected_corners: Vec<TargetCorner>,
}

impl ByteParser for PhotonTrackedTarget {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let yaw = f64::parse_bytes(bytes, ptr);
        let pitch = f64::parse_bytes(bytes, ptr);
        let area = f64::parse_bytes(bytes, ptr);
        let skew = f64::parse_bytes(bytes, ptr);
        let fiducial_id = i32::parse_bytes(bytes, ptr);
        let object_detect_id = i32::parse_bytes(bytes, ptr);
        let object_detect_conf = f32::parse_bytes(bytes, ptr);
        let best_camera_to_target = Transform3D::parse_bytes(bytes, ptr);
        let alt_camera_to_target = Transform3D::parse_bytes(bytes, ptr);
        let min_area_rect_corners = Vec::<TargetCorner>::parse_bytes(bytes, ptr);
        let detected_corners = Vec::<TargetCorner>::parse_bytes(bytes, ptr);

        PhotonTrackedTarget {
            yaw,
            pitch,
            area,
            skew,
            fiducial_id,
            object_detect_id,
            object_detect_conf,
            best_camera_to_target,
            alt_camera_to_target,
            min_area_rect_corners,
            detected_corners,
        }
    }
}

pub struct PnpResult {
    pub best: Transform3D,
    pub alt: Transform3D,
    pub reproj_error: f64,
    pub alt_reproj_error: f64,
    pub ambiguity: f64,
}

impl ByteParser for PnpResult {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let best = Transform3D::parse_bytes(bytes, ptr);
        let alt = Transform3D::parse_bytes(bytes, ptr);
        let reproj_error = f64::parse_bytes(bytes, ptr);
        let alt_reproj_error = f64::parse_bytes(bytes, ptr);
        let ambiguity = f64::parse_bytes(bytes, ptr);

        PnpResult {
            best,
            alt,
            reproj_error,
            alt_reproj_error,
            ambiguity,
        }
    }
}

pub struct PhotonResult {
    pub metadata: PhotonPipelineMetadata,
    pub targets: Vec<PhotonTrackedTarget>,
    pub pnp_results: Vec<PnpResult>,
}

impl ByteParser for PhotonResult {
    fn parse_bytes(bytes: &[u8], ptr: &mut usize) -> Self {
        let metadata = PhotonPipelineMetadata::parse_bytes(bytes, ptr);
        let targets = Vec::<PhotonTrackedTarget>::parse_bytes(bytes, ptr);
        let pnp_results = Vec::<PnpResult>::parse_bytes(bytes, ptr);

        PhotonResult {
            metadata,
            targets,
            pnp_results,
        }
    }
}
