// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Speed curves: the arithmetic the commands share.
//!
//! A curve is a handful of `(at, speed)` points over the clip, joined by
//! straight lines; the engine's [`SpeedCurve`] turns them into a time map.

use concat_core::SpeedCurve;

use crate::model::SpeedPoint;

/// The engine's curve for these points, or None when they make no curve.
pub fn curve_of(points: &[SpeedPoint]) -> Option<SpeedCurve> {
    let raw: Vec<(f64, f64)> = points
        .iter()
        .map(|point: &SpeedPoint| (point.at, point.speed))
        .collect();
    SpeedCurve::new(&raw)
}

/// Source seconds per timeline second over a whole clip with these points;
/// the constant rate's equivalent for a curve.
pub fn mean_of(points: &[SpeedPoint]) -> f64 {
    curve_of(points).map(|curve| curve.mean()).unwrap_or(1.0)
}

/// How many points a hand-drawn speed curve holds at most, matching the
/// colour curves'.
pub const MAX_SPEED_POINTS: usize = 8;

/// The graph's height for a speed, on a log scale: the engine's whole
/// range, 1/16x to 16x, fits the unit square with 1x in the middle.
pub fn y_of_speed(speed: f64) -> f64 {
    (speed.clamp(SpeedCurve::MIN_SPEED, SpeedCurve::MAX_SPEED).log2() + 4.0) / 8.0
}

/// The speed a graph height stands for, the reverse of [`y_of_speed`].
pub fn speed_of_y(y: f64) -> f64 {
    2f64.powf(y.clamp(0.0, 1.0) * 8.0 - 4.0)
}

/// Puts point `point` of the curve at `(at, speed)`, or a new point there
/// when `point` is negative. A curve with nothing yet gets its two ends
/// first, flat at 1x, so the new point bends it rather than making the
/// whole curve; a curve that already holds [`MAX_SPEED_POINTS`] takes no
/// more. The stored order is the order points were put down, so a point
/// being dragged keeps its number while it crosses another; the engine
/// sorts them by `at` when it reads the curve.
pub fn set_point(points: &[SpeedPoint], point: i32, at: f64, speed: f64) -> Option<Vec<SpeedPoint>> {
    let mut points = points.to_vec();
    let at = at.clamp(0.0, 1.0);
    let speed = speed.clamp(SpeedCurve::MIN_SPEED, SpeedCurve::MAX_SPEED);
    match usize::try_from(point) {
        Ok(index) if index < points.len() => {
            points[index] = SpeedPoint { at, speed };
        }
        Ok(_) => return None,
        Err(_) => {
            if points.is_empty() {
                points = vec![
                    SpeedPoint { at: 0.0, speed: 1.0 },
                    SpeedPoint { at: 1.0, speed: 1.0 },
                ];
            }
            if points.len() >= MAX_SPEED_POINTS {
                return None;
            }
            points.push(SpeedPoint { at, speed });
        }
    }
    Some(points)
}

/// Takes point `point` off the curve, `None` for a number the curve does
/// not hold. What is left may be fewer than two points; the caller reads
/// that as a constant rate again.
pub fn remove_point(points: &[SpeedPoint], point: i32) -> Option<Vec<SpeedPoint>> {
    let Ok(index) = usize::try_from(point) else {
        return None;
    };
    if index >= points.len() {
        return None;
    }
    let mut points = points.to_vec();
    points.remove(index);
    Some(points)
}

/// The curve as the graph draws it: a path through the tidied points in a
/// unit box, y downwards, straight lines between them. Empty when the
/// points make no curve.
pub fn path_of(points: &[SpeedPoint]) -> String {
    let Some(curve) = curve_of(points) else {
        return String::new();
    };
    let mut path = String::new();
    for (index, &(at, speed)) in curve.points().iter().enumerate() {
        let verb = if index == 0 { 'M' } else { 'L' };
        path.push_str(&format!("{verb} {at:.4} {:.4} ", 1.0 - y_of_speed(speed)));
    }
    path.trim_end().to_owned()
}

