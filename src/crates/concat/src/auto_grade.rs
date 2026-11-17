// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! One-tap colour correction: measure the frame on screen and set the six
//! Adjust knobs that fix it.
//!
//! The picture is measured after the Adjust knobs already on it, so the
//! correction is a residual: what is left wrong on screen, not what the
//! source was. Pressing Auto twice changes nothing the second time, and a
//! hand grade is only ever nudged, never replaced.
//!
//! The six knobs, in the order they are worked out:
//!
//! - Exposure: the frame's mean light to middle grey (0.18 of white), in
//!   stops, on top of the exposure already set.
//! - Temperature, tint and the gain wheel: the frame's mean colour back
//!   to grey, by the same Bradford adaptation the shader applies and by
//!   the gain wheel's per-channel gains, searched together over the
//!   knobs' own ranges. The search is over the source mean - the measured
//!   mean through the inverse of the balance already on it, and through
//!   the inverse of the gain wheel already on it - so the knobs it finds
//!   are absolute, not stacked on the old ones. A mild cast is left to
//!   the wheel alone and the knob stays at daylight; a strong one splits
//!   between the two, each taking the share it can reach.
//! - Contrast: the frame's tonal spread (5th to 95th percentile of light,
//!   in stops) to 3.5 stops, by scaling the contrast already set.
//! - Saturation: a muted frame lifted towards a mean saturation of 0.30,
//!   never past +40, and never desaturating a frame that is vivid already.
//!
//! Every correction has a dead band (a change smaller than the eye would
//! notice is no change) and a cap (Auto never swings a knob to its end).
//! A black frame, or one too dark to read, grades to nothing: the knobs
//! stay where they were.
//!
//! One limit is honest to name: the gain wheel's reach is bounded too,
//! just wider than the knob's. A wheel can push one channel at most twice
//! as far as another from their luma, which in light is a ratio of about
//! 5.3 to 1 between the most- and least-boosted channel - enough for
//! tungsten, whose red runs about five times its blue, but a cast past
//! that only partly corrects, and the knob takes what share it can. What
//! is not guarded here is skin: a mean-based guard cannot tell a face
//! from a warm cast - a tungsten grey and skin sit at the same hue, only
//! their saturation differs - so a guard on the mean would halve the very
//! correction this pass exists for. Real protection needs the hues across
//! the frame, not its mean, and waits for that measure.

/// What the frame on screen measures as: the means in light, the tonal
/// spread in light, and how vivid it is in the display encoding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameStats {
    /// Mean red, green and blue, in light, 0..=1.
    pub mean: [f32; 3],
    /// Mean luminance, in light.
    pub mean_luma: f32,
    /// 5th, 50th and 95th percentile of luminance, in light.
    pub p5: f32,
    pub p50: f32,
    pub p95: f32,
    /// Mean saturation in the display encoding, 0 grey to 1 full colour.
    pub mean_saturation: f32,
    /// Pixels measured.
    pub samples: usize,
}

/// The display encoding's gamma: the monitor's frame is eight-bit
/// display-encoded, and light is the encoding raised to this, as the
/// shader's `from_display` has it.
const DISPLAY_GAMMA: f32 = 2.4;

/// Middle grey: 18 % of white, the level a contrast turns about and the
/// level an exposure aims for, as the shader's `MID_GREY` has it.
const MID_GREY: f32 = 0.18;

/// Luminance weights, as the shader's `luma` has them.
const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];

fn luma(rgb: [f32; 3]) -> f32 {
    rgb[0] * LUMA[0] + rgb[1] * LUMA[1] + rgb[2] * LUMA[2]
}

/// The display encoding into light, a channel at a time.
fn to_light(encoded: f32) -> f32 {
    encoded.powf(DISPLAY_GAMMA)
}

