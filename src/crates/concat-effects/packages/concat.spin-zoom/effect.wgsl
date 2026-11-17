struct Params { strength: f32 }

// The frame spins a quarter turn and rushes in as the shots cross-fade.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.strength / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let a = p * 1.5707963 * k;
    let z = 1.0 + sin(p * 3.14159) * 0.3 * k;
    let c = cos(a);
    let s = sin(a);
    let d = uv - vec2<f32>(0.5);
    let rot = vec2<f32>(d.x * c - d.y * s, d.x * s + d.y * c);
    let u = vec2<f32>(0.5) + rot / z;
    return mix(from_at(u), to_at(u), p);
}
