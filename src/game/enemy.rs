use crate::prelude::*;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataPoint {
    pub time: Instant,
    pub pose: Pose2d,
    pub size: (Length, Length),
    pub confidence: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Enemy {
    pub id: u8,
    pub history: Vec<DataPoint>,
}

impl Enemy {
    /// Gets the velocity between history point n and n+1. If n == 0,
    /// it will return the most recent velocity.
    ///
    /// ((vx, vy), dt) or ((0m/s, 0m/s), 0s) if < n+2 points
    pub fn velocity(&self, n: usize) -> ((Velocity, Velocity), Time) {
        if self.history.len() < n + 2 {
            return Default::default();
        }

        // get the last two points
        let DataPoint {
            time: t2,
            pose:
                Pose2d {
                    translate: Translate2d { x: x2, y: y2 },
                    ..
                },
            ..
        } = self.entry(n);

        let DataPoint {
            time: t1,
            pose:
                Pose2d {
                    translate: Translate2d { x: x1, y: y1 },
                    ..
                },
            ..
        } = self.entry(n);

        let dt = t2 - t1;
        let dt = Time::new::<second>(dt.as_secs_f64());

        let dx = x2 - x1;
        let dy = y2 - y1;
        let vx = dx / dt;
        let vy = dy / dt;

        ((vx, vy), dt)
    }

    /// Gets the acceleration between velocities n and n+1. If n == 0,
    /// it will return the most recent acceleration.
    ///
    /// ((ax, ay), dt) or ((0m/s^2, 0m/s^2), 0s) if < n+3 points
    pub fn acceleration(&self, n: usize) -> ((Acceleration, Acceleration), Time) {
        if self.history.len() < n + 3 {
            return Default::default();
        }

        // get the last two velocities
        let ((vx2, vy2), dt2) = self.velocity(n);
        let ((vx1, vy1), dt1) = self.velocity(n + 1);

        let dvx = vx2 - vx1;
        let dvy = vy2 - vy1;
        let ax = dvx / dt2;
        let ay = dvy / dt2;

        ((ax, ay), dt2)
    }

    pub fn entry(&self, n: usize) -> DataPoint {
        self.history[self.history.len() - n - 1]
    }

    pub fn pose(&self) -> Pose2d {
        self.history.last().unwrap().pose
    }

    pub fn last_update(&self) -> Instant {
        self.history.last().unwrap().time
    }

    pub fn add_dp(&mut self, dp: DataPoint) {
        self.history.push(dp);
    }
}
