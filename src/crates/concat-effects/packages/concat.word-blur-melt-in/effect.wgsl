struct Params { duration: f32 }

// Each word melts into place, sagging down as it sharpens out of a soft blur, one after another.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let w = clamp((p - reveal_order(uv) * 0.75) * 4.0, 0.0, 1.0);
    // The melt-blur-in formula: a vertical squash that doubles the
    // picture's height at the start and a downward sag of 0.3 of the
    // frame. At w = 1 the squash is 1 and the sag is 0, so the word
    // sits at its rest place.
    let sy = 1.0 + 1.0 * (1.0 - w);
    let coord = vec2<f32>(uv.x, 0.5 + (uv.y - 0.5) / sy) + vec2<f32>(0.0, (1.0 - w) * 0.3);
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
