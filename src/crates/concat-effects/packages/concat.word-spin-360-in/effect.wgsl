struct Params { duration: f32 }

// Rotate a point around the picture centre by `angle` radians.
fn spin_coord(uv: vec2<f32>, angle: f32) -> vec2<f32> {
    let ca = cos(angle);
    let sa = sin(angle);
    let d = uv - vec2<f32>(0.5);
    return vec2<f32>(d.x * ca - d.y * sa, d.x * sa + d.y * ca) + vec2<f32>(0.5);
}
// Each word makes one full 360-degree turn while fading in, one after another.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let w = clamp((p - reveal_order(uv) * 0.75) * 4.0, 0.0, 1.0);
    // At w = 1 the angle is 0 - the full turn is spent - so the
    // rotation is the identity and the word sits at its rest
    // orientation.
    let angle = -(1.0 - w) * 6.2831853;
    let coord = spin_coord(uv, angle);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let c = sample(coord);
    return vec4<f32>(c.rgb * w, c.a * w);
}