/// Measure an eight-bit RGBA frame, row-major, `width` by `height`.
/// At most about twenty thousand pixels are read, evenly across the
/// frame, so a 4K frame costs the same as a small one. `None` when there
/// is nothing to measure.
pub fn analyze(pixels: &[u8], width: u32, height: u32) -> Option<FrameStats> {
    let width = width as usize;
    let height = height as usize;
    if width == 0 || height == 0 || pixels.len() < width * height * 4 {
        return None;
    }
    let total = width * height;
    // A stride that keeps the sample near twenty thousand, whatever the
    // frame's size: the statistics do not need every pixel.
    let stride = (total / 20_000).max(1);
    let mut sum = [0.0f32; 3];
    let mut sum_sat = 0.0f32;
    let mut lumas: Vec<f32> = Vec::with_capacity(total / stride + 1);
    let mut n = 0usize;
    for i in (0..total).step_by(stride) {
        let base = i * 4;
        let display = [
            pixels[base] as f32 / 255.0,
            pixels[base + 1] as f32 / 255.0,
            pixels[base + 2] as f32 / 255.0,
        ];
        let light = [to_light(display[0]), to_light(display[1]), to_light(display[2])];
        sum[0] += light[0];
        sum[1] += light[1];
        sum[2] += light[2];
        let mx = display[0].max(display[1]).max(display[2]);
        let mn = display[0].min(display[1]).min(display[2]);
        if mx > 0.0 {
            sum_sat += (mx - mn) / mx;
        }
        lumas.push(luma(light));
        n += 1;
    }
    if n == 0 {
        return None;
    }
    let count = n as f32;
    let mean = [sum[0] / count, sum[1] / count, sum[2] / count];
    lumas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let at = |p: f32| lumas[((p * (n - 1) as f32).round() as usize).min(n - 1)];
    Some(FrameStats {
        mean,
        mean_luma: luma(mean),
        p5: at(0.05),
        p50: at(0.5),
        p95: at(0.95),
        mean_saturation: sum_sat / count,
        samples: n,
    })
}

// ── the white balance, ported from the shader ──
//
// The shader's `white_balance(rgb, kelvin, tint)` is a von Kries scaling
// in Bradford's cone space, and it is linear: a matrix over the colour,
// divided by a scalar that keeps a grey grey. The port below is the
// shader's own arithmetic, in the same order, so a matrix built here
// agrees with the shader to float precision. Matrices are stored
// column-major, as WGSL's `mat3x3` is: `m[col][row]`.

/// CIE XYZ into cone responses, by Bradford's matrix.
const BRADFORD: [[f32; 3]; 3] = [
    [0.8951, -0.7502, 0.0389],
    [0.2664, 1.7135, -0.0685],
    [-0.1614, 0.0367, 1.0296],
];

/// Light on Rec. 709's primaries into Bradford's cone responses, and back.
const CONES: [[f32; 3]; 3] = [
    [0.4227252927, 0.0556997770, 0.0213826437],
    [0.4913453244, 0.9615340509, 0.0876418678],
    [0.0273579445, 0.0231838105, 0.9805081326],
];
const FROM_CONES: [[f32; 3]; 3] = [
    [2.5380445168, -0.1460040576, -0.0422985104],
    [-1.2932769979, 1.1166483055, -0.0716072203],
    [-0.0402368843, -0.0223290262, 1.0227526883],
];

/// Matrix times matrix, column-major on both sides.
fn mat_mul(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for col in 0..3 {
        for row in 0..3 {
            out[col][row] = a[0][row] * b[col][0] + a[1][row] * b[col][1] + a[2][row] * b[col][2];
        }
    }
    out
}

/// Matrix times vector.
fn mat_vec(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2],
        m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2],
        m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2],
    ]
}

/// A matrix with each row scaled: the shader's `CONES[i] * gains` is a
/// component-wise product, so the row is scaled by `gains[row]`.
fn scale_rows(m: [[f32; 3]; 3], g: [f32; 3]) -> [[f32; 3]; 3] {
    [
        [m[0][0] * g[0], m[0][1] * g[1], m[0][2] * g[2]],
        [m[1][0] * g[0], m[1][1] * g[1], m[1][2] * g[2]],
        [m[2][0] * g[0], m[2][1] * g[1], m[2][2] * g[2]],
    ]
}

/// Where a black body at `kelvin` sits in CIE 1960's (u, v): Kim et al.'s
/// cubic fit to the Planckian locus, 1667 K to 25000 K.
fn planckian(kelvin: f32) -> [f32; 2] {
    let t = kelvin.clamp(1667.0, 25000.0);
    let k = 1000.0 / t;
    let x = if t <= 4000.0 {
        ((-0.2661239 * k - 0.2343589) * k + 0.8776956) * k + 0.179910
    } else {
        ((-3.0258469 * k + 2.1070379) * k + 0.2226347) * k + 0.240390
    };
    let y = if t <= 2222.0 {
        ((-1.1063814 * x - 1.34811020) * x + 2.18555832) * x - 0.20219683
    } else if t <= 4000.0 {
        ((-0.9549476 * x - 1.37418593) * x + 2.09137015) * x - 0.16748867
    } else {
        ((3.0817580 * x - 5.87338670) * x + 3.75112997) * x - 0.37001483
    };
    let d = -2.0 * x + 12.0 * y + 3.0;
    [4.0 * x / d, 6.0 * y / d]
}

