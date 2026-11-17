struct Params { shade: f32 }

// The frame flips like a card around its horizontal axis, shaded at the turn.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let c = cos(progress * 3.14159);
    let sy = clamp(0.5 + (uv.y - 0.5) / max(abs(c), 0.08), 0.0, 1.0);
    let shade = (1.0 - abs(c)) * params.shade / 100.0 * 0.4;
    let col = mix(from_at(vec2<f32>(uv.x, sy)), to_at(vec2<f32>(uv.x, sy)), step(0.5, progress));
    return vec4<f32>(col.rgb * (1.0 - shade), col.a);
}
