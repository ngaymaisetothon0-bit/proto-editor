struct Params { duration: f32 }

// Each word glitches into place, jittering sideways as it sharpens out of a soft blur, one after another.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let w = clamp((p - reveal_order(uv) * 0.75) * 4.0, 0.0, 1.0);
    // The glitch-blur-in formula: a sideways jump of 0.1 of the frame
    // that is either on or off, and whose envelope dies out as the
    // word arrives. At w = 1 the shift is exactly 0, so the word
    // sits at its rest place.
    let shift = (1.0 - w) * 0.1 * step(0.5, fract(sin(w * 30.0) * 43758.5453));
    let coord = uv + vec2<f32>(shift, 0.0);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    // The same 3x3 box blur text-blur-in uses: at full progress the
    // radius is 0, so all nine taps read the same pixel and the
    // picture is exactly sharp.
    let r = texel() * (1.0 - w) * 8.0;
    var sum = vec4<f32>(0.0);
    for (var y: i32 = -1; y <= 1; y++) {
        for (var x: i32 = -1; x <= 1; x++) {
            sum += sample(coord + vec2<f32>(f32(x), f32(y)) * r);
        }
    }
    let c = sum / 9.0;
    return vec4<f32>(c.rgb * w, c.a * w);
}
