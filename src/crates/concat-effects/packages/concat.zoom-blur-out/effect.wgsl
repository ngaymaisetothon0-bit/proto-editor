struct Params { duration: f32 }

// The clip shrinks and softens into a blur as it fades out over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let q = 1.0 - clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let scale = 1.0 - 0.4 * q;
    let centre = (uv - vec2<f32>(0.5)) / scale + vec2<f32>(0.5);
    // 3x3 blur taps, radius growing with q (same pattern as blur-out).
    let r = texel() * q * 8.0;
    var sum = vec4<f32>(0.0);
    for (var y: i32 = -1; y <= 1; y++) {
        for (var x: i32 = -1; x <= 1; x++) {
            sum += sample(centre + vec2<f32>(f32(x), f32(y)) * r);
        }
    }
    let c = sum / 9.0;
    let fade = 1.0 - smoothstep(0.0, 1.0, q);
    return vec4<f32>(c.rgb * fade, c.a * fade);
}
