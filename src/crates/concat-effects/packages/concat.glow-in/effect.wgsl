struct Params { duration: f32 }

// The clip glows in, its brightness rising as it grows to full size over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = 1.0 - pow(1.0 - t, 3.0);
    let scale = 0.95 + 0.05 * p;
    let coord = (uv - vec2<f32>(0.5)) / scale + vec2<f32>(0.5);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let out = sample(coord);
    let bright = 0.5 + 0.5 * p;
    let col = out.rgb * bright;
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(col * fade, out.a * fade);
}
