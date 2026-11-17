struct Params { duration: f32 }

// Seconds left of the clip, or a very long time where the clip has no
// known end - a treatment, a probe - so the look holds its rest there.
fn remain() -> f32 {
    return select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
}

// A one-shot exit, timed to the clip's end: past the duration the
// picture sits at rest, untouched, and only the clip's last moments
// pull back. The zoom is taken in pixels, so a frame wider than it
// is tall grows rigidly rather than shearing.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(remain() / max(params.duration, 0.001), 0.0, 1.0);
    // Cubic ease-out: fast off the mark, soft on landing.
    let p = 1.0 - pow(1.0 - t, 3.0);
    // 15% larger at the clip's last frame, easing back to rest.
    let scale = 1.0 + 0.15 * (1.0 - p);
    let c = (uv - vec2<f32>(0.5)) * frame.size;
    let coord = c / scale / frame.size + vec2<f32>(0.5);
    return sample(coord);
}
