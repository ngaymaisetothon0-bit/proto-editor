struct Params { duration: f32 }

// Each word shimmers into place, its brightness rippling as it sharpens out of a soft blur, one after another.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let w = clamp((p - reveal_order(uv) * 0.75) * 4.0, 0.0, 1.0);
    // No transform in this one: the blur samples around the
    // input point itself, like text-blur-in does.
    let coord = uv;
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
    // The shimmer-blur-in formula, in its decaying form: exactly 1.0
    // at w = 1, so the word settles at its true brightness.
    let shimmer = 1.0 + 0.25 * sin(w * 12.5663706) * (1.0 - w);
    return vec4<f32>(c.rgb * shimmer * w, c.a * w);
}
