struct Params { slats: f32 }

// Vertical blinds open one slat at a time, the next shot peeking through.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let n = 6.0 + params.slats / 100.0 * 12.0;
    let strip = floor(uv.x * n);
    let th = fract(strip * 0.37);
    let m = smoothstep(th, th + 0.2, p);
    let shadow = exp(-abs(p - th) * 20.0) * 0.15;
    let from_c = from_at(uv);
    let from_col = vec4<f32>(from_c.rgb * (1.0 - shadow), from_c.a);
    let base = mix(from_col, to_at(uv), m);
    return base;
}
