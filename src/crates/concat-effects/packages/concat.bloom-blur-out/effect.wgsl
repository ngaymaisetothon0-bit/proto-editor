struct Params { duration: f32 }

// The clip blooms out through a blur, flaring bright as it softens over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let q = 1.0 - clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let r = texel() * q * 8.0;
    let c00 = sample(uv + vec2<f32>(-r.x, -r.y));
    let c01 = sample(uv + vec2<f32>(0.0, -r.y));
    let c02 = sample(uv + vec2<f32>(r.x, -r.y));
    let c10 = sample(uv + vec2<f32>(-r.x, 0.0));
    let c11 = sample(uv);
    let c12 = sample(uv + vec2<f32>(r.x, 0.0));
    let c20 = sample(uv + vec2<f32>(-r.x, r.y));
    let c21 = sample(uv + vec2<f32>(0.0, r.y));
    let c22 = sample(uv + vec2<f32>(r.x, r.y));
    let out = (c00 + c01 + c02 + c10 + c11 + c12 + c20 + c21 + c22) / 9.0;
    // Exactly 1.0 at q = 0, so a probe reads the input unchanged.
    let brightness = 1.0 + 1.0 * q;
    let col = out.rgb * brightness;
    let fade = 1.0 - smoothstep(0.0, 1.0, q);
    return vec4<f32>(col * fade, out.a * fade);
}
