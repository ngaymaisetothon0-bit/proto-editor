struct Params { duration: f32 }

// The clip opens in a circle through a blur, sharpening as the circle covers the frame over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = 1.0 - pow(1.0 - t, 3.0);
    let radius = p * 0.75;
    let d = distance(uv, vec2<f32>(0.5));
    if (d > radius) {
        return vec4<f32>(0.0);
    }
    let r = texel() * (1.0 - p) * 8.0;
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
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
