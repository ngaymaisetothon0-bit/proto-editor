struct Params { roughness: f32 }

// The outgoing shot tears away on a jagged edge, shadowed like torn paper.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let jag = (hash(vec2<f32>(floor(uv.y * 24.0), 0.0), 4.0) - 0.5) * params.roughness / 100.0 * 0.3;
    let line = p * 1.2 - 0.1 + jag;
    let m = smoothstep(line - 0.02, line + 0.02, uv.x);
    let shadow = exp(-abs(uv.x - line) * 25.0) * 0.3;
    let from_c = from_at(uv);
    let from_col = vec4<f32>(from_c.rgb * (1.0 - shadow), from_c.a);
    let base = mix(from_col, to_at(uv), m);
    return base;
}
