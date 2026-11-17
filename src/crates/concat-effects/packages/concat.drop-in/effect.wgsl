struct Params { duration: f32 }

// The clip drops in from above like a falling card, speeding up as it lands in the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    // Ease-in, like gravity: slow off the mark, fast on landing.
    let p = t * t;
    let shift = (1.0 - p) * 1.0;
    let scale = 1.0 + 0.1 * (1.0 - p);
    let coord = (uv - vec2<f32>(0.5)) / scale + vec2<f32>(0.5) - vec2<f32>(0.0, shift);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    return sample(coord);
}
