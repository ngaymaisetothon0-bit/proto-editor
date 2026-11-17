struct Params { duration: f32 }

// The clip bounces out, swelling up a little first, then shrinking away over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let q = 1.0 - clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    // Swell by 15% halfway out, then shrink to 60% as it leaves.
    let scale = 1.0 + 0.15 * sin(q * 3.14159265) - 0.4 * q;
    let coord = (uv - vec2<f32>(0.5)) / scale + vec2<f32>(0.5);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let out = sample(coord);
    let fade = 1.0 - smoothstep(0.0, 1.0, q);
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
