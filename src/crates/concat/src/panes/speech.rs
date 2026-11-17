// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The speech sheet: a title's words, or any words, read aloud.
//!
//! It opens on the selected title's words when a title is selected, else
//! on a blank script to be read at the playhead. The voice is the one
//! chosen last time. The synthesiser runs on a worker and reports as
//! [`SpeechMsg::Progress`], ending as [`SpeechMsg::Finished`] with the
//! written file, which lands in the bin and on the timeline.
//!
//! The model comes first and decides the rest: which voices are offered.
//! VieNeu reads with its 25 built-in Vietnamese speakers at its own pace.
//! The older families - Kokoro, Pocket TTS, VITS (VIVOS), Chatterbox -
//! were removed in 2026-11; the sheet's speed, pauses, quality and
//! sample sections belonged to them and never show for VieNeu.

use std::path::PathBuf;
use std::sync::Arc;

use concat_host::media::{self, MediaSummary};
use concat_host::voices::{self, SavedVoice};
use concat_project::Command;
use concat_project::commands::ClipPatch;
use concat_project::model::{AppliedFilter, ClipKind, MediaItem, MediaOrigin};
use concat_speech::JobStage;
use concat_speech::tts::{
    Family, VoiceInfo, family_of, vieneu_voice_detail, voice_family,
};
use slint::SharedString;

use crate::format::wave_path;
use crate::host::{on_ui_in_project, spawn_in_project};
use crate::i18n::{t, tf};
use crate::panes::Msg;
use crate::panes::captions::CHARS_PER_SECOND;
use crate::panes::pace::{Pace, Stage, Timings};
use crate::panes::settings::installed;
use crate::studio::Studio;
use crate::ui::SpeechSheetData;

/// The default VieNeu voice: the model's own default, Hải Đăng.
const DEFAULT_VIENEU_VOICE: i32 = 4004;
/// The speeds a voice can be asked for, as the rate multiplier the sheet's
/// slider runs over; 1 is the voice's own.
const SPEEDS: (f32, f32) = (0.5, 2.0);
/// The flow steps behind the sheet's three qualities: quick, the
/// engine's own, and fine. Kept for the sheet's quality control, which
/// VieNeu never shows.
const STEPS: [i32; 3] = [3, 5, 8];
/// The breath between sentences the sheet opens on.
const DEFAULT_PAUSES: f32 = 0.2;

/// A style a voice can be read in: a few of the audio effects Concat ships,
/// laid on the speech clip once it lands, where they stay as ordinary
/// effects to change or take off. Original looks of the effects alone -
/// no one's voice, no character's name. The Audio page's card carries
/// its row; `name` names it in the interface's language, through a
/// literal `t` so scripts/locales.py finds the key.
pub struct VoiceStyle {
    pub name: fn() -> String,
    pub effects: &'static [&'static str],
}

/// The Audio page's Voices › Styles shelf, in its order.
pub const STYLES: [VoiceStyle; 6] = [
    VoiceStyle {
        name: || t("voices.styleChipmunk"),
        effects: &["concat.chipmunk"],
    },
    VoiceStyle {
        name: || t("voices.styleRobot"),
        effects: &["concat.robot"],
    },
    VoiceStyle {
        name: || t("voices.styleTelephone"),
        effects: &["concat.telephone"],
    },
    VoiceStyle {
        name: || t("voices.styleMegaphone"),
        effects: &["concat.megaphone"],
    },
    VoiceStyle {
        name: || t("voices.styleRadio"),
        effects: &["concat.radio"],
    },
    VoiceStyle {
        name: || t("voices.styleGiant"),
        effects: &["concat.deep", "concat.hall"],
    },
];

/// What the Audio page's Voices shelf opens the sheet on.
#[derive(Clone, Debug)]
pub enum VoiceChoice {
    /// A voice a model names, by its id.
    Builtin(i32),
    /// A style, by its row of [`STYLES`].
    Style(usize),
}

/// Seconds the model takes to read one second of speech, before a read
/// with it has been timed. Measured on this machine: about 3.0 s of wall
/// clock for 5.7 s of audio.
fn reading_pace() -> f32 {
    0.55
}

