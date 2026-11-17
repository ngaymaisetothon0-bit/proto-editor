// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! A shape that keeps part of a clip's picture: a clip's mask.
//!
//! A mask is drawn analytically, as a signed distance from the shape's
//! edge, so it costs no texture and is sharp at any size. The distance is
//! measured in units of the picture's shorter side, which is what makes a
//! circle round on any aspect and a feather the same softness on a 720p
//! export and a 4K one. Negative is inside; the coverage falls from one to
//! zero across a band the feather sets, centred on the edge, and an
//! inverted mask keeps the outside instead.
//!
//! This is the one definition of the arithmetic. The GPU compositor runs a
//! line-for-line copy of it in WGSL (`concat-render`'s `MASK_WGSL`), and its
//! tests hold the two together; the CPU reference calls [`ShapeMask::coverage`]
//! directly. A change here is a change there.

/// Which shape a mask keeps. The order is the shader's numbering, one up:
/// zero there is "no mask".
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum MaskShape {
    /// One side of a straight line through the centre: the part above it,
    /// turned with the mask.
    Linear,
    /// A band either side of a line through the centre, `height` thick.
    Mirror,
    /// An ellipse filling the mask's box; a circle when it is square.
    #[default]
    Circle,
    /// The mask's box, its corners rounded by `roundness`.
    Rectangle,
    /// A heart filling the box, point down.
    Heart,
    /// A five-pointed star, point up.
    Star,
    /// A closed freehand polygon drawn in the picture.
    Custom,
}

impl MaskShape {
    /// Every shape, in the order a picker lists them.
    pub const ALL: [MaskShape; 7] = [
        MaskShape::Linear,
        MaskShape::Mirror,
        MaskShape::Circle,
        MaskShape::Rectangle,
        MaskShape::Heart,
        MaskShape::Star,
        MaskShape::Custom,
    ];

    /// The name a document writes. Stable for ever.
    pub fn name(self) -> &'static str {
        match self {
            MaskShape::Linear => "linear",
            MaskShape::Mirror => "mirror",
            MaskShape::Circle => "circle",
            MaskShape::Rectangle => "rectangle",
            MaskShape::Heart => "heart",
            MaskShape::Star => "star",
            MaskShape::Custom => "custom",
        }
    }

    /// The shape a document names, if this build knows it.
    pub fn parse(name: &str) -> Option<MaskShape> {
        if name.trim() == "custom" { return Some(MaskShape::Custom); }
        MaskShape::ALL
            .into_iter()
            .find(|shape| shape.name() == name.trim())
    }

    /// The shader's number for this shape: one up from its place in
    /// [`MaskShape::ALL`], zero being no mask at all.
    pub fn code(self) -> f32 {
        match self {
            MaskShape::Linear => 1.0,
            MaskShape::Mirror => 2.0,
            MaskShape::Circle => 3.0,
            MaskShape::Rectangle => 4.0,
            MaskShape::Heart => 5.0,
            MaskShape::Star => 6.0,
            MaskShape::Custom => 7.0,
        }
    }

    /// Whether the box's width means anything: a line has no width, and a
    /// band only a thickness.
    pub fn has_width(self) -> bool {
        !matches!(self, MaskShape::Linear | MaskShape::Mirror)
    }

    /// Whether the box's height means anything: a line has none.
    pub fn has_height(self) -> bool {
        self != MaskShape::Linear
    }
}

