struct Params { duration: f32 }

// Seconds left of the clip, or a very long time where the clip has no
// known end - a treatment, a probe - so the look holds its rest there.
fn remain() -> f32 {
    return select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
}

// A one-shot exit, timed to the clip's end: past the duration the
// picture sits at full strength, untouched, and only the clip's last
// moments fade.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(remain() / max(params.duration, 0.001), 0.0, 1.0);
    let p = smoothstep(0.0, 1.0, t);
    let c = sample(uv);
    return vec4<f32>(c.rgb * p, c.a * p);
}
