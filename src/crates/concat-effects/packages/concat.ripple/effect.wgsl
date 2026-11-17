// A ring spreads out from the centre, its wavy edge washing the incoming
// shot in.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let d = distance(uv, vec2<f32>(0.5, 0.5));
    let edge = progress * 0.75 + 0.04 * sin(d * 40.0 - progress * 10.0);
    let m = 1.0 - smoothstep(edge - 0.02, edge + 0.02, d);
    return mix(from_at(uv), to_at(uv), m);
}
