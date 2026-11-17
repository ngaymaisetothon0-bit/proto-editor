// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Beat detection from a file's waveform peaks: the tempo and the beat
//! times, for cutting to the music the way CapCut's beat sync does.
//!
//! The detector works on the peaks the timeline already draws, so it
//! needs no second decode and no new cache: an onset envelope from the
//! bucket extremes, a pulse search over tempos, and each predicted beat
//! snapped to the loudest onset nearby. It is tuned for music with a
//! steady pulse - a kick on most beats - and says None rather than
//! inventing a grid when there is none, as on speech or flat noise.
//!
//! The maths was prototyped in Python first (see `beat_proto.py` in the
//! assistant's workspace) and checked there on synthetic click trains:
//! 90, 100, 120 and 128 BPM all land exact, within 20 ms a beat, a
//! 120-to-122 BPM drift reads as its middle, and flat noise reads as no
//! beats at all.

use crate::peaks::Peaks;

/// The beats of one file: its tempo, and the beat times in seconds of the
/// file, from its start.
#[derive(Clone, Debug, PartialEq)]
pub struct Beats {
    /// Beats per minute.
    pub bpm: f32,
    /// Beat times, ascending, in seconds of the media file.
    pub beats: Vec<f32>,
}

/// The tempo range the pulse search covers, in BPM.
const MIN_BPM: f64 = 60.0;
/// The tempo range the pulse search covers, in BPM.
const MAX_BPM: f64 = 180.0;
/// The step of the coarse tempo grid, in BPM.
const COARSE_STEP_BPM: f64 = 0.5;
/// How far either side of the coarse pick the fine grid searches, in BPM.
const REFINE_SPAN_BPM: f64 = 1.0;
/// The step of the fine tempo grid, in BPM.
const REFINE_STEP_BPM: f64 = 0.05;
/// How far a predicted beat may move to sit on its onset, in seconds.
const SNAP_WINDOW: f64 = 0.06;
/// A beat counts only when its flux clears this share of the loudest flux.
const BEAT_FLUX_FRACTION: f64 = 0.05;
/// A grid counts only when its mean flux clears this multiple of the
/// flux's overall mean - the check that keeps flat noise beatless.
const CONFIDENCE: f64 = 2.0;
/// A grid counts only when a shorter period does nearly as well: the
/// check that keeps a half-tempo read from winning.
const OCTAVE_FRACTION: f64 = 0.85;
/// The fewest beats a detection may hold.
const MIN_BEATS: usize = 4;