/// Everything that can happen to the speech sheet.
#[derive(Clone, Debug)]
pub enum SpeechMsg {
    /// The tray's Speak tool, or the menu.
    Open,
    /// The Audio page's Voices shelf: the sheet with that voice or style.
    OpenWith(VoiceChoice),
    /// The name a sample is to be saved under, typed.
    SaveNameEdited(String),
    /// A picture for the voice being saved, from the system's picker.
    PicturePicked(PathBuf),
    /// Keep the sample being read in as a voice of the user's own.
    SaveVoice,
    /// The save's worker is done.
    VoiceSaved(Box<Result<SavedVoice, String>>),
    /// A saved voice deleted, by its id.
    ForgetVoice(String),
    /// The style taken off: the speech lands plain.
    ClearStyle,
    Close,
    TextEdited(String),
    VoiceChanged(i32),
    ModelChanged(i32),
    /// The rate, `0.5..=2`.
    SpeedChanged(f32),
    /// The quality: a row of [`STEPS`].
    QualityChanged(i32),
    /// The breath between sentences, `0..=1`.
    PausesChanged(f32),
    /// Seconds into the sample the voice is taken from.
    ReferenceStartChanged(f32),
    /// Read in a sample from the bin rather than a named voice.
    UseSampleChanged(bool),
    /// Which sample: a row of [`SpeechPane::samples`].
    SampleChanged(i32),
    /// Read the script.
    Begin,
    Cancel,
    /// Hear the chosen voice read one sentence, before a whole read.
    PreviewVoice,
    /// The preview's worker is done: the WAV it wrote, the voice's
    /// name, and the voice's id - the card the preview plays under,
    /// so another voice loads its own file instead of toggling the
    /// one already playing - or why not.
    PreviewFinished(Box<Result<(std::path::PathBuf, String, i32), String>>),
    /// The synthesiser's worker reporting where it is, `0..=1`.
    Progress(JobStage),
    /// The synthesiser's worker is done: the file it wrote, probed, or
    /// why not.
    Finished(Box<Result<MediaSummary, String>>),
}

/// The speech sheet's state.
#[derive(Default)]
pub struct SpeechPane {
    pub open: bool,
    /// The title the words came from, and where the sound lands. None
    /// reads a script of its own at the playhead.
    pub clip: Option<String>,
    pub text: String,
    /// Row in the chosen model's voices; see [`SpeechPane::offered`].
    pub voice: usize,
    /// Row in the installed voice model list.
    pub model: usize,
    /// The family of the chosen model, which is which voices are offered
    /// and which of the dials below are read.
    pub family: Family,
    /// The rate, `0.5..=2`; kept for the sheet's slider, which VieNeu
    /// never shows - it reads at its own pace.
    pub speed: f32,
    /// Row of [`STEPS`]; kept for the sheet's quality control, which
    /// VieNeu never shows.
    pub quality: usize,
    /// The breath between sentences, `0..=1`; kept for the sheet's
    /// pauses slider, which VieNeu never shows.
    pub pauses: f32,
    /// Reading in a sample from the bin rather than a named voice;
    /// VieNeu never does, so the sheet never offers it.
    pub use_sample: bool,
    /// Row of [`SpeechPane::samples`]: which recording.
    pub sample: usize,
    /// Seconds into the sample the voice is taken from: the first ten
    /// seconds from there are what the model listens to, and the start of
    /// a recording is not always where the speaking is.
    pub reference_start: f64,
    pub running: bool,
    /// A preview read is running.
    pub previewing: bool,
    /// How far the running read looks.
    pub pace: Pace,
    /// The model and the length of the running read: what its timing is
    /// kept by.
    reading: Option<(String, f32)>,
    /// How long each model took this session, for the next read's bar.
    timings: Timings,
    pub message: String,
    /// Where the running read will land on the timeline, in seconds.
    landing: f64,
    /// Who can speak, from the voice model on disk.
    pub speakers: Vec<VoiceInfo>,
    /// The voices the user saved, newest first. No model reads in one
    /// any more, so the list is never offered. See `concat_host::voices`.
    pub saved: Vec<SavedVoice>,
    /// The saved voice being read in, by its row of `saved`, in place of
    /// a named one. Never set now that no model reads in one.
    pub saved_voice: Option<usize>,
    /// The style the speech lands in: a row of [`STYLES`].
    pub style: Option<usize>,
    /// What the sample is to be saved as, and with which picture.
    pub save_name: String,
    pub save_picture: Option<PathBuf>,
    /// A save running.
    pub saving: bool,
    /// Each saved voice's picture, read once, by its id: the rows are
    /// published on every frame of playback.
    pictures: std::cell::RefCell<std::collections::HashMap<String, slint::Image>>,
}

