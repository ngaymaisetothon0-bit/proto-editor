// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Following one chosen point through a clip, for motion tracking.
//!
//! The user picks a point on the video - a face, a ball, a logo - and the
//! app follows it through the clip, so the picture can be made to follow
//! the point instead. This is the job that follows it: the media decoded
//! small and grey over the stretch the clip shows, the point carried from
//! frame to frame by Lucas-Kanade optical flow (see
//! `concat_vision::stabilize::track::follow_point`), and the positions it
//! was found at handed back in source order. Nothing is cached on disk:
//! tracking is started by the user, on one clip at a time, and the result
//! is written into the clip's keyframes straight away, so measuring again
//! is just tracking again.
//!
//! One job at a time through a [`SingleFlight`], like the stabilizer's.
//! When the flow loses the point - it left the frame, or the picture went
//! flat where it stood - the track ends there; what was found before that
//! is still returned, and the caller says so.

use std::sync::Arc;

use concat_core::time::Rational;
use concat_media::{DecodeOptions, Decoder, FrameSource};
use concat_vision::stabilize::track::{Gray, Pyramid, follow_point};
use concat_vision::stabilize::ANALYSIS_SIDE;

use crate::jobs::SingleFlight;

/// One point of the track: where in the source, and where in the picture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackPoint {
    /// Source seconds of the media file.
    pub at: f64,
    /// Across the picture, `0..=1` from the left edge.
    pub x: f64,
    /// Down the picture, `0..=1` from the top edge.
    pub y: f64,
}

/// What one tracking run follows: one media file, one stretch of it, one
/// point in the first frame.
#[derive(Clone, Debug)]
pub struct TrackRequest {
    /// The media file to follow the point through.
    pub media_path: String,
    /// The media's picture size, when the document knows it: what the
    /// analysis size is worked out from.
    pub media_size: Option<(u32, u32)>,
    /// Source seconds to follow from, the frame the point was picked in.
    pub from: f64,
    /// Source seconds to follow to.
    pub to: f64,
    /// The picked point, normalised `0..=1` in the media's own picture.
    pub point: (f64, f64),
}

/// The tracking service: the one-job slot.
pub struct Trackers {
    gate: Arc<SingleFlight>,
}

impl Default for Trackers {
    fn default() -> Self {
        Trackers::new()
    }
}

impl Trackers {
    /// A service with nothing running.
    pub fn new() -> Trackers {
        Trackers {
            gate: Arc::new(SingleFlight::new()),
        }
    }

    /// Whether a tracking run is going.
    pub fn is_busy(&self) -> bool {
        self.gate.is_busy()
    }

    /// Asks the running analysis to stop after the frame in hand.
    pub fn cancel(&self) {
        self.gate.cancel();
    }

    /// Follows the point through `request`'s stretch. Blocks for the whole
    /// run, so run it on its own thread; `progress` is told `0..=1` of the
    /// source seconds wanted as it goes. Returns the points found, in
    /// source order - up to where the point was lost, if it was.
    pub fn analyse(
        &self,
        request: &TrackRequest,
        progress: &mut dyn FnMut(f32),
    ) -> Result<Vec<TrackPoint>, String> {
        let job = self.gate.begin("motion tracking")?;
        let total = (request.to - request.from).max(0.0);
        if total <= 0.0 {
            return Ok(Vec::new());
        }
        let size = analysis_size(request)?;
        // From a little before the stretch, so the first frame recorded has
        // a frame before it to be followed from.
        let lead = (request.from - 0.1).max(0.0);
        let options = DecodeOptions::default()
            .starting_at(Rational::approximate(lead).unwrap_or(Rational::ZERO))
            .scaled_to(size.0, size.1);
        let mut decoder = Decoder::open(&request.media_path, &options)
            .map_err(|error| format!("{}: {error}", request.media_path))?;
        let (width, height) = (f64::from(size.0), f64::from(size.1));
        // The picked point, in analysis pixels.
        let mut point = (request.point.0 * width, request.point.1 * height);
        let mut previous: Option<Pyramid> = None;
        let mut points: Vec<TrackPoint> = Vec::new();
        progress(0.0);
        loop {
            if job.cancelled() {
                return Err("motion tracking cancelled".to_owned());
            }
            let Some(frame) = decoder.next_frame().map_err(|error| error.to_string())? else {
                break;
            };
            let at = decoder
                .position()
                .map(|position| position.as_f64())
                .unwrap_or_else(|| points.last().map_or(lead, |last| last.at + 1.0 / 30.0));
            if at > request.to {
                break;
            }
            let next = Pyramid::of(Gray::of_frame(&frame));
            if at >= request.from {
                match &previous {
                    None => {
                        // The first frame recorded is the one the point was
                        // picked in: the point starts where the user put it.
                        points.push(TrackPoint {
                            at,
                            x: point.0 / width,
                            y: point.1 / height,
                        });
                    }
                    Some(prev) => match follow_point(prev, &next, (point.0 as f32, point.1 as f32))
                    {
                        Some(found) => {
                            point = (f64::from(found.0), f64::from(found.1));
                            points.push(TrackPoint {
                                at,
                                x: point.0 / width,
                                y: point.1 / height,
                            });
                        }
                        // Lost: the track ends here, with what was found.
                        None => break,
                    },
                }
            }
            previous = Some(next);
            progress((((at - request.from).max(0.0)) / total).min(1.0) as f32);
        }
        progress(1.0);
        Ok(points)
    }
}

/// The size the analysis decodes `request`'s media at: its longest side
/// [`ANALYSIS_SIDE`], or its own when smaller, even on both sides. The
/// file is opened to ask when the document does not know.
fn analysis_size(request: &TrackRequest) -> Result<(u32, u32), String> {
    let (width, height) = match request.media_size.filter(|(w, h)| *w > 0 && *h > 0) {
        Some(size) => size,
        None => {
            let decoder =
                Decoder::open(&request.media_path, &DecodeOptions::default().limited_to(1))
                    .map_err(|error| format!("{}: {error}", request.media_path))?;
            (decoder.width(), decoder.height())
        }
    };
    if width == 0 || height == 0 {
        return Err(format!("{}: no picture to track", request.media_path));
    }
    let longest = width.max(height);
    let scale = (f64::from(ANALYSIS_SIDE) / f64::from(longest)).min(1.0);
    let even = |side: u32| (((f64::from(side) * scale).round() as u32).max(16)) & !1;
    Ok((even(width), even(height)))
}