/// The white of light at `kelvin`, moved `tint` across the locus towards
/// magenta - 100 is 0.07 in (u, v), minus towards green - as cone
/// responses.
fn white_cones(kelvin: f32, tint: f32) -> [f32; 3] {
    let here = planckian(kelvin);
    let next = planckian(kelvin * 1.01);
    let mut along = [next[0] - here[0], next[1] - here[1]];
    let len = (along[0] * along[0] + along[1] * along[1]).sqrt().max(1e-10);
    along = [along[0] / len, along[1] / len];
    let mut across = [along[1], -along[0]];
    if across[1] > 0.0 {
        across = [-across[0], -across[1]];
    }
    let uv = [
        here[0] + across[0] * (tint * 0.0007),
        here[1] + across[1] * (tint * 0.0007),
    ];
    let d = 2.0 * uv[0] - 8.0 * uv[1] + 4.0;
    let x = 3.0 * uv[0] / d;
    let y = 2.0 * uv[1] / d;
    mat_vec(
        BRADFORD,
        [x / y, 1.0, (1.0 - x - y) / y],
    )
}

/// The white balance as a matrix: the linear map the shader applies, so
/// its inverse is the source colour back out of a measured one.
fn wb_matrix(kelvin: f32, tint: f32) -> [[f32; 3]; 3] {
    let day = white_cones(6500.0, 0.0);
    let here = white_cones(kelvin, tint);
    let gains = [
        here[0] / day[0],
        here[1] / day[1],
        here[2] / day[2],
    ];
    let adapt = mat_mul(FROM_CONES, scale_rows(CONES, gains));
    let scale = luma(mat_vec(adapt, [1.0, 1.0, 1.0]));
    let scale = scale.max(1e-6);
    [
        [adapt[0][0] / scale, adapt[0][1] / scale, adapt[0][2] / scale],
        [adapt[1][0] / scale, adapt[1][1] / scale, adapt[1][2] / scale],
        [adapt[2][0] / scale, adapt[2][1] / scale, adapt[2][2] / scale],
    ]
}

/// The white balance itself, as the shader applies it.
fn white_balance(rgb: [f32; 3], kelvin: f32, tint: f32) -> [f32; 3] {
    mat_vec(wb_matrix(kelvin, tint), rgb)
}

// ── the gain wheel, ported from the shader ──
//
// The Adjust panel's gain wheel scales the display level by `2 ** scale`
// per channel, where `scale = 0.5 * wheel_hue(puck) + master`. A display
// level is light raised to 1/2.4, so in light the same wheel is a
// per-channel gain of `2 ** (2.4 * scale)` - exact, pixel for pixel, the
// way the panel's own arithmetic has it. That is the correction the
// temperature knob cannot reach: a diagonal gain in light, where the knob
// is a Bradford matrix bound to the Planckian locus.
//
// A wheel can say any scale whose channels sit within half a stop of
// their luma: the hue it pushes towards is luma-free and reaches at most
// 1, so `scale - luma(scale)` is at most 0.5 in any channel. In light
// that is a ratio of at most `2 ** 2.4`, about 5.3 to 1, between the
// most- and least-boosted channel.

