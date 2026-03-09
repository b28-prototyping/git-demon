/// Per-row road projection table with cumulative curve and slope offsets.
/// Built once per frame from road segments + camera, consumed by all renderers.
use crate::world::camera::Camera;
use crate::world::road_segments::{RoadSegment, SEGMENT_LENGTH};

/// Precomputed road properties for a single scanline row.
#[derive(Debug, Clone, Copy)]
pub struct RoadRow {
    /// Cumulative horizontal curve offset in pixels.
    pub curve_offset: f32,
    /// Cumulative vertical slope offset in pixels.
    pub slope_offset: f32,
    /// 1/z depth scale at this row (0..1].
    pub depth_scale: f32,
    /// World-Z coordinate at this row.
    pub z_world: f32,
}

/// Build a per-row lookup table for all scanlines below the horizon.
///
/// Returns a Vec indexed by `(screen_y - horizon_y)`. Entry 0 is the horizon row,
/// last entry is the bottom of the screen (nearest to camera).
pub fn build_road_table(
    camera: &Camera,
    segments: &[RoadSegment],
    camera_z: f32,
    segment_z_start: f32,
    screen_h: u32,
    horizon_y: u32,
) -> Vec<RoadRow> {
    let road_rows = (screen_h - horizon_y).max(1) as usize;
    let mut table = Vec::with_capacity(road_rows);

    for row_idx in 0..road_rows {
        let _y = horizon_y + row_idx as u32;
        // Invert the camera projection to get z_world at this row.
        // screen_y = horizon_y + (screen_h - horizon_y) * depth_scale
        // depth_scale = (y - horizon_y) / (screen_h - horizon_y)
        // z_rel = near_plane / depth_scale
        let depth_scale = (row_idx as f32 + 0.5) / road_rows as f32;
        if depth_scale < 0.001 {
            table.push(RoadRow {
                curve_offset: 0.0,
                slope_offset: 0.0,
                depth_scale: 0.0,
                z_world: camera_z + camera.draw_distance,
            });
            continue;
        }
        let z_rel = camera.near_plane / depth_scale;
        let z_world = camera_z + z_rel;

        // Accumulate curve and slope from segments between camera and z_world
        let (cum_curve, cum_slope) =
            accumulate_offsets(segments, segment_z_start, camera_z, z_world, depth_scale);

        table.push(RoadRow {
            curve_offset: cum_curve,
            slope_offset: cum_slope,
            depth_scale,
            z_world,
        });
    }

    table
}

/// Accumulate curve and slope offsets from all segments between `from_z` and `to_z`.
fn accumulate_offsets(
    segments: &[RoadSegment],
    segment_z_start: f32,
    from_z: f32,
    to_z: f32,
    _depth_scale: f32,
) -> (f32, f32) {
    let mut cum_curve = 0.0f32;
    let mut cum_slope = 0.0f32;

    for (i, seg) in segments.iter().enumerate() {
        let seg_start = segment_z_start + i as f32 * SEGMENT_LENGTH;
        let seg_end = seg_start + SEGMENT_LENGTH;

        // Only consider segments that overlap with (from_z, to_z)
        if seg_end <= from_z || seg_start >= to_z {
            continue;
        }

        // Clamp to the range we care about
        let start = seg_start.max(from_z);
        let end = seg_end.min(to_z);
        let frac = (end - start) / SEGMENT_LENGTH;

        cum_curve += seg.curve * frac;
        cum_slope += seg.slope * frac;
    }

    // Scale offsets by a visual factor so they produce visible pixel shifts.
    // Curve: already in pixel-like units from generation (~0-65 range).
    // Slope: convert to pixels — multiply by a screen-height-proportional factor.
    let slope_px = cum_slope * 40.0;

    (cum_curve, slope_px)
}

