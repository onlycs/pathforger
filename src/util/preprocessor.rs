use crate::prelude::*;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreprocessorResponse {
    pub enemies: Vec<Pose2d>,
    pub timestamp: Instant,
}

/// When finished, this should take the `res` and process it into
/// a list of `Pose2d` objects representing the center positions of enemy robots
pub async fn photon(res: PhotonResult) -> PreprocessorResponse {
    let timestamp = time::instant_of(res.metadata.capture_time);

    unimplemented!()
}
