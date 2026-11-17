struct Params { duration: f32 }

// Rotate a point around the picture centre by `angle` radians.
fn spin_coord(uv: vec2<f32>, angle: f32) -> vec2<f32> {
    let ca = cos(angle);
    let sa = sin(angle);
    let d = uv - vec2<f32>(0.5);
    return vec2<f32>(d.x * ca - d.y * sa, d.x * sa + d.y * ca) + vec2<f32>(0.5);
}
// Each word makes one full 360-degree turn while fading out, last word first, over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let p = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let e = 1.0 - p;
    // Reversed order: 0 for the last word, so it leaves first.
    let rr = 1.0 - reveal_order(uv);
    let v = 1.0 - clamp((e - rr * 0.75) * 4.0, 0.0, 1.0);
    // At rest (v = 1) the angle is 0, so the rotation is the identity
    // and the word sits at its rest orientation.
    let angle = (1.0 - v) * 6.2831853;
    let coord = spin_coord(uv, angle);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let c = sample(coord);
    return vec4<f32>(c.rgb * v, c.a * v);
}