impl SpeechPane {
    /// Applies one message. The studio is the rest of the window; while
    /// this runs the studio's copy of the pane is a blank it must not read.
    pub fn update(&mut self, msg: SpeechMsg, studio: &mut Studio) {
        match msg {
            // A reading in progress is shown as it is when the panel is
            // asked for again: closing it never stopped the reading.
            SpeechMsg::Open if self.running => self.open = true,
            SpeechMsg::Open => {
                let title = studio
                    .sole_selection()
                    .and_then(|id| studio.clip(&id))
                    .filter(|clip| clip.kind == ClipKind::Text)
                    .cloned();
                let installed = installed(&studio.settings.voices);
                let model = installed.iter().position(|model| model.active).unwrap_or(0);
                let family = installed
                    .get(model)
                    .map(|model| family_of(&model.id))
                    .unwrap_or(Family::VieNeu);
                *self = SpeechPane {
                    open: true,
                    clip: title.as_ref().map(|clip| clip.id.clone()),
                    text: title
                        .and_then(|clip| clip.text.map(|text| text.content))
                        .unwrap_or_default(),
                    voice: 0,
                    model,
                    family,
                    speed: 1.0,
                    quality: 1,
                    pauses: DEFAULT_PAUSES,
                    use_sample: false,
                    sample: 0,
                    reference_start: 0.0,
                    speakers: std::mem::take(&mut self.speakers),
                    timings: std::mem::take(&mut self.timings),
                    saved: std::mem::take(&mut self.saved),
                    pictures: std::mem::take(&mut self.pictures),
                    ..SpeechPane::default()
                };
                // The voice chosen last time, if the model reads with it;
                // and the selected clip's recording as the sample to start
                // from, since that is the voice most likely wanted.
                let wanted = studio.prefs.tts_voice;
                self.voice = self.voice_row(wanted);
                self.sample = self.default_sample(studio);
            }
            SpeechMsg::OpenWith(choice) => {
                if !self.running {
                    self.update(SpeechMsg::Open, studio);
                    self.choose(choice, studio);
                } else {
                    self.open = true;
                }
            }
            SpeechMsg::SaveNameEdited(name) => self.save_name = name,
            SpeechMsg::PicturePicked(path) => self.save_picture = Some(path),
            SpeechMsg::SaveVoice => self.save_voice(studio),
            SpeechMsg::VoiceSaved(result) => {
                self.saving = false;
                match *result {
                    Ok(voice) => {
                        self.reload_saved(studio);
                        // Read in it from here on: it is the voice that
                        // was being read in, now under its name.
                        self.saved_voice = self.saved.iter().position(|held| held.id == voice.id);
                        if self.saved_voice.is_some() {
                            self.use_sample = false;
                        }
                        self.save_name.clear();
                        self.save_picture = None;
                        studio.notify(&tf("voices.saved", &[&voice.name]), false);
                    }
                    Err(error) => self.message = error,
                }
            }
            SpeechMsg::ForgetVoice(id) => {
                let Some(voice) = self.saved.iter().find(|voice| voice.id == id).cloned() else {
                    return;
                };
                match voices::remove(&voice) {
                    Ok(()) => {
                        self.saved_voice = None;
                        self.reload_saved(studio);
                        studio.notify(&tf("voices.removed", &[&voice.name]), false);
                    }
                    Err(error) => studio.notify(&error, true),
                }
            }
            SpeechMsg::ClearStyle => self.style = None,
            SpeechMsg::Close => self.open = false,
            SpeechMsg::TextEdited(text) => self.text = text,
            // Past the named voices would be the saved voices and the
            // clone row; no model offers either now, so only a named
            // voice is ever chosen.
            SpeechMsg::VoiceChanged(index) => {
                let index = index.max(0) as usize;
                let named = self.offered().len();
                let saved = self.saved_offered();
                if index < named {
                    self.voice = index;
                    self.saved_voice = None;
                } else if index < named + saved {
                    self.saved_voice = Some(index - named);
                } else if self.sample_voice().is_some() {
                    self.use_sample = true;
                }
            }
            SpeechMsg::ModelChanged(index) => {
                self.model = index.max(0) as usize;
                // Another model may read with other voices: the list is
                // rebuilt, and the row points at a voice on it.
                let chosen = self.offered().get(self.voice).map(|voice| voice.id);
                self.family = installed(&studio.settings.voices)
                    .get(self.model)
                    .map(|model| family_of(&model.id))
                    .unwrap_or(Family::VieNeu);
                self.voice = self.voice_row(chosen);
                if self.sample_voice().is_none() {
                    self.saved_voice = None;
                }
            }
            SpeechMsg::SpeedChanged(speed) => {
                self.speed = if speed.is_finite() {
                    speed.clamp(SPEEDS.0, SPEEDS.1)
                } else {
                    1.0
                }
            }
            SpeechMsg::QualityChanged(index) => {
                self.quality = (index.max(0) as usize).min(STEPS.len() - 1)
            }
            SpeechMsg::PausesChanged(pauses) => {
                self.pauses = if pauses.is_finite() {
                    pauses.clamp(0.0, 1.0)
                } else {
                    DEFAULT_PAUSES
                }
            }
            SpeechMsg::ReferenceStartChanged(seconds) => {
                self.reference_start = if seconds.is_finite() {
                    f64::from(seconds.max(0.0))
                } else {
                    0.0
                }
            }
            SpeechMsg::UseSampleChanged(on) => self.use_sample = on,
            SpeechMsg::SampleChanged(index) => {
                self.sample = index.max(0) as usize;
                self.reference_start = 0.0;
            }
            SpeechMsg::Begin => self.run(studio),
            SpeechMsg::Cancel => {
                studio.host.speech.cancel();
                self.running = false;
                self.open = false;
            }
            SpeechMsg::PreviewVoice => self.preview_voice(studio),
            SpeechMsg::PreviewFinished(result) => {
                self.previewing = false;
                match *result {
                    Ok((path, name, voice)) => {
                        studio.preview_file(&format!("voice-preview-{voice}"), &name, &path)
                    }
                    Err(error) => self.message = error,
                }
            }
            SpeechMsg::Progress(stage) => self.pace.report(stage),
            SpeechMsg::Finished(result) => {
                self.running = false;
                if let (Ok(_), Some((model, chars))) = (result.as_ref(), self.reading.take()) {
                    self.timings.keep(&model, self.pace.learned(chars));
                }
                match *result {
                    Ok(summary) => {
                        // Marked as read aloud: the bin shelves it under
                        // Generated › Speech, apart from the imports.
                        let mut item = summary.to_new_media();
                        item.origin = Some(MediaOrigin::Speech);
                        let created = studio.apply(Command::AddMedia { item });
                        let media_id = created.or_else(|| {
                            studio
                                .project()
                                .media
                                .iter()
                                .find(|item| item.path == summary.path)
                                .map(|item| item.id.clone())
                        });
                        self.open = false;
                        if let Some(media_id) = media_id {
                            let clip = studio.apply(Command::AddClipAtFirstFree {
                                media_id,
                                start: self.landing,
                            });
                            // In a style: its effects on the clip, as the
                            // library would put them there.
                            if let (Some(clip_id), Some(style)) =
                                (clip, self.style.and_then(|row| STYLES.get(row)))
                            {
                                studio.apply(Command::UpdateClip {
                                    clip_id,
                                    patch: ClipPatch {
                                        filters: Some(
                                            style
                                                .effects
                                                .iter()
                                                .map(|id| AppliedFilter::new(*id))
                                                .collect(),
                                        ),
                                        ..ClipPatch::default()
                                    },
                                });
                            }
                            studio.notify(&t("speech.voiceAddedTimeline"), false);
                        }
                    }
                    // Asked for: the sheet is already on its way down.
                    Err(error) if error.contains("cancel") => self.open = false,
                    // Closed while it ran: the panel is not there to say
                    // it, so the notice does, and the panel too next time.
                    Err(error) => {
                        if !self.open {
                            studio.notify(&error, true);
                        }
                        self.message = error;
                    }
                }
            }
        }
    }

