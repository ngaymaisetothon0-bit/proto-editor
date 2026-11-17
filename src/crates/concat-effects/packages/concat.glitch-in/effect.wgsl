struct Params { duration: f32 }

// The clip glitches in, its picture sliced into bands that jump sideways, then settles over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = 1.0 - pow(1.0 - t, 3.0);
    let band = floor(uv.y * 12.0);
    let shift = (1.0 - p) * 0.02 * sin(band * 1.7 + t * 6.2831853);
    let coord = uv + vec2<f32>(shift, 0.0);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let out = sample(coord);
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