/// A wheel's puck as the colour it pushes towards, as the shader's
/// `wheel_hue` has it: the hue at its angle - red at the top, then
/// clockwise yellow, green, cyan, blue, magenta - as far from grey as the
/// puck is from the middle, with no brightness of its own. Luma-free, and
/// its largest channel is the puck's reach.
fn wheel_hue(x: f32, y: f32) -> [f32; 3] {
    let reach = (x * x + y * y).sqrt().min(1.0);
    if reach < 1e-5 {
        return [0.0; 3];
    }
    let turn = (x.atan2(y) / (2.0 * std::f32::consts::PI) + 1.0).fract();
    let hue = [1.0f32, 2.0 / 3.0, 1.0 / 3.0]
        .map(|off| (((turn + off).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0));
    let lum = luma(hue);
    let chroma = [hue[0] - lum, hue[1] - lum, hue[2] - lum];
    let biggest = chroma[0].abs().max(chroma[1].abs()).max(chroma[2].abs());
    [
        chroma[0] / biggest * reach,
        chroma[1] / biggest * reach,
        chroma[2] / biggest * reach,
    ]
}

/// A wheel's scale, as the shader's `grade_wheels` has it: half its hue
/// plus its master, a stop per channel on the display level.
fn wheel_scale(x: f32, y: f32, m: f32) -> [f32; 3] {
    let hue = wheel_hue(x, y);
    [0.5 * hue[0] + m, 0.5 * hue[1] + m, 0.5 * hue[2] + m]
}

/// The same wheel as per-channel gains in light: a display gain of
/// `2 ** scale` is a light gain of `2 ** (2.4 * scale)`.
fn wheel_light_gains(x: f32, y: f32, m: f32) -> [f32; 3] {
    wheel_scale(x, y, m).map(|s| 2.0f32.powf(2.4 * s))
}

/// A scale into its master and its deviation from it, the deviation
/// scaled back towards neutral when the wheel cannot reach it. The
/// master is clamped to the panel's own range; the deviation is what
/// corrects a cast, and clamping the master only moves the level.
fn split_scale(scale: [f32; 3]) -> (f32, [f32; 3]) {
    let m_raw = luma(scale);
    let mut dev = [scale[0] - m_raw, scale[1] - m_raw, scale[2] - m_raw];
    let worst = dev[0].abs().max(dev[1].abs()).max(dev[2].abs());
    if worst > 0.5 {
        let shrink = 0.5 / worst;
        dev = [dev[0] * shrink, dev[1] * shrink, dev[2] * shrink];
    }
    (m_raw.clamp(-1.0, 1.0), dev)
}

/// A scale back into a puck and a master. The puck's direction is found
/// by turning it in half degrees until the shader's own hue lines up
/// with the scale's, and its distance from the middle is the hue's reach.
fn wheel_from_scale(scale: [f32; 3]) -> (f32, f32, f32) {
    let (m, dev) = split_scale(scale);
    let hue = [2.0 * dev[0], 2.0 * dev[1], 2.0 * dev[2]];
    let reach = hue[0].abs().max(hue[1].abs()).max(hue[2].abs());
    if reach < 1e-6 {
        return (0.0, 0.0, m);
    }
    let target = [hue[0] / reach, hue[1] / reach, hue[2] / reach];
    let mut best_angle = 0.0f32;
    let mut best_dot = -2.0f32;
    let mut step = 0;
    while step < 720 {
        let angle = 2.0 * std::f32::consts::PI * step as f32 / 720.0;
        let got = wheel_hue(reach * angle.sin(), reach * angle.cos());
        let len =
            (got[0] * got[0] + got[1] * got[1] + got[2] * got[2]).sqrt().max(1e-9);
        let dot =
            (got[0] / len) * target[0] + (got[1] / len) * target[1] + (got[2] / len) * target[2];
        if dot > best_dot {
            best_dot = dot;
            best_angle = angle;
        }
        step += 1;
    }
    (
        reach * best_angle.sin(),
        reach * best_angle.cos(),
        m,
    )
}

/// The wheel that best corrects a light mean: the per-channel gains that
/// bring it to grey at its own luma, as a scale, and the mean that scale
/// leaves behind. A mean past the wheel's reach is only partly corrected.
fn best_wheel(mean: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let r = [
        mean[0].max(1e-4),
        mean[1].max(1e-4),
        mean[2].max(1e-4),
    ];
    let level = luma(r);
    let gains = [level / r[0], level / r[1], level / r[2]];
    let wanted = [
        gains[0].log2() / 2.4,
        gains[1].log2() / 2.4,
        gains[2].log2() / 2.4,
    ];
    let (m, dev) = split_scale(wanted);
    let scale = [m + dev[0], m + dev[1], m + dev[2]];
    let light = scale.map(|s| 2.0f32.powf(2.4 * s));
    (
        [r[0] * light[0], r[1] * light[1], r[2] * light[2]],
        scale,
    )
}

/// How far a mean sits from grey, as the search scores it: its range
/// over its luma.
fn spread(mean: [f32; 3]) -> f32 {
    (mean[0].max(mean[1]).max(mean[2]) - mean[0].min(mean[1]).min(mean[2]))
        / luma(mean).max(1e-6)
}

/// A 3x3 inverse, by the adjugate. `None` when the matrix is singular.
fn mat_inv(m: [[f32; 3]; 3]) -> Option<[[f32; 3]; 3]> {
    // Rows of the matrix, for the cofactor arithmetic.
    let a = m[0][0];
    let b = m[1][0];
    let c = m[2][0];
    let d = m[0][1];
    let e = m[1][1];
    let f = m[2][1];
    let g = m[0][2];
    let h = m[1][2];
    let i = m[2][2];
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    if !det.is_finite() || det.abs() < 1e-10 {
        return None;
    }
    let inv_det = 1.0 / det;
    // The adjugate, transposed back into column-major.
    let adj = [
        [(e * i - f * h), (c * h - b * i), (b * f - c * e)],
        [(f * g - d * i), (a * i - c * g), (c * d - a * f)],
        [(d * h - e * g), (b * g - a * h), (a * e - b * d)],
    ];
    Some([
        [adj[0][0] * inv_det, adj[1][0] * inv_det, adj[2][0] * inv_det],
        [adj[0][1] * inv_det, adj[1][1] * inv_det, adj[2][1] * inv_det],
        [adj[0][2] * inv_det, adj[1][2] * inv_det, adj[2][2] * inv_det],
    ])
}

// ── the suggestion ──

/// The six knobs Auto sets, at the values the clip holds now.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurrentAdjust {
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub contrast: f32,
    pub saturation: f32,
    /// The gain wheel's puck and master, as the panel holds them.
    pub gain_x: f32,
    pub gain_y: f32,
    pub gain_m: f32,
}