    /// Hear the chosen voice read one sentence: a short synthesis on a
    /// worker, played through the source preview when it lands.
    fn preview_voice(&mut self, studio: &mut Studio) {
        if self.running || self.previewing {
            return;
        }
        let Some((model, _)) = installed(&studio.settings.voices)
            .get(self.model)
            .map(|model| (model.id.clone(), model.megabytes))
        else {
            self.message = t("speech.downloadVoiceModelFirst");
            return;
        };
        let Some(voice) = self.offered().get(self.voice).map(|speaker| speaker.id) else {
            self.message = t("speech.noVoiceRead");
            return;
        };
        let name = self
            .offered()
            .get(self.voice)
            .map(|speaker| voice_label(&speaker.name).0)
            .unwrap_or_default();
        let text = "Xin chào, đây là giọng đọc thử.";
        let dirs = studio.host.dirs.clone();
        let speech = Arc::clone(&studio.host.speech);
        self.previewing = true;
        self.message.clear();
        spawn_in_project(
            move || {
                let path = speech.preview_voice(&dirs, &model, voice, text)?;
                Ok::<_, String>((path, name, voice))
            },
            |studio, _, _, result| {
                studio.handle(Msg::Speech(SpeechMsg::PreviewFinished(Box::new(result))))
            },
        );
    }

