struct Params { length: f32, angle: f32 }

// A Gaussian of sigma `length` of the layer's pixels along a line `angle`
// degrees from level - anticlockwise, as a protractor reads it - in light.
// Two passes, so a long streak costs tens of taps rather than hundreds:
// `coarse` weighs twenty-four samples spread over six sigmas, and `effect`
// draws straight lines between them - a triangle across two of their
// steps, a tap every half pixel - so the streak falls away smoothly rather
// than in stairs.

const COARSE: i32 = 24;

/// The most taps either side of the centre the effect pass ever reads:
/// `length` caps at 60, so `reach` caps at 30 and `taps` (ceil of it,
/// clamped to 64) never passes 30 — the pass reads 61 taps, -30..30.
/// The loop bound below is a literal for FXC: a runtime-bound loop
/// fails its unroll with X3511 and kills the GPU device (halation,
/// 2026-11-17). 61 iterations is exact, never truncated.
const MAX_TAPS: i32 = 30;

/// Half a pixel along the line, in uv.
fn along() -> vec2<f32> {
    let a = radians(params.angle);
    return vec2<f32>(cos(a), -sin(a)) * 0.5 / frame.size;
}

/// A step of the coarse pass, in half pixels.
fn reach() -> f32 {
    return 12.0 * max(params.length, 0.5) / f32(COARSE);
}

fn coarse(uv: vec2<f32>) -> vec4<f32> {
    let step = along() * reach();
    let sigma = f32(COARSE) / 6.0;
    var sum = vec4<f32>(0.0);
    var total = 0.0;
    for (var i = 0; i < COARSE; i++) {
        let d = f32(i) - f32(COARSE - 1) * 0.5;
        let w = exp(-d * d / (2.0 * sigma * sigma));
        sum += sample_premultiplied(uv + step * d) * w;
        total += w;
    }
    return sum / total;
}

/// `coarse` at `uv`: read from its picture within the layer, and worked
/// out afresh past its outermost pixel centres, where the picture holds its
/// edge but the Gaussian reads on - so a streak by the edge of the frame is
/// the one it would be anywhere else.
fn coarse_near(uv: vec2<f32>) -> vec4<f32> {
    let p = uv * frame.size;
    if (any(p < vec2<f32>(0.5)) || any(p > frame.size - vec2<f32>(0.5))) {
        return coarse(uv);
    }
    return coarse_at(uv);
}

fn effect(uv: vec2<f32>) -> vec4<f32> {
    let reach = reach();
    let taps = min(i32(ceil(reach)), 64);
    var sum = vec4<f32>(0.0);
    var total = 0.0;
    // 61 = 2 * 30 + 1: the loop counts `k` from the literal bound and
    // derives `i`, skipping taps past `taps` — the same taps the old
    // loop read, no more and no fewer.
    for (var k = 0; k < 61; k++) {
        let i = k - MAX_TAPS;
        if (i < -taps || i > taps) {
            continue;
        }
        let w = max(1.0 - abs(f32(i)) / reach, 0.0);
        sum += coarse_near(uv + along() * f32(i)) * w;
        total += w;
    }
    let blurred = sum / total;
    return vec4<f32>(blurred.rgb / max(blurred.a, 1e-6), blurred.a);
}
