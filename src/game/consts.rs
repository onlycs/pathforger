use crate::prelude::*;

// http://firstfrc.blob.core.windows.net/frc2024/Manual/Sections/2024GameManual-05ARENA.pdf
pub const FIELD_WIDTH: fn() -> Length = || Length::new::<meter>(8.21);
pub const FIELD_LENGTH: fn() -> Length = || Length::new::<meter>(16.54);

// TODO: better estimations
// Units in m/s
pub const MAX_ACCEL: fn() -> Acceleration = || Acceleration::new::<mps2>(5.0);
pub const MAX_SPEED: fn() -> Velocity = || Velocity::new::<mps>(14.0);