    /// Reads the script on a worker: the WAV lands in the bin and on the
    /// timeline, at the title's start or at the playhead.
    fn run(&mut self, studio: &mut Studio) {
        let text = self.text.trim().to_owned();
        if text.is_empty() {
            self.message = t("speech.nothingToReadYet");
            return;
        }
        let Some((model, megabytes)) = installed(&studio.settings.voices)
            .get(self.model)
            .map(|model| (model.id.clone(), model.megabytes))
        else {
            self.message = t("speech.downloadVoiceModelFirst");
            return;
        };
        // The voice is one the model names.
        let Some(voice) = self.offered().get(self.voice).map(|speaker| speaker.id) else {
            self.message = t("speech.noVoiceRead");
            return;
        };
        let Some(project) = studio
            .session
            .as_ref()
            .map(|session| session.path().to_owned())
        else {
            return;
        };
        // Remembered: the voice chosen is the voice wanted next time.
        studio.prefs.tts_voice = Some(voice);
        studio.prefs.save(&studio.host.dirs);
        self.landing = self
            .clip
            .as_ref()
            .and_then(|id| studio.clip(id))
            .map(|clip| clip.start)
            .unwrap_or(f64::from(studio.playhead));
        // The read as the bar lays it out: the model loaded unless it
        // still is, and the reading at the pace the last read with this
        // model measured, or the model's usual pace.
        let chars = text.chars().count() as f32;
        let learned = self.timings.get(&model);
        let loaded = studio.host.speech.loaded_model().as_deref() == Some(model.as_str());
        let load = if loaded {
            0.0
        } else {
            learned.load.unwrap_or(0.5 + megabytes / 150.0)
        };
        let per_char = learned
            .per_unit
            .unwrap_or(reading_pace() / CHARS_PER_SECOND / self.rate());
        let mut plan = vec![(Stage::Loading, load)];
        plan.push((Stage::Working, chars * per_char));
        self.pace = Pace::new(plan);
        self.reading = Some((model.clone(), chars));
        // VieNeu reads at its own pace and with its own voices: no
        // reference, no pauses, no steps.
        let request = concat_speech::tts::SpeakRequest {
            model_id: model,
            voice,
            reference: None,
            text,
            speed: self.rate(),
            pauses: None,
            steps: None,
            project,
        };
        let dirs = studio.host.dirs.clone();
        let speech = Arc::clone(&studio.host.speech);
        self.running = true;
        self.message.clear();
        let epoch = crate::host::project_epoch();
        spawn_in_project(
            move || {
                let spoken = speech.speak(&dirs, &request, move |stage| {
                    on_ui_in_project(epoch, move |studio, _, _| {
                        studio.handle(Msg::Speech(SpeechMsg::Progress(stage)));
                    });
                })?;
                let summary = media::probe(&spoken.path)?;
                Ok::<_, String>(summary)
            },
            |studio, _, _, result| {
                studio.handle(Msg::Speech(SpeechMsg::Finished(Box::new(result))))
            },
        );
    }

    /// What the sheet's script would take to say, for the line under it.
    fn estimate(&self) -> String {
        let chars = self.text.trim().chars().count();
        if chars == 0 {
            return t("speech.nothingToReadYet");
        }
        let seconds = chars as f32 / CHARS_PER_SECOND / self.rate();
        let whole = seconds.round() as i32;
        let voice = self
            .offered()
            .get(self.voice)
            .map(|speaker| voice_label(&speaker.name).0)
            .unwrap_or_default();
        tf(
            "speech.aboutInCharacters",
            &[&(whole / 60), &format!("{:02}", whole % 60), &voice, &chars],
        )
    }

    /// Whether the chosen model reads at a speed and with a pause of the
    /// sheet's choosing. VieNeu does neither: it reads at its own pace,
    /// and the sheet does not offer what the engine would ignore.
    pub fn reads_at_a_pace(&self) -> bool {
        false
    }

    /// The rate the request carries: the slider's, or 1 for a model that
    /// reads at its own.
    fn rate(&self) -> f32 {
        if self.reads_at_a_pace() {
            self.speed
        } else {
            1.0
        }
    }

    /// The id under which the chosen model reads in a recording of the
    /// sheet's choosing. VieNeu reads only with its own speakers, so
    /// there is none: no clone row, no sample section.
    fn sample_voice(&self) -> Option<i32> {
        None
    }

