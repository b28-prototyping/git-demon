//! Road segment data for pseudo-3D hills and per-segment curvature.
//! Each segment represents a fixed-length stretch of road with its own
//! curve (horizontal) and slope (vertical) values.

/// World-Z length of each road segment.
pub const SEGMENT_LENGTH: f32 = 125.0;

/// Number of segments to keep ahead of the camera.
pub const SEGMENT_COUNT: usize = 40;

#[derive(Debug, Clone, Copy)]
pub struct RoadSegment {
    /// Horizontal curvature per unit Z (positive = road bends right).
    pub curve: f32,
    /// Vertical slope: positive = uphill, negative = downhill.
    pub slope: f32,
}

/// Generate a road segment for the stretch starting at `z_start`.
/// `commits_per_min` controls hill drama; `time` adds temporal variation.
pub fn generate_segment(z_start: f32, commits_per_min: f32, time: f32) -> RoadSegment {
    // Curvature: sine-wave pattern similar to the existing auto-steer
    let curve = (z_start * 0.0008 + time * 0.05).sin() * 40.0
        + (z_start * 0.0003 + time * 0.02).sin() * 25.0;

    // Slope: gentle rolling by default, more dramatic with git activity
    let base_amplitude = 0.3;
    let activity_boost = (commits_per_min / 4.0).clamp(0.0, 1.0) * 0.5;
    let amplitude = base_amplitude + activity_boost;
    let slope = (z_start * 0.001).sin() * amplitude + (z_start * 0.0004).sin() * amplitude * 0.5;

    RoadSegment { curve, slope }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_segment_returns_bounded_values() {
        for i in 0..100 {
            let z = i as f32 * SEGMENT_LENGTH;
            let seg = generate_segment(z, 0.0, 0.0);
            assert!(seg.curve.abs() < 100.0, "curve out of range: {}", seg.curve);
            assert!(seg.slope.abs() < 2.0, "slope out of range: {}", seg.slope);
        }
    }

    #[test]
    fn test_slope_scales_with_cpm() {
        let z = 1000.0;
        let _low = generate_segment(z, 0.0, 0.0);
        let _high = generate_segment(z, 4.0, 0.0);
        // High CPM should generally produce larger amplitude slopes
        // We test over multiple z values to ensure statistical correctness
        let mut low_total = 0.0f32;
        let mut high_total = 0.0f32;
        for i in 0..200 {
            let z = i as f32 * SEGMENT_LENGTH;
            low_total += generate_segment(z, 0.0, 0.0).slope.abs();
            high_total += generate_segment(z, 4.0, 0.0).slope.abs();
        }
        assert!(
            high_total > low_total,
            "high CPM slope sum ({high_total}) should exceed low CPM ({low_total})"
        );
    }

    #[test]
    fn test_segments_vary_with_z() {
        let a = generate_segment(0.0, 1.0, 0.0);
        let b = generate_segment(500.0, 1.0, 0.0);
        // Different z_start should produce different segments
        assert!(
            (a.curve - b.curve).abs() > 0.01 || (a.slope - b.slope).abs() > 0.01,
            "segments at different z should differ"
        );
    }

    #[test]
    fn test_segments_vary_with_time() {
        let a = generate_segment(1000.0, 1.0, 0.0);
        let b = generate_segment(1000.0, 1.0, 10.0);
        assert!(
            (a.curve - b.curve).abs() > 0.01,
            "segments at different times should have different curves"
        );
    }
}
