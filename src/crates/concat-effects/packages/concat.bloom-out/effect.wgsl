struct Params { duration: f32 }

// The clip blooms out, its brightness dimming to dark as it fades over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let q = 1.0 - clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let out = sample(uv);
    let bright = 1.0 - 0.8 * q;
    let col = out.rgb * bright;
    let fade = 1.0 - smoothstep(0.0, 1.0, q);
    return vec4<f32>(col * fade, out.a * fade);
}