    /// The recordings a voice can be taken from: everything in the bin with
    /// sound in it, in the bin's order.
    pub fn samples<'a>(&self, studio: &'a Studio) -> Vec<&'a MediaItem> {
        studio
            .project()
            .media
            .iter()
            .filter(|item| item.has_audio)
            .collect()
    }

    /// The chosen sample.
    fn sample_media<'a>(&self, studio: &'a Studio) -> Option<&'a MediaItem> {
        self.samples(studio).get(self.sample).copied()
    }

    /// The row to open the sample list on: the selected clip's recording,
    /// when one clip with sound in it is selected, else the first.
    fn default_sample(&self, studio: &Studio) -> usize {
        studio
            .sole_selection()
            .and_then(|id| studio.clip(&id))
            .filter(|clip| clip.kind == ClipKind::Video || clip.kind == ClipKind::Audio)
            .and_then(|clip| {
                self.samples(studio)
                    .iter()
                    .position(|item| item.id == clip.media_id)
            })
            .unwrap_or(0)
    }

    /// How far into a recording of `duration` the voice is taken from: what
    /// the sheet asked for, held short of the end so there is always at
    /// least a second of sound left to listen to.
    fn reference_offset(&self, duration: f64) -> f64 {
        self.reference_start.clamp(0.0, (duration - 1.0).max(0.0))
    }

    /// The voices the chosen model reads with, in table order.
    pub fn offered(&self) -> Vec<&VoiceInfo> {
        // The family's clone id is not a voice to pick from a list: it is
        // what a sample is sent under, and the sample switch offers it.
        self.speakers
            .iter()
            .filter(|speaker| speaker.family == self.family)
            .collect()
    }

    /// The row of `wanted` among the offered voices, or of the family's
    /// default, or the first.
    fn voice_row(&self, wanted: Option<i32>) -> usize {
        let offered = self.offered();
        let default = DEFAULT_VIENEU_VOICE;
        wanted
            .and_then(|id| offered.iter().position(|voice| voice.id == id))
            .or_else(|| offered.iter().position(|voice| voice.id == default))
            .unwrap_or(0)
    }

    /// How many saved voices the voice list offers: none, since no
    /// model reads in one any more.
    fn saved_offered(&self) -> usize {
        if self.clone_row() {
            self.saved.len()
        } else {
            0
        }
    }

    /// The saved voices the list offers.
    fn saved_rows(&self) -> &[SavedVoice] {
        &self.saved[..self.saved_offered()]
    }

    /// The voice list's rows: each offered voice's name. The saved
    /// voices and the clone row never join it now.
    pub fn speaker_rows(&self) -> Vec<SharedString> {
        self.offered()
            .iter()
            .map(|speaker| voice_label(&speaker.name).0.into())
            .chain(
                self.saved_rows()
                    .iter()
                    .map(|voice| voice.name.as_str().into()),
            )
            .chain(self.clone_row().then(|| t("speech.cloneFromSample").into()))
            .collect()
    }

    /// The voice list's second lines: each voice's gender, region and
    /// style, from the model's table.
    pub fn speaker_detail_rows(&self) -> Vec<SharedString> {
        self.offered()
            .iter()
            .map(|speaker| voice_label(&speaker.name).1.into())
            .chain(
                self.saved_rows()
                    .iter()
                    .map(|_| t("speech.savedVoice").into()),
            )
            .chain(
                self.clone_row()
                    .then(|| t("speech.fromRecordingInBin").into()),
            )
            .collect()
    }

    /// The voice list's avatars: a voice's picture where it has one, else
    /// the first letter of its name in the colour its name picks.
    pub fn speaker_avatar_rows(&self) -> Vec<crate::ui::AvatarData> {
        self.offered()
            .iter()
            .map(|speaker| {
                let name = voice_label(&speaker.name).0;
                let picture = speaker
                    .avatar
                    .as_deref()
                    .and_then(|path| slint::Image::load_from_path(path.as_ref()).ok());
                crate::ui::AvatarData {
                    letter: avatar_letter(&name).into(),
                    hue: avatar_hue(&name),
                    pictured: picture.is_some(),
                    picture: picture.unwrap_or_default(),
                }
            })
            .chain(
                self.saved_rows()
                    .iter()
                    .map(|voice| self.saved_avatar(voice)),
            )
            .chain(self.clone_row().then(|| crate::ui::AvatarData {
                letter: "+".into(),
                hue: -1,
                ..Default::default()
            }))
            .collect()
    }

    /// Whether the voice list ends in "Clone from a sample…": never
    /// now - no model reads in a recording.
    fn clone_row(&self) -> bool {
        self.sample_voice().is_some()
    }

    /// The model list's second lines: what each model is for, in a few
    /// words, since which one is chosen decides everything under it.
    pub fn model_detail_rows(&self, studio: &Studio) -> Vec<SharedString> {
        installed(&studio.settings.voices)
            .iter()
            .map(|_| t("speech.vieneuVoicesBuiltIn"))
            .map(SharedString::from)
            .collect()
    }

    /// The sample list's rows: each recording's name.
    pub fn sample_rows(&self, studio: &Studio) -> Vec<SharedString> {
        self.samples(studio)
            .iter()
            .map(|item| item.name.as_str().into())
            .collect()
    }

    /// The sample list's pictures: each recording's envelope as path
    /// commands, the same drawing the bin's card and the timeline's clip
    /// make of it, or empty while the peaks have not been read yet.
    pub fn sample_wave_rows(&self, studio: &Studio) -> Vec<SharedString> {
        self.samples(studio)
            .iter()
            .map(|item| match studio.peaks.get(&item.id) {
                Some(peaks) => {
                    wave_path(peaks, 0.0, item.duration.unwrap_or(0.0) as f32, 64, 0.75).into()
                }
                None => SharedString::new(),
            })
            .collect()
    }

    /// The sample list's second column: how long each recording is.
    pub fn sample_detail_rows(&self, studio: &Studio) -> Vec<SharedString> {
        self.samples(studio)
            .iter()
            .map(|item| {
                let whole = item.duration.unwrap_or(0.0).round() as i64;
                format!("{}:{:02}", whole / 60, whole % 60).into()
            })
            .collect()
    }

    /// The sheet as Slint shows it.
    pub fn data(&self, studio: &Studio) -> SpeechSheetData {
        let sample = self.sample_media(studio);
        let length = sample.and_then(|item| item.duration).unwrap_or(0.0);
        let named = self.offered().len();
        SpeechSheetData {
            open: self.open,
            text: self.text.as_str().into(),
            voice: self
                .saved_voice
                .filter(|row| *row < self.saved_offered())
                .map_or(self.voice, |row| named + row) as i32,
            voice_count: (named + self.saved_offered()) as i32,
            style: self
                .style
                .and_then(|row| STYLES.get(row))
                .map(|style| (style.name)())
                .unwrap_or_default()
                .into(),
            save_name: self.save_name.as_str().into(),
            save_picture: self
                .save_picture
                .as_ref()
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default()
                .into(),
            saving: self.saving,
            model: self.model as i32,
            // VieNeu, the only family left: the sheet's row 4.
            family: 4,
            speed: self.speed,
            quality: self.quality as i32,
            pauses: self.pauses,
            samples_offered: self.sample_voice().is_some(),
            use_sample: self.use_sample,
            sample: self.sample as i32,
            sample_name: sample
                .map(|item| item.name.as_str())
                .unwrap_or_default()
                .into(),
            reference_length: length as f32,
            reference_start: self.reference_offset(length) as f32,
            running: self.running,
            previewing: self.previewing,
            progress: self.pace.floor,
            ceiling: self.pace.ceiling,
            step: self.pace.step,
            left: self.pace.left,
            stage: match self.pace.stage() {
                Some(Stage::Reading) => t("speech.readingTheSample"),
                Some(Stage::Working) => t("speech.generatingVoice"),
                _ => t("speech.loadingTheModel"),
            }
            .into(),
            ready: !installed(&studio.settings.voices).is_empty(),
            estimate: self.estimate().into(),
            message: self.message.as_str().into(),
        }
    }
}

