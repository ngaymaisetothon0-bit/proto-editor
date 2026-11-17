struct Params { duration: f32 }

// The clip drops out through a blur, falling down as it softens over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let q = 1.0 - clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let shift = q * 1.0;
    let coord = uv + vec2<f32>(0.0, shift);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let r = texel() * q * 8.0;
    let c00 = sample(coord + vec2<f32>(-r.x, -r.y));
    let c01 = sample(coord + vec2<f32>(0.0, -r.y));
    let c02 = sample(coord + vec2<f32>(r.x, -r.y));
    let c10 = sample(coord + vec2<f32>(-r.x, 0.0));
    let c11 = sample(coord);
    let c12 = sample(coord + vec2<f32>(r.x, 0.0));
    let c20 = sample(coord + vec2<f32>(-r.x, r.y));
    let c21 = sample(coord + vec2<f32>(0.0, r.y));
    let c22 = sample(coord + vec2<f32>(r.x, r.y));
    let out = (c00 + c01 + c02 + c10 + c11 + c12 + c20 + c21 + c22) / 9.0;
    let fade = 1.0 - smoothstep(0.0, 1.0, q);
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