/// A mask, at one instant: everything the compositor needs to weigh a
/// pixel of the picture by it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ShapeMask {
    /// What it keeps.
    pub shape: MaskShape,
    /// The centre's offset from the picture's centre, in fractions of the
    /// picture's width; positive is right.
    pub x: f64,
    /// The same down the picture's height; positive is down.
    pub y: f64,
    /// The box's width, in the picture's shorter side.
    pub width: f64,
    /// The box's height, in the picture's shorter side; a band's thickness.
    pub height: f64,
    /// Clockwise degrees about the centre.
    pub rotation: f64,
    /// How round a rectangle's corners are: zero square, one as round as
    /// the box's shorter half allows.
    pub roundness: f64,
    /// How soft the edge is, `0..=1`: the band the coverage falls across,
    /// as a fraction of half the picture's shorter side.
    pub feather: f64,
    /// Keeps the outside of the shape instead of the inside.
    pub invert: bool,
    /// Vertices in picture UV coordinates; at most eight are retained.
    pub custom_points: [[f64; 2]; 8],
    /// Number of valid vertices in `custom_points`, clamped to eight.
    pub custom_point_count: u8,
}

impl Default for ShapeMask {
    /// A circle in the middle, two thirds of the picture's shorter side
    /// across, with a hard edge.
    fn default() -> Self {
        ShapeMask {
            shape: MaskShape::Circle,
            x: 0.0,
            y: 0.0,
            width: 0.66,
            height: 0.66,
            rotation: 0.0,
            roundness: 0.0,
            feather: 0.0,
            invert: false,
            custom_points: [[0.0; 2]; 8],
            custom_point_count: 0,
        }
    }
}

/// The parts of a mask the shader takes, packed as it takes them: seven
/// four-vectors, so a layer's mask rides along on its vertices.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MaskUniform {
    /// Shape code, invert (0 or 1), the feather band and the corner radius,
    /// the last two in short-side units.
    pub a: [f32; 4],
    /// The centre in fractions of the picture, from its centre, and the
    /// box's half width and half height in short-side units.
    pub b: [f32; 4],
    /// The cosine and sine of the turn, and the picture's width and height
    /// over its shorter side.
    pub c: [f32; 4],
    /// Four packed x/y pairs each for the custom polygon.
    pub d: [f32; 4],
    /// Custom polygon vertices five and six.
    pub e: [f32; 4],
    /// Custom polygon vertices seven and eight.
    pub f: [f32; 4],
    /// Reserved for alignment and future mask data.
    pub g: [f32; 4],
}

impl MaskUniform {
    /// No mask: every pixel kept.
    pub const NONE: MaskUniform = MaskUniform {
        a: [0.0; 4],
        b: [0.0; 4],
        c: [1.0, 0.0, 1.0, 1.0],
        d: [0.0; 4],
        e: [0.0; 4],
        f: [0.0; 4],
        g: [0.0; 4],
    };
}

impl ShapeMask {
    /// What the shader takes for this mask over a picture `width` by
    /// `height`, with `pixel` the size of one output pixel in the picture's
    /// shorter side: the narrowest band an edge may have, so a hard edge is
    /// still smoothed over a pixel rather than stair-stepped.
    pub fn uniform(&self, width: f32, height: f32, pixel: f32) -> MaskUniform {
        let short = width.min(height).max(1.0);
        let finite = |value: f64, fallback: f64| {
            if value.is_finite() {
                value as f32
            } else {
                fallback as f32
            }
        };
        let half_w = (finite(self.width, 0.66) / 2.0).max(1e-4);
        let half_h = (finite(self.height, 0.66) / 2.0).max(1e-4);
        let band = (finite(self.feather, 0.0).clamp(0.0, 1.0) * 0.5).max(pixel.max(1e-6));
        let radius = finite(self.roundness, 0.0).clamp(0.0, 1.0) * half_w.min(half_h);
        let (sin, cos) = finite(self.rotation, 0.0).to_radians().sin_cos();
        let mut points = [[0.0f32; 2]; 8];
        for (index, point) in self.custom_points.iter().enumerate() {
            let dx = (finite(point[0], 0.5) - 0.5 - finite(self.x, 0.0)) as f32 * (width / short);
            let dy = (finite(point[1], 0.5) - 0.5 - finite(self.y, 0.0)) as f32 * (height / short);
            points[index] = [dx * cos + dy * sin, -dx * sin + dy * cos];
        }
        MaskUniform {
            a: [
                self.shape.code(),
                if self.invert { 1.0 } else { 0.0 },
                band,
                if self.shape == MaskShape::Custom { f32::from(self.custom_point_count.min(8)) } else { radius },
            ],
            b: [finite(self.x, 0.0), finite(self.y, 0.0), half_w, half_h],
            c: [cos, sin, width / short, height / short],
            d: [points[0][0], points[0][1], points[1][0], points[1][1]],
            e: [points[2][0], points[2][1], points[3][0], points[3][1]],
            f: [points[4][0], points[4][1], points[5][0], points[5][1]],
            g: [points[6][0], points[6][1], points[7][0], points[7][1]],
        }
    }