/// Look up road table values for a given world-Z coordinate.
/// Returns `Some((curve_offset, slope_offset, depth_scale))` or None if z is out of range.
pub fn lookup_at_z(
    table: &[RoadRow],
    z_world: f32,
    camera_z: f32,
    horizon_y: u32,
) -> Option<(f32, f32, f32)> {
    if table.is_empty() {
        return None;
    }

    // Table is indexed 0..road_rows where 0=horizon (far), last=bottom (near).
    // z_world decreases from entry 0 to last entry.
    // Find the entry with closest z_world.

    // Binary search: table[0].z_world is largest, table[last].z_world is smallest.
    if z_world > table[0].z_world || z_world < table[table.len() - 1].z_world {
        return None;
    }

    // Linear scan from far to near (table is small, typically <200 entries)
    let mut best = 0;
    let mut best_dist = (table[0].z_world - z_world).abs();
    for (i, row) in table.iter().enumerate().skip(1) {
        let dist = (row.z_world - z_world).abs();
        if dist < best_dist {
            best_dist = dist;
            best = i;
        } else {
            break; // z_world is monotonically decreasing, so first minimum is the best
        }
    }

    let row = &table[best];
    let _ = (camera_z, horizon_y); // used for interface compatibility
    Some((row.curve_offset, row.slope_offset, row.depth_scale))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::camera::Camera;
    use crate::world::road_segments;

    fn flat_segments(n: usize) -> Vec<RoadSegment> {
        vec![
            RoadSegment {
                curve: 0.0,
                slope: 0.0,
            };
            n
        ]
    }

    fn hilly_segments(n: usize) -> Vec<RoadSegment> {
        (0..n)
            .map(|i| RoadSegment {
                curve: if i % 2 == 0 { 10.0 } else { -10.0 },
                slope: if i < n / 2 { 0.5 } else { -0.5 },
            })
            .collect()
    }

    #[test]
    fn test_table_size_matches_road_rows() {
        let cam = Camera::new();
        let segs = flat_segments(road_segments::SEGMENT_COUNT);
        let table = build_road_table(&cam, &segs, 0.0, 0.0, 200, 50);
        assert_eq!(table.len(), 150); // 200 - 50
    }

    #[test]
    fn test_flat_segments_zero_offsets() {
        let cam = Camera::new();
        let segs = flat_segments(road_segments::SEGMENT_COUNT);
        let table = build_road_table(&cam, &segs, 0.0, 0.0, 200, 50);
        for row in &table {
            assert!(
                row.curve_offset.abs() < 0.001,
                "flat segments should have zero curve: {}",
                row.curve_offset
            );
            assert!(
                row.slope_offset.abs() < 0.001,
                "flat segments should have zero slope: {}",
                row.slope_offset
            );
        }
    }

    #[test]
    fn test_z_world_monotonically_decreasing() {
        let cam = Camera::new();
        let segs = flat_segments(road_segments::SEGMENT_COUNT);
        let table = build_road_table(&cam, &segs, 0.0, 0.0, 200, 50);
        for i in 1..table.len() {
            assert!(
                table[i].z_world < table[i - 1].z_world,
                "z_world should decrease: row {} z={} >= row {} z={}",
                i,
                table[i].z_world,
                i - 1,
                table[i - 1].z_world
            );
        }
    }

    #[test]
    fn test_depth_scale_monotonically_increasing() {
        let cam = Camera::new();
        let segs = flat_segments(road_segments::SEGMENT_COUNT);
        let table = build_road_table(&cam, &segs, 0.0, 0.0, 200, 50);
        for i in 1..table.len() {
            assert!(
                table[i].depth_scale >= table[i - 1].depth_scale,
                "depth_scale should increase: row {} ds={} < row {} ds={}",
                i,
                table[i].depth_scale,
                i - 1,
                table[i - 1].depth_scale
            );
        }
    }

    #[test]
    fn test_hilly_segments_produce_slope_offset() {
        let cam = Camera::new();
        let segs = hilly_segments(road_segments::SEGMENT_COUNT);
        let table = build_road_table(&cam, &segs, 0.0, 0.0, 200, 50);
        // Far rows (index 0..10) should have accumulated some slope
        let has_slope = table.iter().any(|r| r.slope_offset.abs() > 0.1);
        assert!(
            has_slope,
            "hilly segments should produce non-zero slope offsets"
        );
    }

    #[test]
    fn test_hilly_segments_produce_curve_offset() {
        let cam = Camera::new();
        let segs = hilly_segments(road_segments::SEGMENT_COUNT);
        let table = build_road_table(&cam, &segs, 0.0, 0.0, 200, 50);
        let has_curve = table.iter().any(|r| r.curve_offset.abs() > 0.1);
        assert!(
            has_curve,
            "hilly segments should produce non-zero curve offsets"
        );
    }

    #[test]
    fn test_lookup_at_z_valid() {
        let cam = Camera::new();
        let segs = flat_segments(road_segments::SEGMENT_COUNT);
        let table = build_road_table(&cam, &segs, 0.0, 0.0, 200, 50);
        // Pick a z_world from the middle of the table
        let mid = &table[table.len() / 2];
        let result = lookup_at_z(&table, mid.z_world, 0.0, 50);
        assert!(result.is_some(), "lookup should succeed for valid z_world");
    }

    #[test]
    fn test_lookup_at_z_out_of_range() {
        let cam = Camera::new();
        let segs = flat_segments(road_segments::SEGMENT_COUNT);
        let table = build_road_table(&cam, &segs, 0.0, 0.0, 200, 50);
        // Way beyond draw distance
        let result = lookup_at_z(&table, 99999.0, 0.0, 50);
        assert!(result.is_none(), "lookup beyond range should return None");
    }

    #[test]
    fn test_lookup_returns_correct_values() {
        let cam = Camera::new();
        let segs = hilly_segments(road_segments::SEGMENT_COUNT);
        let table = build_road_table(&cam, &segs, 0.0, 0.0, 200, 50);
        let mid = &table[table.len() / 2];
        let (curve, slope, ds) = lookup_at_z(&table, mid.z_world, 0.0, 50).unwrap();
        assert!((curve - mid.curve_offset).abs() < 0.1);
        assert!((slope - mid.slope_offset).abs() < 0.1);
        assert!((ds - mid.depth_scale).abs() < 0.01);
    }
}
