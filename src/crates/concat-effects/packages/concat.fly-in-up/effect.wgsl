struct Params { duration: f32 }

// A one-shot entrance, timed to the clip itself: `frame.clip_time` is
// zero at the clip's first frame however far into the timeline it sits,
// so the rise plays the same on every clip it is put on. Past the
// duration the picture sits at rest, untouched.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    // Cubic ease-out: fast off the mark, soft on landing.
    let p = 1.0 - pow(1.0 - t, 3.0);
    let shift = (1.0 - p) * 1.0;
    let coord = uv - vec2<f32>(0.0, shift);
    // The sampler clamps to the edge, so a coord outside the picture
    // would smear its edge instead of showing nothing: say so plainly.
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    return sample(coord);
}
