use crate::prelude::*;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EnemyRobot {
    pub id: u8,
    pub history: Vec<(Instant, Pose2d)>, // first..last timestamp
}

impl EnemyRobot {
    /// Up to n most recent velocities
    ///
    /// List of ((vx, vy), dt). Returns ((0, 0), 0) if there are not
    /// enough points
    fn recent_velocities(&self, n: usize) -> Vec<((Velocity, Velocity), Time)> {
        if self.history.len() < 2 {
            return vec![Default::default()];
        }

        // get up to the last n points
        let mut points = self.history.iter().rev().take(n + 1).collect::<Vec<_>>();
        points.reverse();

        let chunks = points.array_windows::<2>();
        let mut velocities = vec![];

        for chunk in chunks {
            let [(ts1, pose1), (ts2, pose2)] = chunk;

            let dt = ts2.duration_since(*ts1).as_secs_f64();
            let dt = Time::new::<second>(dt);

            let dx = pose2.translate.x - pose1.translate.x;
            let vx = dx / dt;

            let dy = pose2.translate.y - pose1.translate.y;
            let vy = dy / dt;

            velocities.push(((vx, vy), dt));
        }

        velocities
    }

    /// Up to n most recent angular velocities
    ///
    /// List of (omega, dt). Returns (0rad/s, 0s) if there are not
    /// enough points
    fn recent_angular_velocities(&self, n: usize) -> Vec<(AngularVelocity, Time)> {
        if self.history.len() < 2 {
            return vec![(AngularVelocity::new::<radps>(0.0), Time::new::<second>(0.0))];
        }

        // get up to the last n points
        let mut points = self.history.iter().rev().take(n + 1).collect::<Vec<_>>();
        points.reverse();

        let chunks = points.array_windows::<2>();
        let mut angular_velocities = vec![];

        for chunk in chunks {
            let [(ts1, pose1), (ts2, pose2)] = chunk;

            let dt = ts2.duration_since(*ts1).as_secs_f64();
            let dt = Time::new::<second>(dt);

            let dtheta = pose2.rotate.angle - pose1.rotate.angle;
            let omega = dtheta / dt;

            angular_velocities.push((omega.into(), dt));
        }

        angular_velocities
    }

    fn pose(&self) -> Pose2d {
        self.history.last().unwrap().1
    }

    fn last_update(&self) -> Instant {
        self.history.last().unwrap().0
    }
}

impl EnemyRobot {
    // TODO: better estimations
    // Units in m/s or rad/s
    const MAX_ACCELERATION: f64 = 7.0;
    const MAX_SPEED: f64 = 7.0;
    const MAX_ANGULAR_ACCEL: f64 = 90f64.to_radians();
    const MAX_ANGULAR_SPEED: f64 = 90f64.to_radians();
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipelineEnemies {
    pub enemies: Vec<EnemyRobot>,
}

impl PipelineEnemies {
    pub fn new() -> Self {
        Self { enemies: vec![] }
    }

    pub fn assign_enemy(&mut self, pose: Pose2d, ts: Instant) -> Option<u8> {
        // get the min distance from the pose to the enemies
        let enemy = self
            .enemies
            .iter_mut()
            .filter(|en| en.last_update() != ts) // no double updates
            .map(|en| {
                let dx = en.pose().translate.x - pose.translate.x;
                let dy = en.pose().translate.y - pose.translate.y;
                (dx * dx + dy * dy, en)
            })
            .min_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap())
            .map(|(_, en)| en);

        let Some(enemy) = enemy else {
            // no enemies registered yet
            self.enemies.push(EnemyRobot {
                id: self.enemies.len() as u8,
                history: vec![(ts, pose)],
            });

            return None;
        };

        // calculate the average speed and acceleration
        let vels = enemy.recent_velocities(5);

        let sum_vx = vels.iter().map(|((vx, _), _)| vx.get::<mps>()).sum::<f64>();
        let sum_vy = vels.iter().map(|((_, vy), _)| vy.get::<mps>()).sum::<f64>();

        let avg_vx = Velocity::new::<mps>(sum_vx / vels.len() as f64);
        let avg_vy = Velocity::new::<mps>(sum_vy / vels.len() as f64);

        let last_ts = enemy.history.last().unwrap().0;
        let dt = ts.duration_since(last_ts).as_secs_f64();
        let dt = Time::new::<second>(dt);

        let max_accel = Acceleration::new::<mps2>(EnemyRobot::MAX_ACCELERATION);

        let max_dvx = max_accel * dt;
        let max_dvy = max_accel * dt;

        let max_vx = avg_vx + max_dvx;
        let min_vx = avg_vx - max_dvx;
        let max_vy = avg_vy + max_dvy;
        let min_vy = avg_vy - max_dvy;

        let max_dx = max_vx * dt;
        let max_dy = max_vy * dt;
        let min_dx = min_vx * dt;
        let min_dy = min_vy * dt;

        let lastx = enemy.pose().translate.x;
        let lasty = enemy.pose().translate.y;

        let newx = pose.translate.x;
        let newy = pose.translate.y;

        let dx = newx - lastx;
        let dy = newy - lasty;

        if dx > max_dx || dx < min_dx || dy > max_dy || dy < min_dy {
            // this robot is new
            self.enemies.push(EnemyRobot {
                id: self.enemies.len() as u8,
                history: vec![(ts, pose)],
            });

            return None;
        }

        // update the enemy
        enemy.history.push((ts, pose));

        Some(enemy.id)
    }

    pub fn on_photon_update(
        &mut self,
        PreprocessorResponse { enemies, timestamp }: PreprocessorResponse,
    ) {
        // would use hashset, but can only have <= 3 enemies, wastes memory
        let updated_current = enemies
            .into_iter()
            .filter_map(|en| self.assign_enemy(en, timestamp))
            .collect::<Vec<_>>();

        for i in (0..self.enemies.len()).rev() {
            if !updated_current.contains(&self.enemies[i].id) {
                self.enemies.remove(i);
            }
        }
    }
}