    /// How much of the picture is kept at `u`, `v` - `0..1` across it as
    /// it is seen - over a picture `width` by `height`, with `pixel` as in
    /// [`ShapeMask::uniform`].
    pub fn coverage(&self, u: f32, v: f32, width: f32, height: f32, pixel: f32) -> f32 {
        coverage(&self.uniform(width, height, pixel), u, v)
    }
}

/// How much of the picture a packed mask keeps at `u`, `v`: the shader's
/// `mask_coverage`, line for line.
pub fn coverage(mask: &MaskUniform, u: f32, v: f32) -> f32 {
    let [code, invert, band, radius] = mask.a;
    if code < 0.5 {
        return 1.0;
    }
    let [x, y, half_w, half_h] = mask.b;
    let [cos, sin, aspect_w, aspect_h] = mask.c;
    // The pixel's offset from the mask's centre in short-side units, then
    // turned back into the mask's own frame (clockwise in y-down).
    let dx = (u - 0.5 - x) * aspect_w;
    let dy = (v - 0.5 - y) * aspect_h;
    let p = [dx * cos + dy * sin, -dx * sin + dy * cos];
    let points = [
        [mask.d[0], mask.d[1]], [mask.d[2], mask.d[3]],
        [mask.e[0], mask.e[1]], [mask.e[2], mask.e[3]],
        [mask.f[0], mask.f[1]], [mask.f[2], mask.f[3]],
        [mask.g[0], mask.g[1]], [mask.g[2], mask.g[3]],
    ];
    let d = distance_with_points(code, p, [half_w, half_h], radius, &points);
    let kept = 1.0 - smoothstep(-band * 0.5, band * 0.5, d);
    if invert > 0.5 { 1.0 - kept } else { kept }
}

/// The signed distance from the shape's edge at `p`, in the mask's own
/// frame and short-side units: negative inside.
pub fn distance(code: f32, p: [f32; 2], half: [f32; 2], radius: f32) -> f32 {
    distance_with_points(code, p, half, radius, &[[0.0; 2]; 8])
}

fn distance_with_points(code: f32, p: [f32; 2], half: [f32; 2], radius: f32, points: &[[f32; 2]; 8]) -> f32 {
    let [hx, hy] = half;
    match code.round() as i32 {
        1 => p[1],
        2 => p[1].abs() - hy,
        3 => ellipse(p, half),
        4 => rounded_box(p, half, radius),
        5 => {
            // The heart's own frame: y up, its point at the origin, its
            // lobes' tops at 1.1036 and its sides at ±0.6036. Fitted to the
            // box and measured back in the box's units.
            const SIDE: f32 = 0.603_553_4;
            const MID: f32 = 0.551_776_7;
            let q = [p[0] / hx * SIDE, MID - p[1] / hy * MID];
            heart(q) * (hx / SIDE).min(hy / MID)
        }
        6 => star(p[0] / hx, -p[1] / hy) * hx.min(hy),
        7 => polygon_distance(p, points, radius.round().clamp(0.0, 8.0) as usize),
        _ => -1.0,
    }
}

