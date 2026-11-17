struct Params { duration: f32 }

// Seconds left of the clip, or a very long time where the clip has no
// known end - a treatment, a probe - so the look holds its rest there.
fn remain() -> f32 {
    return select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
}

// A one-shot exit, timed to the clip's end: past the duration the
// picture sits at rest, untouched, and only the clip's last moments
// sink away.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(remain() / max(params.duration, 0.001), 0.0, 1.0);
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
