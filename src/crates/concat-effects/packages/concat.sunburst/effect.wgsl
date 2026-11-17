struct Params { glow: f32 }

// Eight rays burst from the centre of the frame as the shots cross-fade.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = sin(progress * 3.14159) * params.glow / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let base = mix(from_at(uv), to_at(uv), p);
    let d = distance(uv, vec2<f32>(0.5, 0.5));
    let ang = atan2(uv.y - 0.5, uv.x - 0.5);
    let rays = 0.5 + 0.5 * cos(ang * 8.0);
    let burst = rays * exp(-d * 4.0) * k * 0.4;
    return base + vec4<f32>(vec3<f32>(burst), 0.0);
}
