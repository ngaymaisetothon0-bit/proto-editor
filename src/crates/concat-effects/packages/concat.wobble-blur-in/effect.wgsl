struct Params { duration: f32 }

// The clip wobbles in through a blur, rocking as it sharpens over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = 1.0 - pow(1.0 - t, 3.0);
    let angle = (1.0 - p) * 0.3 * sin(t * 18.8495559);
    let scale = 0.8 + 0.2 * p;
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
    let b = texel() * (1.0 - p) * 8.0;
    let c00 = sample(coord + vec2<f32>(-b.x, -b.y));
    let c01 = sample(coord + vec2<f32>(0.0, -b.y));
    let c02 = sample(coord + vec2<f32>(b.x, -b.y));
    let c10 = sample(coord + vec2<f32>(-b.x, 0.0));
    let c11 = sample(coord);
    let c12 = sample(coord + vec2<f32>(b.x, 0.0));
    let c20 = sample(coord + vec2<f32>(-b.x, b.y));
    let c21 = sample(coord + vec2<f32>(0.0, b.y));
    let c22 = sample(coord + vec2<f32>(b.x, b.y));
    let out = (c00 + c01 + c02 + c10 + c11 + c12 + c20 + c21 + c22) / 9.0;
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
