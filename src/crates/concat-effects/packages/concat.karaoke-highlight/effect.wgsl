struct Params { words: f32, boundary1: f32, boundary2: f32, highlight_r: f32, highlight_g: f32, highlight_b: f32, important: f32 }

// For a title clip: the word being spoken is tinted the highlight colour, and the important word stays tinted when one is named. With no words set, the picture passes through unchanged.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let n = params.words;
    if (n < 1.0) {
        return c;
    }
    // Progress through the clip, 0..1.
    let p = clamp(frame.clip_time / max(frame.clip_duration, 0.001), 0.0, 1.0);
    // The word being spoken: before the first boundary it is word 0,
    // before the second it is word 1, after that word 2 - never past
    // the last word there is.
    let active_word = min(select(select(2.0, 1.0, p < params.boundary2), 0.0, p < params.boundary1), n - 1.0);
    // The word painted at this pixel, 0 for the first word.
    let w = round(reveal_order(uv) * (n - 1.0));
    if (w == active_word || w == params.important) {
        let hl = vec3<f32>(params.highlight_r, params.highlight_g, params.highlight_b);
        // Multiply, not replace: the glyphs keep their shape and alpha,
        // and the shadow and plate stay dark instead of turning gold.
        return vec4<f32>(c.rgb * hl, c.a);
    }
    return c;
}
