struct Params { duration: f32 }

// The clip settles out of a spreading ripple, like a stone dropped in water, over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = 1.0 - pow(1.0 - t, 3.0);
    let d = distance((uv - vec2<f32>(0.5)) * frame.size, vec2<f32>(0.0));
    let wave = sin(d * 0.05 - t * 12.5663706) * (1.0 - p) * 0.02;
    let centre_d = distance(uv, vec2<f32>(0.5, 0.5));
    let dir = (uv - vec2<f32>(0.5, 0.5)) / max(centre_d, 0.001);
    let coord = uv + dir * wave;
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let out = sample(coord);
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
