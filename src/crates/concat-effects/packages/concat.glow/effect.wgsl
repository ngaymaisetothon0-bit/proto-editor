struct Params { amount: f32 }

// The picture's light spread wide - a Gaussian of sixteen of the layer's
// pixels, across and then down, at a quarter of them - and screened back
// over the picture by the amount. The screen is taken in the display
// encoding the glow was drawn in, so on SDR it reads as it always did; a
// level past white is carried on past it rather than bent back. The light
// is spread premultiplied, so a transparent pixel adds none.

const SIGMA: f32 = 16.0;

/// How far the taps reach, in sigmas.
const REACH: f32 = 3.0;

/// The most taps one blur pass ever unrolls. The loop bound must be a
/// compile-time constant: FXC refuses to unroll a loop whose trip count
/// comes from a runtime value (halation, 2026-11-17: X3511, the GPU
/// device lost, every card after it failed). Both passes here ask for
/// 49 pairs — across from the constants (ceil(3*16)+1), down from the
/// same sigma once the pass sizes cancel (16 * rows / frame.size.y = 16,
/// frame.size being the layer's size in every pass) — so 56 covers them
/// with room for float error, and the capped path is never taken.
const MAX_PAIRS: i32 = 56;

fn weight(d: f32, s: f32) -> f32 {
    return exp(-d * d / (2.0 * s * s));
}

fn layer_at(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    return vec4<f32>(c.rgb * c.a, c.a);
}

/// The layer weighed along its row about `uv`, two pixels to a fetch.
fn across(uv: vec2<f32>) -> vec4<f32> {
    let width = frame.size.x;
    let centre = uv.x * width;
    let pairs = i32(ceil(REACH * SIGMA)) + 1;
    let capped = min(pairs, MAX_PAIRS);
    // Keep the original first tap while the kernel fits, so the output
    // is bit-identical. When it does not fit, re-centre the window on
    // `centre` so the truncated kernel stays symmetric instead of
    // keeping only its left-hand taps.
    var first = floor(centre - REACH * SIGMA - 0.5) + 0.5;
    if (pairs > MAX_PAIRS) {
        first = floor(centre - f32(capped - 1) - 0.5) + 0.5;
    }
    var sum = vec4<f32>(0.0);
    var total = 0.0;
    for (var i = 0; i < MAX_PAIRS; i++) {
        if (i >= capped) {
            break;
        }
        let a = first + f32(2 * i);
        let wa = weight(a - centre, SIGMA);
        let wb = weight(a + 1.0 - centre, SIGMA);
        let w = wa + wb;
        sum += layer_at(vec2<f32>((a + wb / w) / width, uv.y)) * w;
        total += w;
    }
    return sum / total;
}

/// `across` weighed down its column about `uv`, in its own pixels.
fn down(uv: vec2<f32>) -> vec4<f32> {
    let rows = 1.0 / across_texel().y;
    let s = SIGMA * rows / frame.size.y;
    let centre = uv.y * rows;
    let pairs = i32(ceil(REACH * s)) + 1;
    let capped = min(pairs, MAX_PAIRS);
    // Same constant bound as `across`: `down`'s pairs are 49 too (see
    // MAX_PAIRS), so the capped path is never taken and this pass is
    // bit-identical; the bound is constant all the same, so a future
    // param change cannot reintroduce the unroll failure here.
    var first = floor(centre - REACH * s - 0.5) + 0.5;
    if (pairs > MAX_PAIRS) {
        first = floor(centre - f32(capped - 1) - 0.5) + 0.5;
    }
    var sum = vec4<f32>(0.0);
    var total = 0.0;
    for (var i = 0; i < MAX_PAIRS; i++) {
        if (i >= capped) {
            break;
        }
        let a = first + f32(2 * i);
        let wa = weight(a - centre, s);
        let wb = weight(a + 1.0 - centre, s);
        let w = wa + wb;
        sum += across_at(vec2<f32>(uv.x, (a + wb / w) / rows)) * w;
        total += w;
    }
    return sum / total;
}

/// `over` screened onto `base`, both display-encoded: light added the way
/// two projectors add it, a level of `base` past white kept past it.
fn screen(base: vec3<f32>, over: vec3<f32>) -> vec3<f32> {
    let b = min(base, vec3<f32>(1.0));
    return vec3<f32>(1.0) - (vec3<f32>(1.0) - b) * (vec3<f32>(1.0) - over) + (base - b);
}

fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let base = to_display(c.rgb);
    let glowing = screen(base, to_display(down_at(uv).rgb));
    return vec4<f32>(from_display(mix(base, glowing, params.amount / 100.0)), c.a);
}