fn polygon_distance(p: [f32; 2], points: &[[f32; 2]; 8], count: usize) -> f32 {
    if count < 3 { return 1.0; }
    let mut inside = false;
    let mut nearest = f32::INFINITY;
    for i in 0..count {
        let a = points[i];
        let b = points[(i + 1) % count];
        let ab = [b[0] - a[0], b[1] - a[1]];
        let ap = [p[0] - a[0], p[1] - a[1]];
        let length2 = ab[0] * ab[0] + ab[1] * ab[1];
        let t = if length2 > 1e-10 { ((ap[0] * ab[0] + ap[1] * ab[1]) / length2).clamp(0.0, 1.0) } else { 0.0 };
        let dx = ap[0] - ab[0] * t;
        let dy = ap[1] - ab[1] * t;
        nearest = nearest.min((dx * dx + dy * dy).sqrt());
        if (a[1] > p[1]) != (b[1] > p[1]) {
            let cross = (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0];
            if p[0] < cross { inside = !inside; }
        }
    }
    if inside { -nearest } else { nearest }
}

/// WGSL's `smoothstep`, for `edge0 < edge1`.
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// WGSL's `sign`: zero at zero, which Rust's `signum` is not.
fn sign(value: f32) -> f32 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        0.0
    }
}

fn length(p: [f32; 2]) -> f32 {
    (p[0] * p[0] + p[1] * p[1]).sqrt()
}

fn dot2(p: [f32; 2]) -> f32 {
    p[0] * p[0] + p[1] * p[1]
}

/// An ellipse with these half axes. Exact on a circle; on an ellipse a
/// close approximation (Quilez's), which is what a feather band needs -
/// the edge itself, where the distance is zero, is exact either way.
fn ellipse(p: [f32; 2], r: [f32; 2]) -> f32 {
    let k0 = length([p[0] / r[0], p[1] / r[1]]);
    let k1 = length([p[0] / (r[0] * r[0]), p[1] / (r[1] * r[1])]);
    if k1 <= 1e-9 {
        return -r[0].min(r[1]);
    }
    k0 * (k0 - 1.0) / k1
}

/// A box with these half sides and corners rounded by `radius`.
fn rounded_box(p: [f32; 2], half: [f32; 2], radius: f32) -> f32 {
    let q = [p[0].abs() - half[0] + radius, p[1].abs() - half[1] + radius];
    length([q[0].max(0.0), q[1].max(0.0)]) + q[0].max(q[1]).min(0.0) - radius
}

/// Quilez's heart: y up, the point at the origin.
fn heart(p: [f32; 2]) -> f32 {
    let x = p[0].abs();
    let y = p[1];
    if y + x > 1.0 {
        return length([x - 0.25, y - 0.75]) - std::f32::consts::SQRT_2 / 4.0;
    }
    let m = (x + y).max(0.0) * 0.5;
    dot2([x, y - 1.0]).min(dot2([x - m, y - m])).sqrt() * sign(x - y)
}

