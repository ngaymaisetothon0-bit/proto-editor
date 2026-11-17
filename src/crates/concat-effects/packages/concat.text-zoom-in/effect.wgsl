struct Params { duration: f32 }

// The whole title settles into place from larger and fainter over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = smoothstep(0.0, 1.0, t);
    // 150% of size and fully transparent before the zoom starts.
    let scale = 1.5 - 0.5 * p;
    let c = (uv - vec2<f32>(0.5)) * frame.size;
    let coord = c / scale / frame.size + vec2<f32>(0.5);
    let out = sample(coord);
    return vec4<f32>(out.rgb * p, out.a * p);
}
