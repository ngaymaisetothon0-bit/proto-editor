struct Params { duration: f32 }

// A one-shot entrance, timed to the clip itself: `frame.clip_time` is
// zero at the clip's first frame however far into the timeline it sits,
// so the pop plays the same on every clip it is put on. Past the
// duration the picture sits at rest, untouched. The zoom is taken in
// pixels, so a frame wider than it is tall grows rigidly rather than
// shearing.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    // Cubic ease-out: fast off the mark, soft on landing.
    let p = 1.0 - pow(1.0 - t, 3.0);
    // 60% of size and fully transparent at the clip's first frame.
    let scale = 0.6 + 0.4 * p;
    let c = (uv - vec2<f32>(0.5)) * frame.size;
    let coord = c / scale / frame.size + vec2<f32>(0.5);
    let out = sample(coord);
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(out.rgb * fade, out.a * fade);
}