impl Default for CurrentAdjust {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            temperature: 6500.0,
            tint: 0.0,
            contrast: 0.0,
            saturation: 0.0,
            gain_x: 0.0,
            gain_y: 0.0,
            gain_m: 0.0,
        }
    }
}

/// The six knobs after Auto, and whether any of them moved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuggestedAdjust {
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub gain_x: f32,
    pub gain_y: f32,
    pub gain_m: f32,
    pub changed: bool,
}

/// The tonal spread a well-graded frame spans, in stops from its 5th to
/// its 95th percentile of light.
const TARGET_SPREAD_STOPS: f32 = 3.5;

/// The vividness a frame is lifted towards, as mean saturation in the
/// display encoding.
const TARGET_SATURATION: f32 = 0.30;

/// Work out the five knobs from the measured frame and the values the
/// clip holds now. A knob that would move less than its dead band keeps
/// its value, so the result reports no change for a frame already
/// balanced.
pub fn suggest(stats: &FrameStats, current: &CurrentAdjust) -> SuggestedAdjust {
    let mut out = SuggestedAdjust {
        exposure: current.exposure,
        temperature: current.temperature,
        tint: current.tint,
        contrast: current.contrast,
        saturation: current.saturation,
        gain_x: current.gain_x,
        gain_y: current.gain_y,
        gain_m: current.gain_m,
        changed: false,
    };

    // Exposure: the mean light to middle grey, in stops. A frame too
    // dark to read - a fade to black - is left alone.
    if stats.mean_luma > 1e-4 {
        let delta = (MID_GREY / stats.mean_luma).log2();
        if delta.abs() >= 0.05 {
            let next = (current.exposure + delta).clamp(-3.0, 3.0);
            if (next - current.exposure).abs() >= 0.05 {
                out.exposure = next;
            }
        }
    }

    // White balance: the source mean - the measured mean through the
    // inverse of the balance already on it, and through the inverse of
    // the gain wheel already on it, since the wheel scales the light the
    // same way - back to grey, by the knob and the wheel searched
    // together. A knob is scored after the wheel has done its best on
    // what the knob leaves, so a mild cast is left to the wheel and the
    // knob only moves when the wheel cannot reach it alone. The search
    // favours the neutral end of a tie, so a frame already grey keeps its
    // knobs.
    if let Some(inv) = mat_inv(wb_matrix(current.temperature, current.tint)) {
        let wheel_now = wheel_light_gains(current.gain_x, current.gain_y, current.gain_m);
        let measured = [
            stats.mean[0] / wheel_now[0],
            stats.mean[1] / wheel_now[1],
            stats.mean[2] / wheel_now[2],
        ];
        let source = mat_vec(inv, measured);
        let source_luma = luma(source);
        if source_luma > 1e-4 {
            let mut best: Option<(f32, f32, f32)> = None;
            let mut k = 2000.0f32;
            while k <= 11000.0 {
                let mut tint = -100.0f32;
                while tint <= 100.0 {
                    let corrected = white_balance(source, k, tint);
                    let (after_wheel, _) = best_wheel(corrected);
                    // The spread after the wheel, plus a nudge towards
                    // neutral: among near-equal answers the one nearest
                    // daylight wins.
                    let score = spread(after_wheel)
                        + 0.002 * ((k - 6500.0).abs() / 4500.0 + tint.abs() / 100.0);
                    if best.is_none_or(|(_, _, best_score)| score < best_score) {
                        best = Some((k, tint, score));
                    }
                    tint += 5.0;
                }
                k += 50.0;
            }
            if let Some((k, tint, _)) = best {
                if spread(source) >= 0.02 {
                    // The wheel for the knob chosen: the scale that
                    // finishes what the knob leaves, as a puck and a
                    // master on the panel. A wheel the knob already
                    // finished without, or one that would barely move,
                    // stays where it is: a hand grade is only ever
                    // nudged, never replaced.
                    let corrected = white_balance(source, k, tint);
                    let (after_wheel, scale) = best_wheel(corrected);
                    if spread(after_wheel) + 0.02 < spread(corrected) {
                        let (x, y, m) = wheel_from_scale(scale);
                        if (x - current.gain_x).abs() >= 0.01
                            || (y - current.gain_y).abs() >= 0.01
                            || (m - current.gain_m).abs() >= 0.01
                        {
                            out.gain_x = x;
                            out.gain_y = y;
                            out.gain_m = m;
                        }
                    }
                    if k != current.temperature || tint != current.tint {
                        out.temperature = k;
                        out.tint = tint;
                    }
                }
            }
        }
    }

    // Contrast: the tonal spread to 3.5 stops, by scaling the contrast
    // already set. The spread is measured in light, where the shader's
    // contrast is a power about middle grey, so the spread scales with
    // its amount.
    if stats.p5 > 1e-4 && stats.p95 > stats.p5 {
        let spread = (stats.p95 / stats.p5).log2();
        if spread > 0.5 && spread < TARGET_SPREAD_STOPS {
            let factor = (TARGET_SPREAD_STOPS / spread).clamp(1.0, 1.4);
            let amount = (1.0 + current.contrast / 100.0) * factor;
            let mut next = (amount - 1.0) * 100.0;
            next = next.clamp(current.contrast - 50.0, current.contrast + 50.0);
            next = next.clamp(-100.0, 100.0);
            if (next - current.contrast).abs() >= 3.0 {
                out.contrast = next;
            }
        }
    }

    // Saturation: a muted frame lifted towards 0.30, by the factor the
    // shader's knob applies. A vivid frame is left alone, and the lift
    // never passes +40.
    if stats.mean_saturation > 0.02 && stats.mean_saturation < TARGET_SATURATION {
        let needed = (TARGET_SATURATION / stats.mean_saturation).clamp(1.0, 1.5);
        let now = if current.saturation >= 0.0 {
            (1.0 + current.saturation / 100.0) * (1.0 + current.saturation / 100.0)
        } else {
            1.0 + 2.0 * current.saturation / 100.0
        };
        let target = now * needed;
        let next = if target >= 1.0 {
            100.0 * (target.sqrt() - 1.0)
        } else {
            100.0 * (target - 1.0) / 2.0
        };
        let next = next.clamp(0.0, 40.0);
        if next - current.saturation >= 3.0 {
            out.saturation = next;
        }
    }

    out.changed = out.exposure != current.exposure
        || out.temperature != current.temperature
        || out.tint != current.tint
        || out.contrast != current.contrast
        || out.saturation != current.saturation
        || out.gain_x != current.gain_x
        || out.gain_y != current.gain_y
        || out.gain_m != current.gain_m;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    fn close_rgb(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
        close(a[0], b[0], eps) && close(a[1], b[1], eps) && close(a[2], b[2], eps)
    }

    #[test]
    fn daylight_balance_is_the_identity() {
        let rgb = [0.3, 0.5, 0.2];
        assert!(close_rgb(white_balance(rgb, 6500.0, 0.0), rgb, 1e-4));
        let m = wb_matrix(6500.0, 0.0);
        assert!(close_rgb(mat_vec(m, rgb), rgb, 1e-4));
    }

    #[test]
    fn a_balance_undoes_itself() {
        let rgb = [0.4, 0.25, 0.6];
        for (k, tint) in [(3200.0, 0.0), (9000.0, 0.0), (5500.0, 40.0), (7500.0, -60.0)] {
            let m = wb_matrix(k, tint);
            let inv = mat_inv(m).expect("a balance inverts");
            let back = mat_vec(inv, mat_vec(m, rgb));
            assert!(close_rgb(back, rgb, 1e-3), "{k}K tint {tint}");
            // And the matrix times its inverse is the identity.
            let id = mat_mul(m, inv);
            assert!(close(id[0][0], 1.0, 1e-3));
            assert!(close(id[1][1], 1.0, 1e-3));
            assert!(close(id[2][2], 1.0, 1e-3));
            assert!(close(id[0][1], 0.0, 1e-3));
        }
    }

    /// A suggested grade applied to a light mean: the knob's matrix,
    /// then the wheel's light gains.
    fn apply_grade(mean: [f32; 3], got: &SuggestedAdjust) -> [f32; 3] {
        let balanced = white_balance(mean, got.temperature, got.tint);
        let gains = wheel_light_gains(got.gain_x, got.gain_y, got.gain_m);
        [
            balanced[0] * gains[0],
            balanced[1] * gains[1],
            balanced[2] * gains[2],
        ]
    }

    fn stats_for(mean: [f32; 3]) -> FrameStats {
        FrameStats {
            mean,
            mean_luma: 0.18,
            p5: 0.02,
            p50: 0.18,
            p95: 0.6,
            mean_saturation: 0.35,
            samples: 100,
        }
    }

    #[test]
    fn wheel_hue_is_luma_free_and_reaches() {
        for (x, y) in [(0.5, 0.0), (-0.3, 0.8), (0.0, 0.0), (1.2, 0.0), (0.0, -1.0)] {
            let hue = wheel_hue(x, y);
            assert!(close(luma(hue), 0.0, 1e-5), "hue has no luma: {hue:?}");
            let reach = (x * x + y * y).sqrt().min(1.0);
            let biggest = hue[0].abs().max(hue[1].abs()).max(hue[2].abs());
            assert!(close(biggest, reach, 1e-5), "hue reaches {reach}: {hue:?}");
        }
    }

    #[test]
    fn a_wheel_round_trips_through_its_scale() {
        // Feasible scales come back as themselves; one past the wheel's
        // reach comes back as the closest the panel can hold.
        for scale in [
            [0.3, -0.1, 0.05],
            [-0.4, 0.2, 0.1],
            [0.0, 0.0, 0.0],
            [0.9, -0.9, 0.0],
        ] {
            let (x, y, m) = wheel_from_scale(scale);
            assert!(x >= -1.0 && x <= 1.0 && y >= -1.0 && y <= 1.0, "puck in the disc");
            assert!(m >= -1.0 && m <= 1.0, "master in range");
            let back = wheel_scale(x, y, m);
            let (want_m, want_dev) = split_scale(scale);
            let want = [
                want_m + want_dev[0],
                want_m + want_dev[1],
                want_m + want_dev[2],
            ];
            assert!(
                close_rgb(back, want, 0.02),
                "scale {scale:?} round-trips to {back:?}, want {want:?}"
            );
        }
    }

    #[test]
    fn a_warm_frame_is_cooled_and_a_cool_one_warmed() {
        // A mild warm mean: too much red, too little blue. The joint
        // search leaves the knob at daylight and lets the wheel do it.
        let warm = stats_for([0.30, 0.25, 0.18]);
        let got = suggest(&warm, &CurrentAdjust::default());
        assert_eq!(got.temperature, 6500.0, "mild warm leaves the knob");
        assert!(
            (got.gain_x * got.gain_x + got.gain_y * got.gain_y).sqrt() > 0.1,
            "mild warm moves the puck: {got:?}"
        );
        assert!(spread(apply_grade(warm.mean, &got)) < 0.05, "corrected spread");

        // A mild cool mean: too much blue, too little red.
        let cool = stats_for([0.18, 0.25, 0.30]);
        let got = suggest(&cool, &CurrentAdjust::default());
        assert_eq!(got.temperature, 6500.0, "mild cool leaves the knob");
        assert!(spread(apply_grade(cool.mean, &got)) < 0.05, "corrected spread");
    }

    #[test]
    fn a_tungsten_frame_is_fully_corrected() {
        // Tungsten light: a grey three times as red as blue. The knob
        // alone halves its spread; knob and wheel together take it to
        // grey - the knob moves part-way, the wheel finishes it.
        let tungsten = stats_for([0.522, 0.282, 0.100]);
        assert!(spread(tungsten.mean) > 1.0, "the cast is strong");
        let got = suggest(&tungsten, &CurrentAdjust::default());
        assert!(got.temperature > 6500.0, "tungsten moves the knob: {}", got.temperature);
        assert!(
            (got.gain_x * got.gain_x + got.gain_y * got.gain_y).sqrt() > 0.5,
            "tungsten moves the puck far: {got:?}"
        );
        assert!(
            spread(apply_grade(tungsten.mean, &got)) < 0.05,
            "tungsten corrected: {:?}",
            apply_grade(tungsten.mean, &got)
        );
    }

    #[test]
    fn an_extreme_cast_is_partly_corrected() {
        // Past the wheel's reach even with the knob: the spread falls
        // from over 2 to under 0.45, but does not vanish.
        let extreme = stats_for([0.60, 0.20, 0.05]);
        let got = suggest(&extreme, &CurrentAdjust::default());
        let after = spread(apply_grade(extreme.mean, &got));
        assert!(after < 0.45, "extreme partly corrected: {after}");
        assert!(after > 0.05, "extreme does not vanish: {after}");
    }

    #[test]
    fn auto_is_idempotent() {
        // Press Auto on a tungsten frame, then press it again on the
        // corrected frame with the knobs it set: the second press changes
        // nothing, because the source it recovers is the same source.
        let tungsten = stats_for([0.522, 0.282, 0.100]);
        let first = suggest(&tungsten, &CurrentAdjust::default());
        assert!(first.changed, "first press moves");
        let corrected = stats_for(apply_grade(tungsten.mean, &first));
        let current = CurrentAdjust {
            exposure: first.exposure,
            temperature: first.temperature,
            tint: first.tint,
            contrast: first.contrast,
            saturation: first.saturation,
            gain_x: first.gain_x,
            gain_y: first.gain_y,
            gain_m: first.gain_m,
        };
        let second = suggest(&corrected, &current);
        assert!(!second.changed, "second press rests: {second:?}");
    }

    #[test]
    fn a_dark_frame_is_lifted_and_a_bright_one_pulled() {
        let dark = FrameStats {
            mean: [0.05, 0.05, 0.05],
            mean_luma: 0.05,
            p5: 0.01,
            p50: 0.05,
            p95: 0.12,
            mean_saturation: 0.1,
            samples: 100,
        };
        let got = suggest(&dark, &CurrentAdjust::default());
        assert!(got.exposure > 1.0, "dark lifts: {}", got.exposure);
        // Lifting to middle grey from 0.05 is log2(0.18/0.05) = 1.85 stops.
        assert!(close(got.exposure, 1.85, 0.05));

        let bright = FrameStats {
            mean: [0.6, 0.6, 0.6],
            mean_luma: 0.6,
            p5: 0.3,
            p50: 0.6,
            p95: 0.9,
            mean_saturation: 0.1,
            samples: 100,
        };
        let got = suggest(&bright, &CurrentAdjust::default());
        assert!(got.exposure < -1.0, "bright pulls: {}", got.exposure);
    }

    #[test]
    fn a_flat_frame_gains_contrast_and_a_muted_one_colour() {
        // p5 0.10 to p95 0.20 is one stop of spread; the target is 3.5,
        // so the contrast is pushed up hard, capped by the 1.4x factor.
        let flat = FrameStats {
            mean: [0.18, 0.18, 0.18],
            mean_luma: 0.18,
            p5: 0.10,
            p50: 0.18,
            p95: 0.20,
            mean_saturation: 0.35,
            samples: 100,
        };
        let got = suggest(&flat, &CurrentAdjust::default());
        assert!(got.contrast > 20.0, "flat gains contrast: {}", got.contrast);

        let muted = FrameStats {
            mean: [0.18, 0.18, 0.18],
            mean_luma: 0.18,
            p5: 0.02,
            p50: 0.18,
            p95: 0.6,
            mean_saturation: 0.08,
            samples: 100,
        };
        let got = suggest(&muted, &CurrentAdjust::default());
        assert!(got.saturation > 10.0, "muted gains colour: {}", got.saturation);
        assert!(got.saturation <= 40.0);
    }

    #[test]
    fn a_balanced_frame_is_left_alone() {
        let balanced = FrameStats {
            mean: [0.18, 0.18, 0.18],
            mean_luma: 0.18,
            p5: 0.02,
            p50: 0.18,
            p95: 0.6,
            mean_saturation: 0.35,
            samples: 100,
        };
        let got = suggest(&balanced, &CurrentAdjust::default());
        assert!(!got.changed, "balanced stays: {got:?}");
        assert_eq!(got.gain_x, 0.0, "wheel stays at rest");
        assert_eq!(got.gain_y, 0.0, "wheel stays at rest");
        assert_eq!(got.gain_m, 0.0, "wheel stays at rest");
    }

    #[test]
    fn a_black_frame_grades_to_nothing() {
        let black = FrameStats {
            mean: [0.0, 0.0, 0.0],
            mean_luma: 0.0,
            p5: 0.0,
            p50: 0.0,
            p95: 0.0,
            mean_saturation: 0.0,
            samples: 100,
        };
        let got = suggest(&black, &CurrentAdjust::default());
        assert!(!got.changed, "black stays: {got:?}");
    }

    #[test]
    fn analyze_reads_a_flat_frame() {
        // Mid-grey in the display encoding: 0.5 ^ 2.4 of light.
        let mut pixels = vec![0u8; 4 * 4 * 4];
        for px in pixels.chunks_exact_mut(4) {
            px[0] = 128;
            px[1] = 128;
            px[2] = 128;
            px[3] = 255;
        }
        let stats = analyze(&pixels, 4, 4).expect("a frame measures");
        assert_eq!(stats.samples, 16);
        assert!(close(stats.mean_luma, 0.5f32.powf(2.4), 0.01));
        assert!(close(stats.mean_saturation, 0.0, 1e-6));
        assert!(stats.p5 <= stats.p50 && stats.p50 <= stats.p95);
        assert!(analyze(&[], 0, 0).is_none());
    }
}
