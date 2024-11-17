use super::*;
use rand::{rngs::ThreadRng, Rng};

macro_rules! join_bytes {
    ($($bytes:expr),*) => {
        {
            let mut joined = Vec::new();
            $(joined.extend_from_slice(&$bytes);)*
            joined
        }
    };
}

fn dummy_duration(rng: &mut ThreadRng) -> (Duration, Vec<u8>) {
    let micros: i64 = rng.gen();
    let duration = Duration::from_micros(micros as u64);
    (duration, micros.to_le_bytes().to_vec())
}

fn dummy_u64(rng: &mut ThreadRng) -> (u64, Vec<u8>) {
    let dummy = rng.gen::<i64>().abs();
    (dummy as u64, dummy.to_le_bytes().to_vec())
}

fn dummy_u16(rng: &mut ThreadRng) -> (u16, Vec<u8>) {
    let dummy = rng.gen::<i16>().abs();
    (dummy as u16, dummy.to_le_bytes().to_vec())
}

fn dummy_fiducial_id(rng: &mut ThreadRng) -> (FiducialId, Vec<u8>) {
    let dummy = rng.gen::<i32>().max(-1);
    let bytes = dummy.to_le_bytes().to_vec();

    if dummy == -1 {
        (FiducialId(None), bytes)
    } else {
        (FiducialId(Some(dummy as u32)), bytes)
    }
}

fn dummy_vec<T>(
    mut len: i8,
    generator: fn(&mut ThreadRng) -> (T, Vec<u8>),
    rng: &mut ThreadRng,
) -> (Vec<T>, Vec<u8>) {
    len = len.abs();

    let mut bytes = len.to_le_bytes().to_vec();
    let mut data = vec![];

    for _ in 0..len {
        let (ndata, nbytes) = generator(rng);
        data.push(ndata);
        bytes.extend(nbytes);
    }

    (data, bytes)
}

fn dummy_optional<T>(
    rng: &mut ThreadRng,
    generator: fn(&mut ThreadRng) -> (T, Vec<u8>),
) -> (Option<T>, Vec<u8>) {
    let is_some: bool = rng.gen();

    if !is_some {
        return (None, vec![0]);
    }

    let (data, data_bytes) = generator(rng);
    let bytes = join_bytes!([1], data_bytes);

    (Some(data), bytes)
}

fn dummy_quaternion(rng: &mut ThreadRng) -> (Quaternion, Vec<u8>) {
    let w: f64 = rng.gen();
    let x: f64 = rng.gen();
    let y: f64 = rng.gen();
    let z: f64 = rng.gen();

    let bytes = join_bytes!(
        w.to_le_bytes(),
        x.to_le_bytes(),
        y.to_le_bytes(),
        z.to_le_bytes()
    );

    (Quaternion { w, x, y, z }, bytes)
}

fn dummy_translate3d(rng: &mut ThreadRng) -> (Translate3d, Vec<u8>) {
    let x: f64 = rng.gen();
    let y: f64 = rng.gen();
    let z: f64 = rng.gen();

    let bytes = join_bytes!(x.to_le_bytes(), y.to_le_bytes(), z.to_le_bytes());

    (Translate3d { x, y, z }, bytes)
}

fn dummy_target_corner(rng: &mut ThreadRng) -> (TargetCorner, Vec<u8>) {
    let x: f64 = rng.gen();
    let y: f64 = rng.gen();

    let bytes = join_bytes!(x.to_le_bytes(), y.to_le_bytes());

    (TargetCorner { x, y }, bytes)
}

fn dummy_detected_object(rng: &mut ThreadRng) -> (DetectedObject, Vec<u8>) {
    let (id, id_bytes) = dummy_u64(rng);
    let confidence: f32 = rng.gen();

    let bytes = join_bytes!(id_bytes, confidence.to_le_bytes());

    (DetectedObject { id, confidence }, bytes)
}

fn dummy_transform3d(rng: &mut ThreadRng) -> (Transform3d, Vec<u8>) {
    let (translation, translation_bytes) = dummy_translate3d(rng);
    let (rotation, quaternion_bytes) = dummy_quaternion(rng);

    let bytes = join_bytes!(translation_bytes, quaternion_bytes);

    (
        Transform3d {
            translation,
            rotation,
        },
        bytes,
    )
}

fn dummy_target_transforms(rng: &mut ThreadRng) -> (TargetTransforms, Vec<u8>) {
    let (best, best_bytes) = dummy_transform3d(rng);
    let (alt, alt_bytes) = dummy_transform3d(rng);

    let bytes = join_bytes!(best_bytes, alt_bytes);

    (TargetTransforms { best, alt }, bytes)
}

