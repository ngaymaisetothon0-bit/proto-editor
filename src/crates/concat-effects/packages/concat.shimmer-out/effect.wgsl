struct Params { duration: f32 }

// The clip shimmers out, its brightness dipping once as it fades over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let q = 1.0 - clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let out = sample(uv);
    // Exactly 1.0 at q = 0, so a probe reads the input unchanged.
    let shimmer = 1.0 - 0.25 * sin(q * 3.14159265);
    let col = out.rgb * shimmer;
    let fade = 1.0 - smoothstep(0.0, 1.0, q);
    return vec4<f32>(col * fade, out.a * fade);
}