/// `tag` written into `text` at byte offset `at` - the caret, as the script
/// box reports it - with a space on whichever side is against a word, so
/// `[laugh]` lands as a word of its own and never as the tail of one. An
/// offset inside a character steps back to the character's start; one past
/// the end is the end. A trailing space is always written when the tag is
/// not already followed by one, so the caller can put the caret after it.
pub fn insert_tag(text: &str, tag: &str, at: usize) -> String {
    let mut at = at.min(text.len());
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    let (before, after) = text.split_at(at);
    let lead = if before.is_empty() || before.ends_with(char::is_whitespace) {
        ""
    } else {
        " "
    };
    let trail = if after.starts_with(char::is_whitespace) {
        ""
    } else {
        " "
    };
    format!("{before}{lead}{tag}{trail}{after}")
}

impl SpeechPane {
    /// The sheet, closed, with the voices saved on this machine.
    pub fn with_saved(dirs: &concat_host::AppDirs) -> SpeechPane {
        SpeechPane {
            saved: voices::list(&voices::dir_in(dirs)),
            ..SpeechPane::default()
        }
    }

    /// The saved voices, read from disk again.
    pub fn reload_saved(&mut self, studio: &Studio) {
        self.saved = voices::list(&voices::dir_in(&studio.host.dirs));
        self.pictures.borrow_mut().clear();
        if self.saved_voice.is_some_and(|row| row >= self.saved.len()) {
            self.saved_voice = None;
        }
    }

    /// The sheet, just opened, on what the Audio page's card names.
    fn choose(&mut self, choice: VoiceChoice, studio: &Studio) {
        let models = installed(&studio.settings.voices);
        let pick_model = |pane: &mut SpeechPane, row: usize| {
            pane.model = row;
            pane.family = models
                .get(row)
                .map(|model| family_of(&model.id))
                .unwrap_or(Family::VieNeu);
        };
        match choice {
            VoiceChoice::Builtin(id) => {
                let Some(family) = voice_family(id) else {
                    return;
                };
                if self.family != family {
                    match models
                        .iter()
                        .position(|model| family_of(&model.id) == family)
                    {
                        Some(row) => pick_model(self, row),
                        None => {
                            self.message = t("speech.downloadModelForVoice");
                            return;
                        }
                    }
                }
                self.voice = self.voice_row(Some(id));
            }
            VoiceChoice::Style(row) => self.style = (row < STYLES.len()).then_some(row),
        }
    }

    /// Keeps the sample being read in as a voice of the user's own, on a
    /// worker: a few seconds of it from where the sheet listens.
    fn save_voice(&mut self, studio: &mut Studio) {
        let name = self.save_name.trim().to_owned();
        if name.is_empty() || self.saving {
            return;
        }
        let Some(sample) = self.sample_media(studio).cloned() else {
            self.message = t("speech.pickSampleVoice");
            return;
        };
        let start = self.reference_offset(sample.duration.unwrap_or(0.0));
        let picture = self.save_picture.clone();
        let dir = voices::dir_in(&studio.host.dirs);
        self.saving = true;
        self.message.clear();
        crate::host::spawn(
            move || {
                voices::save(
                    &dir,
                    &name,
                    std::path::Path::new(&sample.path),
                    start,
                    picture.as_deref(),
                )
            },
            |studio, _, _, result| {
                studio.handle(Msg::Speech(SpeechMsg::VoiceSaved(Box::new(result))));
            },
        );
    }
}

