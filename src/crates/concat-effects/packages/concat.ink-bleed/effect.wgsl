struct Params { softness: f32 }

// The next shot bleeds in like ink on paper, the spreading edge dark before it turns.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let n = hash(floor(uv * vec2<f32>(6.0, 4.0)), 5.0) * 0.6
        + hash(floor(uv * vec2<f32>(18.0, 12.0)), 9.0) * 0.4;
    let w = 0.08 + params.softness / 100.0 * 0.25;
    let m = smoothstep(n - w, n + w, p);
    let edge = exp(-abs(n - p) * 10.0) * 0.2;
    let base = mix(from_at(uv), to_at(uv), m);
    return vec4<f32>(base.rgb * (1.0 - edge * (1.0 - m)), base.a);
}
