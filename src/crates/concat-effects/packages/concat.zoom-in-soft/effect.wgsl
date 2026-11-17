struct Params { duration: f32 }

// A one-shot entrance, timed to the clip itself: `frame.clip_time` is
// zero at the clip's first frame however far into the timeline it sits,
// so the push-in plays the same on every clip it is put on. Past the
// duration the picture sits at rest, untouched. The zoom is taken in
// pixels, so a frame wider than it is tall grows rigidly rather than
// shearing.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    // Cubic ease-out: fast off the mark, soft on landing.
    let p = 1.0 - pow(1.0 - t, 3.0);
    // 15% larger at the clip's first frame, easing to rest.
    let scale = 1.0 + 0.15 * (1.0 - p);
    let c = (uv - vec2<f32>(0.5)) * frame.size;
    let coord = c / scale / frame.size + vec2<f32>(0.5);
    return sample(coord);
}