/// Quilez's five-pointed star, outer radius one, y up, a point straight up.
fn star(x: f32, y: f32) -> f32 {
    const K1: [f32; 2] = [0.809_017, -0.587_785_24];
    const K2: [f32; 2] = [-0.809_017, -0.587_785_24];
    const INNER: f32 = 0.45;
    let mut p = [x.abs(), y];
    let d1 = (K1[0] * p[0] + K1[1] * p[1]).max(0.0) * 2.0;
    p = [p[0] - d1 * K1[0], p[1] - d1 * K1[1]];
    let d2 = (K2[0] * p[0] + K2[1] * p[1]).max(0.0) * 2.0;
    p = [p[0] - d2 * K2[0], p[1] - d2 * K2[1]];
    p[0] = p[0].abs();
    p[1] -= 1.0;
    let ba = [INNER * -K1[1] - 0.0, INNER * K1[0] - 1.0];
    let h = ((p[0] * ba[0] + p[1] * ba[1]) / dot2(ba)).clamp(0.0, 1.0);
    length([p[0] - ba[0] * h, p[1] - ba[1] * h]) * sign(p[1] * ba[0] - p[0] * ba[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mask(shape: MaskShape) -> ShapeMask {
        ShapeMask {
            shape,
            ..ShapeMask::default()
        }
    }

    /// On a square picture of 100 pixels a side, one pixel is a hundredth.
    const PIXEL: f32 = 0.01;

    fn kept(mask: &ShapeMask, u: f32, v: f32) -> f32 {
        mask.coverage(u, v, 100.0, 100.0, PIXEL)
    }

    #[test]
    fn every_shape_keeps_its_middle_and_drops_its_far_corner() {
        for shape in MaskShape::ALL {
            let mask = ShapeMask {
                shape,
                width: 0.5,
                height: 0.5,
                ..ShapeMask::default()
            };
            // The line keeps the half above it, so its "middle" is a hair
            // above the centre.
            let (u, v) = if shape == MaskShape::Linear {
                (0.5, 0.3)
            } else {
                (0.5, 0.5)
            };
            assert!(kept(&mask, u, v) > 0.999, "{shape:?} keeps its middle");
            assert!(
                kept(&mask, 0.98, 0.98) < 0.001,
                "{shape:?} drops the corner"
            );
            let inverted = ShapeMask {
                invert: true,
                ..mask
            };
            assert!(kept(&inverted, u, v) < 0.001, "{shape:?} inverted");
            assert!(kept(&inverted, 0.98, 0.98) > 0.999, "{shape:?} inverted");
        }
    }

    #[test]
    fn a_circle_is_exact_and_round_on_a_wide_picture() {
        let circle = mask(MaskShape::Circle);
        // Radius 0.33 of the short side. On the edge, half kept.
        let d = distance(3.0, [0.33, 0.0], [0.33, 0.33], 0.0);
        assert!(d.abs() < 1e-6);
        let d = distance(3.0, [0.0, 0.5], [0.33, 0.33], 0.0);
        assert!((d - 0.17).abs() < 1e-6, "{d}");
        // On a 200 by 100 picture the short side is the height, so the
        // edge is 33 pixels from the centre both ways: u is 0.5 + 33/200.
        let wide = |u: f32, v: f32| circle.coverage(u, v, 200.0, 100.0, PIXEL);
        assert!((wide(0.5 + 0.33 / 2.0, 0.5) - 0.5).abs() < 1e-3);
        assert!((wide(0.5, 0.5 + 0.33) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn the_feather_is_a_smoothstep_centred_on_the_edge() {
        // Feather 0.4 is a band 0.2 of the short side wide, 0.1 each way.
        let soft = ShapeMask {
            feather: 0.4,
            ..mask(MaskShape::Circle)
        };
        let at = |r: f32| kept(&soft, 0.5 + r, 0.5);
        assert!((at(0.33) - 0.5).abs() < 1e-4, "half on the edge");
        assert!(at(0.33 - 0.1) > 0.9999, "all of it a band inside");
        assert!(at(0.33 + 0.1) < 1e-4, "none of it a band outside");
        // Halfway into the band, smoothstep's quarter: 0.15625 kept out.
        assert!((at(0.33 - 0.05) - 0.843_75).abs() < 1e-3);
        assert!((at(0.33 + 0.05) - 0.156_25).abs() < 1e-3);
        // A hard edge is still a pixel wide, not a step.
        let hard = mask(MaskShape::Circle);
        assert!((kept(&hard, 0.5 + 0.33, 0.5) - 0.5).abs() < 1e-3);
        assert!(kept(&hard, 0.5 + 0.33 + PIXEL, 0.5) < 1e-3);
    }

    #[test]
    fn a_rectangle_rounds_its_corners() {
        let half = [0.3, 0.2];
        // Square corners: the corner point is on the edge.
        assert!(distance(4.0, [0.3, 0.2], half, 0.0).abs() < 1e-6);
        // Fully round: the corner is cut, by r(√2 - 1) along the diagonal.
        let r = 0.2;
        let d = distance(4.0, [0.3, 0.2], half, r);
        assert!(
            (d - r * (std::f32::consts::SQRT_2 - 1.0)).abs() < 1e-5,
            "{d}"
        );
        // The middle of a side is untouched by the rounding.
        assert!(distance(4.0, [0.3, 0.0], half, r).abs() < 1e-6);
        assert!((distance(4.0, [0.0, 0.0], half, r) + 0.2).abs() < 1e-6);
    }

    #[test]
    fn the_line_and_the_band_turn_with_the_mask() {
        let line = mask(MaskShape::Linear);
        assert!(kept(&line, 0.5, 0.4) > 0.999);
        assert!(kept(&line, 0.5, 0.6) < 0.001);
        // Turned a quarter clockwise, "above" is the right: the line's
        // up points along +x once the frame is turned.
        let turned = ShapeMask {
            rotation: 90.0,
            ..line
        };
        assert!(kept(&turned, 0.6, 0.5) > 0.999);
        assert!(kept(&turned, 0.4, 0.5) < 0.001);
        let band = ShapeMask {
            height: 0.2,
            ..mask(MaskShape::Mirror)
        };
        assert!(kept(&band, 0.1, 0.5) > 0.999, "a band runs edge to edge");
        assert!(kept(&band, 0.5, 0.65) < 0.001);
        assert!((kept(&band, 0.5, 0.6) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn the_heart_and_the_star_fill_their_box() {
        let half = [0.3, 0.3];
        // The heart's point is on the box's bottom edge, its lobes at the
        // top, its sides at the box's sides.
        assert!(distance(5.0, [0.0, 0.3], half, 0.0).abs() < 1e-4);
        assert!(distance(5.0, [0.0, 0.0], half, 0.0) < 0.0);
        assert!(distance(5.0, [0.0, -0.31], half, 0.0) > 0.0);
        // The notch between the lobes is outside.
        assert!(distance(5.0, [0.0, -0.28], half, 0.0) > 0.0);
        // Its widest, where the lobes' circles are, is the box's side.
        assert!(distance(5.0, [0.3, -0.1077], half, 0.0).abs() < 1e-3);
        // The star's top point is on the box's top edge; between two
        // points it is outside, and its middle is in.
        assert!(distance(6.0, [0.0, -0.3], half, 0.0).abs() < 1e-4);
        assert!(distance(6.0, [0.0, -0.2], half, 0.0) < 0.0);
        assert!(distance(6.0, [0.0, 0.0], half, 0.0) < 0.0);
        let between = [
            0.3 * 36f32.to_radians().sin(),
            -0.3 * 36f32.to_radians().cos(),
        ];
        assert!(distance(6.0, between, half, 0.0) > 0.0);
    }

    #[test]
    fn a_moved_mask_is_measured_from_its_own_centre() {
        let moved = ShapeMask {
            x: 0.25,
            y: -0.25,
            width: 0.2,
            height: 0.2,
            ..mask(MaskShape::Rectangle)
        };
        assert!(kept(&moved, 0.75, 0.25) > 0.999);
        assert!(kept(&moved, 0.5, 0.5) < 0.001);
    }

    #[test]
    fn names_round_trip_and_nonsense_is_survivable() {
        for shape in MaskShape::ALL {
            assert_eq!(MaskShape::parse(shape.name()), Some(shape));
        }
        assert_eq!(MaskShape::parse("blob"), None);
        let broken = ShapeMask {
            width: f64::NAN,
            feather: f64::INFINITY,
            ..ShapeMask::default()
        };
        let value = kept(&broken, 0.5, 0.5);
        assert!(value.is_finite());
        assert_eq!(coverage(&MaskUniform::NONE, 0.1, 0.9), 1.0);
    }
}
