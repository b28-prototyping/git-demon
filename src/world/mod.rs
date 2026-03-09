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
    pub speed_multiplier: f32,
    pub curve_multiplier: f32,
    /// How long Up/Down has been held continuously (for exponential ramp)
    pub speed_hold_time: f32,
    /// How long Left/Right has been held continuously (for exponential ramp)
    pub curve_hold_time: f32,
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
            speed: 30.0,
            speed_target: 30.0 + seed.speed_base * 210.0,
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
            speed_multiplier: 1.0,
            curve_multiplier: 1.0,
            speed_hold_time: 0.0,
            curve_hold_time: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;

        // Lerp speed toward target (with user multiplier)
        let effective_target = self.speed_target * self.speed_multiplier;
        self.speed += (effective_target - self.speed) * 4.0 * dt;
        self.z_offset += self.speed * dt;
        self.camera_z += self.speed * dt;

        // Auto-steering: gentle sinusoidal weave for visual interest
        // Two overlapping sine waves at different frequencies for organic feel
        let base_steer = (self.time * 0.3).sin() * 40.0
            + (self.time * 0.13).sin() * 25.0
            + (self.time * 0.07).sin() * 15.0;
        self.steer_angle = base_steer * self.curve_multiplier;

        // Combine auto-steer with activity-driven curve
        let curve_target = self.curve_target * self.curve_multiplier;
        self.curve_offset += (curve_target + self.steer_angle - self.curve_offset) * 2.5 * dt;

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
                } else if matches!(obj, RoadsideObject::CommitCar { .. }) {
                    if lane_toggle {
                        objects::Lane::RoadRight
                    } else {
                        objects::Lane::RoadLeft
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::poller::{CommitSummary, PollResult};
    use crate::git::seed::RepoSeed;
    use image::Rgba;
    use std::collections::HashMap;

    fn test_seed() -> RepoSeed {
        RepoSeed {
            accent_hue: 180.0,
            saturation: 0.8,
            terrain_roughness: 0.5,
            speed_base: 0.5,
            author_colors: {
                let mut m = HashMap::new();
                m.insert("Alice".to_string(), Rgba([100, 200, 100, 255]));
                m
            },
            total_commits: 250,
            repo_name: "test-repo".to_string(),
        }
    }

    fn empty_poll(cpm: f32) -> PollResult {
        PollResult {
            commits: Vec::new(),
            commits_per_min: cpm,
            lines_added: 0,
            lines_deleted: 0,
            files_changed: 0,
            window_minutes: 30,
            polled_at: chrono::Utc::now(),
        }
    }

    fn poll_with_commit(
        message: &str,
        author: &str,
        added: u32,
        deleted: u32,
        cpm: f32,
    ) -> PollResult {
        PollResult {
            commits: vec![CommitSummary {
                sha_short: "abc1234".to_string(),
                message: message.to_string(),
                author: author.to_string(),
                lines_added: added,
                lines_deleted: deleted,
                files_changed: 1,
                timestamp: chrono::Utc::now(),
            }],
            commits_per_min: cpm,
            lines_added: added,
            lines_deleted: deleted,
            files_changed: 1,
            window_minutes: 30,
            polled_at: chrono::Utc::now(),
        }
    }

    // --- WorldState::new ---

    #[test]
    fn test_new_defaults() {
        let seed = test_seed();
        let w = WorldState::new(&seed);
        assert_eq!(w.z_offset, 0.0);
        assert_eq!(w.camera_z, 0.0);
        assert!((w.speed - 30.0).abs() < 0.001);
        assert!((w.speed_target - (30.0 + 0.5 * 210.0)).abs() < 0.001);
        assert_eq!(w.commits_per_min, 0.0);
        assert_eq!(w.tier, VelocityTier::Flatline);
        assert_eq!(w.total_commits, 250);
        assert!(w.pending_objects.is_empty());
        assert!(w.active_objects.is_empty());
    }

    // --- WorldState::update ---

    #[test]
    fn test_update_speed_lerp() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        // Set cpm so speed_target remains high after recomputation
        // speed_target(2.0) = 30.0 + 420.0 = 450.0
        w.commits_per_min = 2.0;
        w.speed = 1.0;
        let old_speed = w.speed;
        w.update(0.1);
        assert!(w.speed > old_speed, "speed should increase toward target");
        assert!(w.speed < 450.0, "speed should not overshoot target");
    }

    #[test]
    fn test_update_z_advances() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        w.speed = 2.0;
        w.speed_target = 2.0;
        let dt = 0.5;
        w.update(dt);
        // z_offset and camera_z advance by speed * dt = 2.0 * 0.5 = 1.0
        // (speed changes slightly during update due to lerp, but target==speed so minimal)
        assert!(w.z_offset > 0.0);
        assert!(w.camera_z > 0.0);
    }

    #[test]
    fn test_update_time_advances() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        w.update(0.016);
        assert!((w.time - 0.016).abs() < 0.0001);
    }

    #[test]
    fn test_update_tier_recomputed() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        w.commits_per_min = 2.0;
        w.update(0.016);
        assert_eq!(w.tier, VelocityTier::Demon);
    }

    #[test]
    fn test_update_despawn() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        // Place an object behind camera
        w.active_objects.push((
            objects::Lane::Left,
            -10.0, // well behind camera_z=0
            RoadsideObject::VelocitySign {
                commits_per_min: 1.0,
            },
        ));
        assert_eq!(w.active_objects.len(), 1);
        w.update(0.016);
        assert_eq!(
            w.active_objects.len(),
            0,
            "object behind camera should be despawned"
        );
    }

    #[test]
    fn test_update_spawn_pending() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        w.pending_objects.push_back(RoadsideObject::VelocitySign {
            commits_per_min: 1.0,
        });
        w.pending_objects.push_back(RoadsideObject::VelocitySign {
            commits_per_min: 2.0,
        });
        assert_eq!(w.active_objects.len(), 0);
        w.update(0.016);
        assert_eq!(
            w.active_objects.len(),
            2,
            "pending objects should be spawned"
        );
        assert!(
            w.pending_objects.is_empty(),
            "pending queue should be drained"
        );
    }

    #[test]
    fn test_lane_alternation() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        for _ in 0..4 {
            w.pending_objects.push_back(RoadsideObject::VelocitySign {
                commits_per_min: 1.0,
            });
        }
        w.update(0.016);
        let lanes: Vec<_> = w.active_objects.iter().map(|(l, _, _)| *l).collect();
        assert_eq!(lanes[0], objects::Lane::Left);
        assert_eq!(lanes[1], objects::Lane::Right);
        assert_eq!(lanes[2], objects::Lane::Left);
        assert_eq!(lanes[3], objects::Lane::Right);
    }

    // --- WorldState::ingest_poll ---

    #[test]
    fn test_ingest_poll_creates_commit_car() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        let poll = poll_with_commit("fix bug", "Alice", 10, 5, 0.1);
        w.ingest_poll(&poll, &seed);
        let has_car = w
            .pending_objects
            .iter()
            .any(|o| matches!(o, RoadsideObject::CommitCar { .. }));
        assert!(has_car, "commit should produce CommitCar");
    }

    #[test]
    fn test_lane_assignment_commit_car() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        w.pending_objects.push_back(RoadsideObject::CommitCar {
            message: "test".to_string(),
            author: "Alice".to_string(),
            author_color: Rgba([100, 200, 100, 255]),
        });
        w.pending_objects.push_back(RoadsideObject::VelocitySign {
            commits_per_min: 1.0,
        });
        w.update(0.016);
        let car_lane = w
            .active_objects
            .iter()
            .find(|(_, _, o)| matches!(o, RoadsideObject::CommitCar { .. }))
            .map(|(l, _, _)| *l);
        let sign_lane = w
            .active_objects
            .iter()
            .find(|(_, _, o)| matches!(o, RoadsideObject::VelocitySign { .. }))
            .map(|(l, _, _)| *l);
        assert!(
            car_lane == Some(objects::Lane::RoadLeft) || car_lane == Some(objects::Lane::RoadRight),
            "CommitCar should be on road lane, got {:?}",
            car_lane
        );
        assert!(
            sign_lane == Some(objects::Lane::Left) || sign_lane == Some(objects::Lane::Right),
            "VelocitySign should be on verge lane, got {:?}",
            sign_lane
        );
    }

    #[test]
    fn test_ingest_poll_creates_addition_tower() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        let poll = poll_with_commit("big feature", "Alice", 100, 0, 0.1);
        w.ingest_poll(&poll, &seed);
        let has_tower = w
            .pending_objects
            .iter()
            .any(|o| matches!(o, RoadsideObject::AdditionTower { .. }));
        assert!(has_tower, ">50 lines_added should produce AdditionTower");
    }

    #[test]
    fn test_ingest_poll_no_tower_small_addition() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        let poll = poll_with_commit("small fix", "Alice", 30, 0, 0.1);
        w.ingest_poll(&poll, &seed);
        let has_tower = w
            .pending_objects
            .iter()
            .any(|o| matches!(o, RoadsideObject::AdditionTower { .. }));
        assert!(
            !has_tower,
            "<=50 lines_added should not produce AdditionTower"
        );
    }

    #[test]
    fn test_ingest_poll_creates_deletion_shard() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        let poll = poll_with_commit("cleanup", "Alice", 0, 80, 0.1);
        w.ingest_poll(&poll, &seed);
        let has_shard = w
            .pending_objects
            .iter()
            .any(|o| matches!(o, RoadsideObject::DeletionShard { .. }));
        assert!(has_shard, ">50 lines_deleted should produce DeletionShard");
    }

    #[test]
    fn test_ingest_poll_tier_gate_on_change() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        assert_eq!(w.tier, VelocityTier::Flatline);
        let poll = empty_poll(0.5); // Active tier
        w.ingest_poll(&poll, &seed);
        let has_gate = w
            .active_objects
            .iter()
            .any(|(_, _, o)| matches!(o, RoadsideObject::TierGate { .. }));
        assert!(
            has_gate,
            "tier change should spawn TierGate in active_objects"
        );
        // TierGate should be at camera_z + NEAR_SPAWN
        let gate_z = w
            .active_objects
            .iter()
            .find_map(|(_, z, o)| matches!(o, RoadsideObject::TierGate { .. }).then_some(*z))
            .unwrap();
        assert!(
            (gate_z - NEAR_SPAWN).abs() < 0.1,
            "TierGate should be at NEAR_SPAWN"
        );
    }

    #[test]
    fn test_ingest_poll_no_tier_gate_same_tier() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        // First poll: set tier to Active
        let poll1 = empty_poll(0.5);
        w.ingest_poll(&poll1, &seed);
        w.tier = VelocityTier::Active; // sync tier

        // Second poll: same cpm, same tier
        let poll2 = empty_poll(0.5);
        let gates_before = w
            .active_objects
            .iter()
            .filter(|(_, _, o)| matches!(o, RoadsideObject::TierGate { .. }))
            .count();
        w.ingest_poll(&poll2, &seed);
        let gates_after = w
            .active_objects
            .iter()
            .filter(|(_, _, o)| matches!(o, RoadsideObject::TierGate { .. }))
            .count();
        assert_eq!(
            gates_after, gates_before,
            "same tier should not spawn another TierGate"
        );
    }

    #[test]
    fn test_ingest_poll_curve_shift_on_burst() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        assert_eq!(w.curve_target, 0.0);
        let poll = empty_poll(1.5); // > 1.0, should trigger curve shift
        w.ingest_poll(&poll, &seed);
        // curve_target is randomized, just check it's in range
        assert!(w.curve_target >= -60.0 && w.curve_target <= 60.0);
    }

    #[test]
    fn test_ingest_poll_no_curve_shift_low_cpm() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        let poll = empty_poll(0.5); // <= 1.0
        w.ingest_poll(&poll, &seed);
        assert_eq!(w.curve_target, 0.0, "low cpm should not shift curve_target");
    }

    // --- draw_distance ---

    #[test]
    fn test_draw_distance_normal() {
        let seed = test_seed();
        let w = WorldState::new(&seed);
        assert!((w.draw_distance() - 200.0).abs() < 0.1);
    }

    #[test]
    fn test_draw_distance_velocity_demon() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        w.tier = VelocityTier::VelocityDemon;
        assert!((w.draw_distance() - 240.0).abs() < 0.1);
    }

    // --- sector ---

    #[test]
    fn test_sector() {
        let seed = test_seed();
        let w = WorldState::new(&seed);
        assert_eq!(w.sector(), 2); // 250 / 100 = 2
    }

    // --- tier_index ---

    #[test]
    fn test_tier_index() {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        w.tier = VelocityTier::Demon;
        assert_eq!(w.tier_index(), 3);
    }
}