impl SpeechPane {
    /// A saved voice's avatar: its picture, or its initial in its colour.
    pub fn saved_avatar(&self, voice: &SavedVoice) -> crate::ui::AvatarData {
        let picture = voice
            .picture_path()
            .map(|path| {
                self.pictures
                    .borrow_mut()
                    .entry(voice.id.clone())
                    .or_insert_with(|| slint::Image::load_from_path(&path).unwrap_or_default())
                    .clone()
            })
            .filter(|image| image.size().width > 0);
        crate::ui::AvatarData {
            letter: avatar_letter(&voice.name).into(),
            hue: avatar_hue(&voice.name),
            pictured: picture.is_some(),
            picture: picture.unwrap_or_default(),
        }
    }
}

/// A voice's avatar as the Audio page draws it: its initial in the colour
/// its name picks.
pub fn letter_avatar(name: &str) -> crate::ui::AvatarData {
    crate::ui::AvatarData {
        letter: avatar_letter(name).into(),
        hue: avatar_hue(name),
        ..Default::default()
    }
}

/// A voice's name and its second line, as the voice list says them.
pub fn voice_names(name: &str) -> (String, String) {
    voice_label(name)
}

/// A VieNeu voice as a person would say it: the name is the voice, and
/// the table says what it is.
fn voice_label(name: &str) -> (String, String) {
    if let Some(detail) = vieneu_voice_detail(name) {
        return (name.to_owned(), detail.to_owned());
    }
    (name.to_owned(), String::new())
}

/// The letter a voice's avatar shows: the first of its first name.
fn avatar_letter(name: &str) -> String {
    name.split_whitespace()
        .next()
        .and_then(|first| first.chars().next())
        .map(|letter| letter.to_uppercase().collect())
        .unwrap_or_default()
}

/// Which of the avatar palette's eight hues a voice's name picks: a hash
/// of the name (FNV-1a), so a voice keeps its colour from run to run.
fn avatar_hue(name: &str) -> i32 {
    let hash = name
        .to_lowercase()
        .bytes()
        .fold(0x811c_9dc5_u32, |hash, byte| {
            (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
        });
    (hash % 8) as i32
}

#[cfg(test)]
mod tests {
    use super::{avatar_hue, avatar_letter, insert_tag, voice_label};

    /// An avatar is the first letter of the first name, in a colour the
    /// name settles: the same every time, one of the palette's eight.
    #[test]
    fn a_voice_avatar_is_its_initial_in_its_own_colour() {
        assert_eq!(avatar_letter("Bria"), "B");
        assert_eq!(avatar_letter("émile dupont"), "É");
        assert_eq!(avatar_letter(""), "");
        assert_eq!(avatar_hue("Bria"), avatar_hue("bria"));
        let hues: std::collections::HashSet<i32> = [
            "Bria", "Adam", "Heart", "Lewis", "Nova", "Sky", "Eric", "Alba",
        ]
        .iter()
        .map(|name| avatar_hue(name))
        .collect();
        assert!(hues.iter().all(|hue| (0..8).contains(hue)));
        assert!(hues.len() > 2, "names spread over the palette: {hues:?}");
    }

    /// A tag lands as a word of its own wherever the caret is: spaced off a
    /// word on either side, not doubled against a space already there, and
    /// snapped to a character when the caret is reported inside one.
    #[test]
    fn a_tag_lands_as_its_own_word() {
        assert_eq!(insert_tag("", "[laugh]", 0), "[laugh] ");
        assert_eq!(insert_tag("hello", "[laugh]", 5), "hello [laugh] ");
        assert_eq!(insert_tag("hello world", "[sigh]", 5), "hello [sigh] world");
        assert_eq!(insert_tag("hello world", "[sigh]", 6), "hello [sigh] world");
        assert_eq!(
            insert_tag("hello world", "[sigh]", 3),
            "hel [sigh] lo world"
        );
        assert_eq!(insert_tag("hi", "[chuckle]", 0), "[chuckle] hi");
        // Past the end is the end; inside "é" (two bytes) steps back to it.
        assert_eq!(insert_tag("hi", "[laugh]", 40), "hi [laugh] ");
        assert_eq!(insert_tag("café", "[laugh]", 4), "caf [laugh] é");
        assert_eq!(insert_tag("café ", "[laugh]", 6), "café [laugh] ");
    }

    #[test]
    fn a_voice_name_reads_as_a_person_would_say_it() {
        let _english = crate::i18n::language_lock();
        let (title, detail) = voice_label("Hải Đăng");
        assert_eq!(title, "Hải Đăng");
        assert!(detail.contains("Nam"));
        assert!(detail.contains("Bắc"));
        assert_eq!(voice_label("Thiện Minh").1, "Nam · Bắc · kể chuyện");
        assert_eq!(voice_label("unknown").0, "unknown");
        assert_eq!(voice_label("unknown").1, "");
    }
}
