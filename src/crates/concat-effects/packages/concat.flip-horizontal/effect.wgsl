struct Params { shade: f32 }

// The frame flips like a card around its vertical axis, shaded at the turn.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let c = cos(progress * 3.14159);
    let sx = clamp(0.5 + (uv.x - 0.5) / max(abs(c), 0.08), 0.0, 1.0);
    let shade = (1.0 - abs(c)) * params.shade / 100.0 * 0.4;
    let col = mix(from_at(vec2<f32>(sx, uv.y)), to_at(vec2<f32>(sx, uv.y)), step(0.5, progress));
    return vec4<f32>(col.rgb * (1.0 - shade), col.a);
}
