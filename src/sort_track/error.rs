use std::{backtrace::Backtrace, panic::Location};
use thiserror::Error;

use lapjv::LapJVError;
use ndarray::ShapeError;
use ndarray_linalg::error::LinalgError;

#[derive(Debug, Error)]
pub enum KalmanError {
    #[error("At {location}: NdArray Shaping Error:{source}\n")]
    ShapeError {
        #[from]
        source: ShapeError,
        backtrace: Backtrace,
        location: &'static Location<'static>,
    },

    #[error("At {location}: Linear Algebra Error:{source}\n")]
    LinAlg {
        #[from]
        source: LinalgError,
        backtrace: Backtrace,
        location: &'static Location<'static>,
    },
}

#[derive(Debug, Error)]
pub enum MotionTrackError {
    #[error("At {location}: Kalman Error:{source}\n")]
    KalmanError {
        #[from]
        source: KalmanError,
        backtrace: Backtrace,
        location: &'static Location<'static>,
    },

    #[error("At {location}: NdArray Shaping Error:{source}\n")]
    ShapeError {
        #[from]
        source: ShapeError,
        backtrace: Backtrace,
        location: &'static Location<'static>,
    },

    #[error("At {location}: LapJV Error:{source}\n")]
    LapJVError {
        #[from]
        source: LapJVError,
        backtrace: Backtrace,
        location: &'static Location<'static>,
    },
}
