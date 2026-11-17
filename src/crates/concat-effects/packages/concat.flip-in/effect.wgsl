struct Params { duration: f32 }

// The clip flips in like a card turning around its vertical axis, fading up as it lands in the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = 1.0 - pow(1.0 - t, 3.0);
    let angle = (1.0 - p) * 1.5707963;
    let co = cos(angle);
    let sx = 0.5 + (uv.x - 0.5) / max(co, 0.08);
    let coord = vec2<f32>(sx, uv.y);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let out = sample(coord);
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
