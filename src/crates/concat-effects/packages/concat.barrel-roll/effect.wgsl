struct Params { shade: f32 }

// The whole frame rolls a full turn as the new shot lands.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let a = progress * 6.28318;
    let c = cos(a);
    let s = sin(a);
    let ruv = uv - vec2<f32>(0.5);
    let rot = vec2<f32>(ruv.x * c - ruv.y * s, ruv.x * s + ruv.y * c) + vec2<f32>(0.5);
    let shade = (1.0 - abs(c)) * params.shade / 100.0 * 0.4;
    let col = mix(from_at(rot), to_at(rot), step(0.5, progress));
    return vec4<f32>(col.rgb * (1.0 - shade), col.a);
}
