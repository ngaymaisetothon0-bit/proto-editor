struct Params { duration: f32 }

// The clip shimmers in through a blur, its brightness rippling as it sharpens over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let r = texel() * (1.0 - t) * 8.0;
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
    // Exactly 1.0 at t = 1, so the clip settles at its true brightness.
    let shimmer = 1.0 + 0.25 * sin(t * 12.5663706) * (1.0 - t);
    let col = out.rgb * shimmer;
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(col * fade, out.a * fade);
}
