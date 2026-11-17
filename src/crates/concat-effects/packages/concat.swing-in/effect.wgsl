struct Params { duration: f32 }

// The clip swings in like a pendulum, rocking less and less as it lands, over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = 1.0 - pow(1.0 - t, 3.0);
    // Pendulum: rocks back and forth, amplitude dying away as p -> 1.
    let angle = (1.0 - p) * 0.5 * cos(t * 9.42477796);
    let scale = 0.9 + 0.1 * p;
    // The turn is taken in pixels, so a frame wider than it is tall
    // turns rigidly rather than shearing.
    let c = (uv - vec2<f32>(0.5)) * frame.size;
    let s = sin(angle);
    let co = cos(angle);
    let r = vec2<f32>(c.x * co - c.y * s, c.x * s + c.y * co) / scale;
    let coord = r / frame.size + vec2<f32>(0.5);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let out = sample(coord);
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
