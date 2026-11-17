struct Params { duration: f32 }

// The clip pulses in, growing with a heartbeat that settles as it lands in the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = 1.0 - pow(1.0 - t, 3.0);
    let scale = 0.7 + 0.3 * p + 0.1 * sin(t * 18.8495559) * (1.0 - p);
    let coord = (uv - vec2<f32>(0.5)) / scale + vec2<f32>(0.5);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let out = sample(coord);
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
