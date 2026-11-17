struct Params { duration: f32 }

// A one-shot entrance, timed to the clip itself: `frame.clip_time` is
// zero at the clip's first frame however far into the timeline it sits,
// so the fade plays the same on every clip it is put on. Past the
// duration the picture sits at full strength, untouched.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = smoothstep(0.0, 1.0, t);
    let c = sample(uv);
    return vec4<f32>(c.rgb * p, c.a * p);
}
