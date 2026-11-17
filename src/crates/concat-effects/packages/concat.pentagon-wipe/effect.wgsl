struct Params { soft: f32 }

// A pentagon edge expands from the centre as the new shot takes over.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let c = uv - vec2<f32>(0.5);
    let d = max(max(dot(c, vec2<f32>(0.0, 1.0)), dot(c, vec2<f32>(0.9511, 0.3090))), max(dot(c, vec2<f32>(0.5878, -0.8090)), max(dot(c, vec2<f32>(-0.5878, -0.8090)), dot(c, vec2<f32>(-0.9511, 0.3090)))));
    let d2 = d / 0.809;
    let edge = 0.02 + params.soft / 100.0 * 0.08;
    let m = smoothstep(d2 - edge, d2 + edge, p);
    return mix(from_at(uv), to_at(uv), m);
}