fn dummy_pnp_result(rng: &mut ThreadRng) -> (PNPResult, Vec<u8>) {
    let (best, best_bytes) = dummy_transform3d(rng);
    let (alt, alt_bytes) = dummy_transform3d(rng);
    let error: f64 = rng.gen();
    let alt_error: f64 = rng.gen();
    let ambiguity: f64 = rng.gen();

    let bytes = join_bytes!(
        best_bytes,
        alt_bytes,
        error.to_le_bytes(),
        alt_error.to_le_bytes(),
        ambiguity.to_le_bytes()
    );

    (
        PNPResult {
            best,
            alt,
            error,
            alt_error,
            ambiguity,
        },
        bytes,
    )
}

fn dummy_pipeline_metadata(rng: &mut ThreadRng) -> (PhotonPipelineMetadata, Vec<u8>) {
    let (seqid, seqid_bytes) = dummy_u64(rng);
    let (capture_time, capture_bytes) = dummy_duration(rng);
    let (publish_time, publish_bytes) = dummy_duration(rng);
    let (last_handshake, handshake_bytes) = dummy_duration(rng);

    let bytes = join_bytes!(seqid_bytes, capture_bytes, publish_bytes, handshake_bytes);

    (
        PhotonPipelineMetadata {
            seqid,
            capture_time,
            publish_time,
            last_handshake,
        },
        bytes,
    )
}

fn dummy_tracked_target(rng: &mut ThreadRng) -> (PhotonTrackedTarget, Vec<u8>) {
    let yaw: f64 = rng.gen();
    let pitch: f64 = rng.gen();
    let area: f64 = rng.gen();
    let skew: f64 = rng.gen();
    let (fiducial_id, fiducial_bytes) = dummy_fiducial_id(rng);
    let (detected, detected_bytes) = dummy_detected_object(rng);
    let (to_target, transforms_bytes) = dummy_target_transforms(rng);
    let ambiguity: f64 = rng.gen();
    let (area_rect_corners, area_corners_bytes) = dummy_vec(4, dummy_target_corner, rng);
    let (detected_corners, detected_corners_bytes) = dummy_vec(4, dummy_target_corner, rng);

    let bytes = join_bytes!(
        yaw.to_le_bytes(),
        pitch.to_le_bytes(),
        area.to_le_bytes(),
        skew.to_le_bytes(),
        fiducial_bytes,
        detected_bytes,
        transforms_bytes,
        ambiguity.to_le_bytes(),
        area_corners_bytes,
        detected_corners_bytes
    );

    (
        PhotonTrackedTarget {
            yaw,
            pitch,
            area,
            skew,
            fiducial_id,
            detected,
            to_target,
            ambiguity,
            area_rect_corners,
            detected_corners,
        },
        bytes,
    )
}

fn dummy_multi_target_pnp(rng: &mut ThreadRng) -> (MultiTargetPNP, Vec<u8>) {
    let (pnp, pnp_bytes) = dummy_pnp_result(rng);
    let (num_fiducials, num_fiducials_bytes) = dummy_u16(rng);

    let bytes = join_bytes!(pnp_bytes, num_fiducials_bytes);

    (MultiTargetPNP { pnp, num_fiducials }, bytes)
}

fn dummy_photon_result(rng: &mut ThreadRng) -> (PhotonResult, Vec<u8>) {
    let (metadata, metadata_bytes) = dummy_pipeline_metadata(rng);
    let (targets, targets_bytes) = dummy_vec(1, dummy_tracked_target, rng);
    let (pnp, pnp_bytes) = dummy_optional(rng, dummy_multi_target_pnp);

    let bytes = join_bytes!(metadata_bytes, targets_bytes, pnp_bytes);

    (
        PhotonResult {
            metadata,
            targets,
            pnp,
        },
        bytes,
    )
}

macro_rules! test_for {
    ($test:ident, $dummy:ident, $ty:ty) => {
        #[test]
        fn $test() {
            let mut rng = rand::thread_rng();
            let (good, bytes) = $dummy(&mut rng);
            let test = deserialize::<$ty>(&bytes);

            assert!(test.is_ok(), "{:?}", test.err());
            assert_eq!(good, test.unwrap());
        }
    };
}

test_for!(quaternion, dummy_quaternion, Quaternion);
test_for!(translate3d, dummy_translate3d, Translate3d);
test_for!(target_corner, dummy_target_corner, TargetCorner);
test_for!(detected_object, dummy_detected_object, DetectedObject);
test_for!(transform3d, dummy_transform3d, Transform3d);
test_for!(target_transforms, dummy_target_transforms, TargetTransforms);
test_for!(pnp_result, dummy_pnp_result, PNPResult);
test_for!(metadata, dummy_pipeline_metadata, PhotonPipelineMetadata);
test_for!(tracked_target, dummy_tracked_target, PhotonTrackedTarget);
test_for!(multi_target_pnp, dummy_multi_target_pnp, MultiTargetPNP);
test_for!(photon_result, dummy_photon_result, PhotonResult);