/// The Speed panel's curve presets, by the index it sends: 0 is a
/// constant rate (no curve), then Ease In, Ease Out, Bullet Time, Hero
/// Moment, Flash, and 6 Custom - flat at 1x, the line the graph bends.
/// A preset is points of `(at, speed)` over the clip, joined by straight
/// lines; the engine tidies them - sorted, clamped, anchored at both
/// ends - so a preset means the same on any clip, whatever its length.
pub fn preset_points(index: i32) -> Option<Vec<SpeedPoint>> {
    let raw: &[(f64, f64)] = match index {
        // A constant rate is the curve going away; an empty list is
        // what the command takes for that, and it stores as None.
        0 => return Some(Vec::new()),
        // Slow in, fast out: the push-in that opens a shot.
        1 => &[(0.0, 0.5), (1.0, 2.0)],
        // Fast in, slow out: the settle at the end of a move.
        2 => &[(0.0, 2.0), (1.0, 0.5)],
        // Fast, then a slow middle, then fast again: the bullet-time
        // pause on the moment in the middle.
        3 => &[(0.0, 2.0), (0.4, 0.25), (0.6, 0.25), (1.0, 2.0)],
        // Slow through most of the clip, then a rush at the end: the
        // hero-moment reveal.
        4 => &[(0.0, 0.25), (0.7, 0.5), (1.0, 3.0)],
        // Steady, with a quick burst in the middle: the flash of a
        // cut or a beat.
        5 => &[(0.0, 1.0), (0.45, 1.0), (0.55, 4.0), (0.65, 1.0), (1.0, 1.0)],
        // Flat at 1x: the custom curve's starting line.
        6 => &[(0.0, 1.0), (1.0, 1.0)],
        _ => return None,
    };
    Some(
        raw.iter()
            .map(|&(at, speed)| SpeedPoint { at, speed })
            .collect(),
    )
}

/// Which preset these points are, by the panel's numbering: 0 for no
/// curve at all, 6 for the flat custom line, -1 for a curve no preset
/// makes.
pub fn preset_index_of(points: &[SpeedPoint]) -> i32 {
    if points.is_empty() {
        return 0;
    }
    for index in 1..=6 {
        let Some(preset) = preset_points(index) else {
            continue;
        };
        if preset.len() == points.len()
            && preset.iter().zip(points.iter()).all(|(want, got)| {
                (want.at - got.at).abs() < 1e-9 && (want.speed - got.speed).abs() < 1e-9
            })
        {
            return index;
        }
    }
    -1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_round_trip() {
        assert_eq!(preset_points(0).unwrap().len(), 0);
        assert_eq!(preset_index_of(&[]), 0);
        for index in 1..=6 {
            let points = preset_points(index).unwrap();
            assert_eq!(preset_index_of(&points), index);
            let mean = mean_of(&points);
            assert!(mean >= SpeedCurve::MIN_SPEED && mean <= SpeedCurve::MAX_SPEED);
        }
        assert!(preset_points(7).is_none());
        assert!(preset_points(-1).is_none());
        assert_eq!(
            preset_index_of(&[SpeedPoint { at: 0.0, speed: 1.0 }, SpeedPoint { at: 1.0, speed: 1.0 }]),
            6
        );
        assert_eq!(
            preset_index_of(&[
                SpeedPoint { at: 0.0, speed: 0.5 },
                SpeedPoint { at: 0.5, speed: 1.0 },
                SpeedPoint { at: 1.0, speed: 2.0 },
            ]),
            -1
        );
    }

    #[test]
    fn y_maps_speed_logarithmically() {
        assert!((y_of_speed(0.0625) - 0.0).abs() < 1e-9);
        assert!((y_of_speed(1.0) - 0.5).abs() < 1e-9);
        assert!((y_of_speed(16.0) - 1.0).abs() < 1e-9);
        for speed in [0.0625, 0.25, 1.0, 4.0, 16.0] {
            assert!((speed_of_y(y_of_speed(speed)) - speed).abs() < 1e-9);
        }
    }

    #[test]
    fn points_are_put_down_moved_and_taken_off() {
        let seeded = set_point(&[], -1, 0.5, 2.0).unwrap();
        assert_eq!(seeded.len(), 3);
        assert_eq!(seeded[0], SpeedPoint { at: 0.0, speed: 1.0 });
        assert_eq!(seeded[1], SpeedPoint { at: 1.0, speed: 1.0 });
        assert_eq!(seeded[2], SpeedPoint { at: 0.5, speed: 2.0 });
        let moved = set_point(&seeded, 2, 0.25, 0.5).unwrap();
        assert_eq!(moved[2], SpeedPoint { at: 0.25, speed: 0.5 });
        assert!(set_point(&moved, 9, 0.5, 1.0).is_none());
        let one = remove_point(&moved, 0).unwrap();
        assert_eq!(one.len(), 2);
        assert_eq!(remove_point(&one, 0).unwrap().len(), 1);
        assert!(remove_point(&one, 5).is_none());
        let mut full = seeded.clone();
        for _ in 0..5 {
            full = set_point(&full, -1, 0.5, 1.0).unwrap();
        }
        assert_eq!(full.len(), 8);
        assert!(set_point(&full, -1, 0.5, 1.0).is_none());
    }

    #[test]
    fn path_is_a_straight_line_through_the_points() {
        assert_eq!(
            path_of(&preset_points(1).unwrap()),
            "M 0.0000 0.6250 L 1.0000 0.3750"
        );
        assert!(path_of(&[]).is_empty());
    }
}
