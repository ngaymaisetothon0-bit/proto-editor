struct Params { duration: f32 }

// The clip spins away +120 degrees, shrinking to 60% and fading out, over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let t = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let out_p = 1.0 - smoothstep(0.0, 1.0, t);
    let angle = out_p * 2.0943951;
    let scale = 1.0 - 0.4 * out_p;
    let c = (uv - vec2<f32>(0.5)) * frame.size;
    let s = sin(angle);
    let co = cos(angle);
    let r = vec2<f32>(c.x * co - c.y * s, c.x * s + c.y * co) / scale;
    let coord = r / frame.size + vec2<f32>(0.5);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let out = sample(coord);
    let fade = 1.0 - out_p;
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
