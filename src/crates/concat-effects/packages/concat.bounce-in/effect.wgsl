struct Params { duration: f32 }

// A one-shot entrance, timed to the clip itself: `frame.clip_time` is
// zero at the clip's first frame however far into the timeline it sits.
// Ease-out-back: the picture pops in from small, overshoots its rest
// size a little, then settles. Past the duration it sits at rest.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    let p = 1.0 + c3 * pow(t - 1.0, 3.0) + c1 * pow(t - 1.0, 2.0);
    let scale = 0.6 + 0.4 * p;
    let coord = (uv - 0.5) / scale + 0.5;
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    return sample(coord);
}
