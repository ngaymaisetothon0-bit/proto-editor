struct Params { feather: f32 }

// The next shot burns through the bright parts first, the edge glowing as it spreads.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let from_col = from_at(uv);
    let to_col = to_at(uv);
    let th = 1.0 - smoothstep(0.0, 1.0, progress);
    let w = 0.05 + params.feather / 100.0 * 0.25;
    let l = luma(from_col.rgb);
    let m = smoothstep(th - w, th + w, l);
    let glow = exp(-abs(l - th) * 12.0) * 0.25;
    let base = mix(from_col, to_col, m);
    return base + vec4<f32>(vec3<f32>(glow), 0.0);
}
