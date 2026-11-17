// A page turns over the cut: the outgoing shot peels away on a diagonal, its curl shadowed, the next shot underneath.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let d = (uv.x + uv.y) * 0.5;
    let line = p * 1.3 - 0.15;
    let m = smoothstep(line - 0.06, line + 0.06, d);
    let shadow = exp(-abs(d - line) * 18.0) * 0.35;
    let curl = exp(-abs(d - (line + 0.03)) * 40.0) * 0.25;
    let from_c = from_at(uv);
    let from_col = vec4<f32>(from_c.rgb * (1.0 - shadow), from_c.a);
    let base = mix(from_col, to_at(uv), m);
    return base + vec4<f32>(vec3<f32>(curl), 0.0);
}
