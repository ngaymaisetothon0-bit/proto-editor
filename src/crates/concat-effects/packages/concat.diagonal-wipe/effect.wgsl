struct Params { soft: f32 }

// A soft diagonal edge sweeps across with a shadow, the new shot following behind.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let d = (uv.x + uv.y) * 0.5;
    let edge = 0.02 + params.soft / 100.0 * 0.08;
    let m = smoothstep(d - edge, d + edge, p);
    let shadow = exp(-abs(p - d) * 30.0) * 0.2;
    let from_c = from_at(uv);
    let from_col = vec4<f32>(from_c.rgb * (1.0 - shadow), from_c.a);
    return mix(from_col, to_at(uv), m);
}
