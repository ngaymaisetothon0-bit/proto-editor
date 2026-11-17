struct Params { duration: f32 }

// The clip flashes in from white, the picture fading up out of the flash over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = 1.0 - pow(1.0 - t, 3.0);
    let out = sample(uv);
    let flash = (1.0 - p) * 0.8;
    let col = mix(out.rgb, vec3<f32>(1.0), flash);
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(col * fade, out.a * fade);
}
