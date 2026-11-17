struct Params { tone: f32, fresh: f32 }

// A garden in daylight: fresh greens lifted, soft sun, everything a little more alive.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.0, 0.03, -0.01), tone);
    out = vibrance(out, 1.0 + tone * 0.22);
    out = white_balance(out, 6500.0 + params.fresh * 10.0);
    out = s_curve(out, params.fresh / 100.0 * 0.25);
    return vec4<f32>(out, c.a);
}
