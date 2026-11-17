struct Params { duration: f32 }

// The clip ripples out, a spreading wave growing over the picture as it fades over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let q = 1.0 - clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let d = distance((uv - vec2<f32>(0.5)) * frame.size, vec2<f32>(0.0));
    let wave = sin(d * 0.05 - q * 12.5663706) * q * 0.02;
    let centre_d = distance(uv, vec2<f32>(0.5, 0.5));
    let dir = (uv - vec2<f32>(0.5, 0.5)) / max(centre_d, 0.001);
    let coord = uv + dir * wave;
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let out = sample(coord);
    let fade = 1.0 - smoothstep(0.0, 1.0, q);
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
