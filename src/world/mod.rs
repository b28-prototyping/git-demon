pub mod objects;
pub mod speed;

use std::collections::VecDeque;

use self::objects::RoadsideObject;
use self::speed::VelocityTier;
use crate::git::seed::RepoSeed;
use crate::git::PollResult;

pub struct WorldState {
    pub z_offset: f32,
    pub camera_z: f32,
    pub speed: f32,
    pub speed_target: f32,
    pub commits_per_min: f32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub files_changed: u32,
    pub tier: VelocityTier,
    pub time: f32,
    pub total_commits: u64,
    pub pending_objects: VecDeque<RoadsideObject>,
    pub active_objects: Vec<(objects::Lane, f32, RoadsideObject)>,
    pub curve_offset: f32,
    pub curve_target: f32,
    pub steer_angle: f32,
}

const SPAWN_DISTANCE: f32 = 100.0;
const NEAR_SPAWN: f32 = 30.0;
const DRAW_DISTANCE: f32 = 200.0;
const DESPAWN_BEHIND: f32 = 5.0;

impl WorldState {
    pub fn new(seed: &RepoSeed) -> Self {
        Self {
            z_offset: 0.0,
            camera_z: 0.0,
            speed: 1.5,
            speed_target: 1.5 + seed.speed_base * 2.8,
            commits_per_min: 0.0,
            lines_added: 0,
            lines_deleted: 0,
            files_changed: 0,
            tier: VelocityTier::Flatline,
            time: 0.0,
            total_commits: seed.total_commits,
            pending_objects: VecDeque::new(),
            active_objects: Vec::new(),
            curve_offset: 0.0,
            curve_target: 0.0,
            steer_angle: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;

        // Lerp speed toward target
        self.speed += (self.speed_target - self.speed) * 4.0 * dt;
        self.z_offset += self.speed * dt;
        self.camera_z += self.speed * dt;

        // Auto-steering: gentle sinusoidal weave for visual interest
        // Two overlapping sine waves at different frequencies for organic feel
        self.steer_angle = (self.time * 0.3).sin() * 40.0
            + (self.time * 0.13).sin() * 25.0
            + (self.time * 0.07).sin() * 15.0;

        // Combine auto-steer with activity-driven curve
        self.curve_offset += (self.curve_target + self.steer_angle - self.curve_offset) * 2.5 * dt;

        // Update tier
        self.tier = VelocityTier::from_commits_per_min(self.commits_per_min);
        self.speed_target = speed::speed_target(self.commits_per_min);

        // Spawn pending objects
        if !self.pending_objects.is_empty() {
            let count = self.pending_objects.len();
            let spacing = SPAWN_DISTANCE / count.max(1) as f32;
            let mut lane_toggle = false;
            let mut i = 0;
            while let Some(obj) = self.pending_objects.pop_front() {
                let z = self.camera_z + SPAWN_DISTANCE + spacing * i as f32;
                let lane = if matches!(obj, RoadsideObject::TierGate { .. }) {
                    objects::Lane::Center
                } else if lane_toggle {
                    objects::Lane::Right
                } else {
                    objects::Lane::Left
                };
                lane_toggle = !lane_toggle;
                self.active_objects.push((lane, z, obj));
                i += 1;
            }
        }

        // Despawn objects behind camera
        self.active_objects
            .retain(|(_, z, _)| *z > self.camera_z - DESPAWN_BEHIND);
    }

    pub fn ingest_poll(&mut self, result: &PollResult, seed: &RepoSeed) {
        self.commits_per_min = result.commits_per_min;
        self.lines_added = result.lines_added;
        self.lines_deleted = result.lines_deleted;
        self.files_changed = result.files_changed;

        let old_tier = self.tier;

        objects::ingest_poll_to_queue(result, seed, &mut self.pending_objects);

        let new_tier = VelocityTier::from_commits_per_min(result.commits_per_min);
        if new_tier != old_tier {
            // Spawn tier gate immediately
            self.active_objects.push((
                objects::Lane::Center,
                self.camera_z + NEAR_SPAWN,
                RoadsideObject::TierGate { tier: new_tier },
            ));
        }

        // Shift curve target on activity bursts
        if result.commits_per_min > 1.0 {
            use rand::RngExt;
            let mut rng = rand::rng();
            self.curve_target = rng.random_range(-60.0..60.0);
        }
    }

    pub fn tier_index(&self) -> u8 {
        self.tier as u8
    }

    pub fn draw_distance(&self) -> f32 {
        if self.tier == VelocityTier::VelocityDemon {
            DRAW_DISTANCE * 1.2
        } else {
            DRAW_DISTANCE
        }
    }

    pub fn sector(&self) -> u64 {
        self.total_commits / 100
    }
}
