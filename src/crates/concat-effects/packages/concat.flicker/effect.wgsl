struct Params { speed: f32 }

// The old shot flickers like a dying bulb, then the cut snaps to the new one.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.speed / 100.0;
    let flick = 0.7 + 0.3 * sin(progress * (20.0 + k * 60.0));
    let p = smoothstep(0.35, 0.65, progress);
    let from_c = from_at(uv);
    let base = mix(vec4<f32>(from_c.rgb * flick, from_c.a), to_at(uv), p);
    return base;
}