/// The tempo and beat times of these peaks, or None when there is no
/// steady pulse to find.
pub fn detect_beats(peaks: &Peaks) -> Option<Beats> {
    let rate = f64::from(peaks.buckets_per_second);
    let n = peaks.len();
    if n == 0 || rate <= 0.0 {
        return None;
    }
    // The onset envelope: how much louder each bucket is than the one
    // before it. A kick is a rise; a decay is not, so falls read as zero.
    let mut flux = vec![0.0f64; n];
    let mut flux_max = 0.0f64;
    let mut previous = 0.0f64;
    for index in 0..n {
        let energy = (-peaks.min[index]).max(peaks.max[index]) as f64;
        if index > 0 {
            let rise = energy - previous;
            if rise > 0.0 {
                flux[index] = rise;
                if rise > flux_max {
                    flux_max = rise;
                }
            }
        }
        previous = energy;
    }
    if flux_max <= 0.0 {
        return None;
    }
    // The pulse search: for each tempo and each phase, the mean flux at
    // the predicted beats. The best mean wins the tempo - but a period
    // twice the true one scores nearly as well, since it still lands on
    // every other onset, so among the periods within reach of the best the
    // shortest is taken, which is the true tempo and not its half.
    let mean_at = |period: f64, phase: usize| -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0usize;
        let mut pos = phase as f64;
        while pos < n as f64 {
            let at = pos.round() as usize;
            if at < n {
                sum += flux[at];
                count += 1;
            }
            pos += period;
        }
        if count >= MIN_BEATS {
            Some(sum / count as f64)
        } else {
            None
        }
    };
    let period_of = |bpm: f64| 60.0 / bpm * rate;
    let mut best: Option<(f64, f64, usize)> = None;
    let mut bpm = MIN_BPM;
    while bpm <= MAX_BPM + 1e-9 {
        let period = period_of(bpm);
        for phase in 0..period.ceil() as usize {
            if let Some(score) = mean_at(period, phase)
                && best.is_none_or(|(top, _, _)| score > top)
            {
                best = Some((score, period, phase));
            }
        }
        bpm += COARSE_STEP_BPM;
    }
    let (top, _, _) = best?;
    if top < BEAT_FLUX_FRACTION * flux_max {
        return None;
    }
    let mut picked: Option<(f64, usize)> = None;
    bpm = MIN_BPM;
    while bpm <= MAX_BPM + 1e-9 {
        let period = period_of(bpm);
        for phase in 0..period.ceil() as usize {
            if let Some(score) = mean_at(period, phase)
                && score >= OCTAVE_FRACTION * top
                && picked.is_none_or(|(shortest, _)| period < shortest)
            {
                picked = Some((period, phase));
            }
        }
        bpm += COARSE_STEP_BPM;
    }
    let (mut period, mut phase) = picked?;
    let mut bpm = 60.0 / (period / rate);
    // The fine grid: a quarter of a BPM off, over a three-minute song,
    // drifts the last beat by most of a second, so the tempo is refined
    // to a twentieth of a BPM around the coarse pick.
    let mut best_score = f64::NEG_INFINITY;
    for step in -(REFINE_SPAN_BPM / REFINE_STEP_BPM) as i32..=(REFINE_SPAN_BPM / REFINE_STEP_BPM) as i32 {
        let candidate = bpm + step as f64 * REFINE_STEP_BPM;
        if candidate < MIN_BPM || candidate > MAX_BPM {
            continue;
        }
        let candidate_period = period_of(candidate);
        for candidate_phase in 0..candidate_period.ceil() as usize {
            if let Some(score) = mean_at(candidate_period, candidate_phase)
                && score > best_score
            {
                best_score = score;
                bpm = candidate;
                period = candidate_period;
                phase = candidate_phase;
            }
        }
    }
    // Confidence, judged on the raw grid before any snapping: a snapped
    // position takes the loudest bucket nearby, which would flatter the
    // score even on noise.
    let overall = flux.iter().sum::<f64>() / n as f64;
    let mut grid: Vec<usize> = Vec::new();
    let mut pos = phase as f64;
    while pos < n as f64 {
        grid.push((pos.round() as usize).min(n - 1));
        pos += period;
    }
    if grid.len() < MIN_BEATS || overall <= 0.0 {
        return None;
    }
    let grid_mean = grid.iter().map(|&at| flux[at]).sum::<f64>() / grid.len() as f64;
    if grid_mean < CONFIDENCE * overall {
        return None;
    }
    // The beats: the grid, each moved to the loudest onset within a
    // hair either side, so a beat that drifts a little still lands on
    // the drum that made it.
    let window = (SNAP_WINDOW * rate).round() as usize;
    let period_seconds = period / rate;
    let mut beats: Vec<f32> = Vec::new();
    let mut pos = phase as f64;
    while pos < n as f64 {
        let centre = pos.round() as usize;
        let from = centre.saturating_sub(window);
        let to = (centre + window + 1).min(n);
        let mut loudest = from;
        for at in from + 1..to {
            if flux[at] > flux[loudest] {
                loudest = at;
            }
        }
        let time = loudest as f64 / rate;
        if flux[loudest] > BEAT_FLUX_FRACTION * flux_max
            && beats
                .last()
                .is_none_or(|&last| time - f64::from(last) > 0.5 * period_seconds)
        {
            beats.push(time as f32);
        }
        pos += period;
    }
    if beats.len() < MIN_BEATS {
        return None;
    }
    Some(Beats {
        bpm: bpm as f32,
        beats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Peaks for an energy array at 200 buckets a second.
    fn peaks_of(energy: &[f32]) -> Peaks {
        Peaks {
            min: energy.iter().map(|&value| -0.9 * value).collect(),
            max: energy.to_vec(),
            buckets_per_second: 200.0,
        }
    }

    /// A click train: a loud bucket or two on each beat, quiet noise
    /// elsewhere, with optional quieter hits halfway between beats.
    fn click_train(bpm: f64, seconds: f64, half: bool) -> Peaks {
        let rate = 200.0;
        let count = (seconds * rate) as usize;
        let mut energy = vec![0.03f32; count];
        let period = 60.0 / bpm;
        let mut at = 0.0;
        while at < seconds {
            let index = (at * rate) as usize;
            if index < count {
                energy[index] = 1.0;
                if index + 1 < count {
                    energy[index + 1] = 1.0;
                }
            }
            if half {
                let off = ((at + period / 2.0) * rate) as usize;
                if off < count {
                    energy[off] = 0.55;
                    if off + 1 < count {
                        energy[off + 1] = 0.55;
                    }
                }
            }
            at += period;
        }
        peaks_of(&energy)
    }

    fn nearest_error(beats: &[f32], bpm: f64) -> f64 {
        let period = 60.0 / bpm;
        let first = f64::from(beats[0]);
        beats
            .iter()
            .map(|&beat| {
                let offset = (f64::from(beat) - first).rem_euclid(period);
                offset.min(period - offset)
            })
            .fold(0.0, f64::max)
    }

    #[test]
    fn a_clean_click_train_reads_exact() {
        let found = detect_beats(&click_train(120.0, 30.0, false)).expect("beats");
        assert!((f64::from(found.bpm) - 120.0).abs() < 0.6);
        assert_eq!(found.beats.len(), 59);
        assert!(nearest_error(&found.beats, 120.0) < 0.05);
    }

    #[test]
    fn off_beat_hats_do_not_double_the_tempo() {
        let found = detect_beats(&click_train(100.0, 30.0, true)).expect("beats");
        assert!((f64::from(found.bpm) - 100.0).abs() < 0.6);
        assert!(nearest_error(&found.beats, 100.0) < 0.05);
    }

    #[test]
    fn flat_noise_has_no_beats() {
        let energy = vec![0.2f32; 20 * 200];
        assert!(detect_beats(&peaks_of(&energy)).is_none());
    }

    #[test]
    fn empty_peaks_have_no_beats() {
        let empty = Peaks {
            min: Vec::new(),
            max: Vec::new(),
            buckets_per_second: 200.0,
        };
        assert!(detect_beats(&empty).is_none());
    }
}
