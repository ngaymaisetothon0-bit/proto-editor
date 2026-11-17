// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The captions sheet: the tray's Captions tool, as a form and then as a
//! progress report.
//!
//! One switch says what it captions: "Use Sample Voice", on, takes
//! the words from a clip with sound, chosen in the sheet from the clips on
//! the timeline that have any; off, from a script typed in it. It opens on
//! when a clip with sound is selected, or when a clip's menu opened it on
//! that clip, and off otherwise. A voice run lands as one title clip per
//! cue, after the sheet shows the cues for a review; the whole run is
//! still one undo step, since every command names one gesture.
//! The transcriber runs on a worker and reports as
//! [`CaptionsMsg::Progress`], ending as [`CaptionsMsg::Finished`].
//!
//! The sheet also reads a SubRip file in as captions, with the same look,
//! and writes the timeline's captions out as one.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use concat_project::Command;
use concat_project::commands::ClipPatch;
use concat_project::model::{AppliedFilter, ClipKind, KeyEase, KeyProperty, TextStyle};
use concat_project::srt;
use concat_speech::JobStage;
use concat_speech::cues::words_to_cues;
use concat_speech::transcribe::{Segment, Word};

use crate::format::{colour_of, hex_of, parse_colour};
use crate::host::{on_ui_in_project, spawn_in_project};
use crate::i18n::{t, tf};
use crate::panes::Msg;
use crate::panes::pace::{Pace, Stage, Timings};
use crate::panes::settings::installed;
use crate::studio::Studio;
use crate::ui::{CaptionsSheetData, TranscriptLineData};

/// Where a caption sits, by the sheet's row: a frame-height fraction from
/// the centre, positive down. Bottom, centre, top.
const CAPTION_OFFSETS: [f64; 3] = [0.35, 0.0, -0.35];
/// A caption's cap height by the sheet's row, as a fraction of the frame.
const CAPTION_SIZES: [f64; 3] = [0.04, 0.05, 0.065];
/// A rough speaking rate, for a script's timing and the speech sheet's
/// estimate.
pub const CHARS_PER_SECOND: f32 = 14.0;

/// How long a cue clip's pop lasts, in timeline seconds: the first
/// half-second of the cue, or the whole cue when it is shorter.
const POP_SECONDS: f64 = 0.5;
/// The gesture every command of one AutoSub run names, so the clips and
/// their pop keys fold into a single undo step (see
/// [`Studio::apply_in_gesture`]).
const AUTOSUB_GESTURE: &str = "autosub-captions";

/// The karaoke highlight effect, attached to every cue clip that has
/// word timings - see `karaoke_filter`.
const KARAOKE_EFFECT: &str = "concat.karaoke-highlight";

/// The highlight colour a cue's words get in step 4's karaoke: gold, the
/// same gold the Gold title preset uses. Stored from step 3; the clips do
/// not use it yet, since one clip holds one colour today.
const DEFAULT_HIGHLIGHT: &str = "#ffd60a";

/// The faces the cue font picker offers: the bundled faces that cover
/// Vietnamese, in the order the picker shows them. (Hanken Grotesk's
/// Regular, SemiBold and Bold were checked against the Vietnamese
/// precomposed letters and combining marks: all present. Fraunces Bold
/// lacks only the combining hook U+0309 and dot below U+0323, which
/// precomposed Vietnamese text never needs.) The cue's own family joins
/// at the end when it is none of these - a project font, or a title look
/// set elsewhere - so the picker never loses the look a run started in.
const CUE_FONTS: [&str; 4] = ["Be Vietnam Pro", "Fraunces", "Hanken Grotesk", "Inter"];

/// Everything that can happen to the captions sheet.
#[derive(Clone, Debug)]
pub enum CaptionsMsg {
    /// The tray's Captions tool: the selected clip's voice when it has
    /// sound, the script otherwise.
    Open,
    /// A clip's menu: the sheet on that clip's voice.
    OpenOnClip(String),
    Close,
    /// The "Use Sample Voice" switch.
    VoiceToggled(bool),
    /// Row in the list of clips with sound.
    SourceChanged(i32),
    TextEdited(String),
    ModelChanged(i32),
    PlacementChanged(i32),
    SizeChanged(i32),
    ModeChanged(i32),
    TranscriptLineToggled(i32),
    ApplyTranscriptEdit,
    /// A reviewed run of cues as title clips, each with its pop keys.
    ApplyCues,
    /// A cue row clicked: select it, and jump the preview to it.
    CueSelected(i32),
    /// The selected cue's text, edited in the field under the table.
    CueTextEdited(String),
    /// The font picker's row, into `cue_font_options`.
    FontChanged(i32),
    /// A text-colour swatch.
    TextColorChanged(slint::Color),
    /// A highlight-colour swatch.
    HighlightColorChanged(slint::Color),
    /// The animation row: 0 Pop, 1 Fade, 2 Slide, 3 None.
    AnimationChanged(i32),
    ClearTranscript,
    /// Run the pass the sheet describes.
    Begin,
    Cancel,
    /// The transcriber's worker reporting the stage it is in.
    Progress(JobStage),
    /// The transcriber's worker is done: what was said, or why not.
    Finished(Result<Vec<Segment>, String>),
    /// Asks for a SubRip file to read in.
    ImportSrt,
    /// The SubRip file chosen: its cues as captions.
    ImportSrtFrom(PathBuf),
    /// Asks where to write the captions as SubRip.
    ExportSrt,
    /// Where they go.
    ExportSrtTo(PathBuf),
}

/// The clip a transcription is running over, as the finished segments
/// need it: where it starts on the timeline and how fast it plays.
#[derive(Clone, Debug)]
struct Subject {
    clip_id: String,
    media_id: String,
    track_id: String,
    source_start: f64,
    duration: f64,
    start: f64,
    speed: f64,
    /// The look chosen when the run began.
    look: (f64, f64),
    /// The transcriber model, and the thirty-second windows whisper reads
    /// the sound in: what the run's timing is kept by.
    model: String,
    windows: f32,
}

/// Seconds whisper takes over one thirty-second window, per megabyte of
/// model, before a run on the model has been timed: about a second for
/// base and ten for medium on eight cores of a recent CPU.
const WINDOW_SECONDS_PER_MB: f32 = 1.0 / 160.0;

/// The captions sheet's state.
#[derive(Default)]
pub struct CaptionsPane {
    pub open: bool,
    /// "Use Sample Voice": the words come from [`Self::clip`]'s sound
    /// when on, from [`Self::text`] when off.
    pub from_voice: bool,
    /// The clip whose voice is transcribed: one with sound, chosen from the
    /// timeline's. Kept while the switch is off, so turning it back on
    /// returns to the same clip. The panel floats over a live editor, so
    /// a clip with sound selected since is the one shown and run instead;
    /// see [`Self::subject_clip`].
    pub clip: Option<String>,
    /// The sole selection when [`Self::clip`] was chosen - opened on, or
    /// picked from the list - so a later selection can be told from it.
    chosen_with: Option<String>,
    /// The script's words.
    pub text: String,
    /// Row in the installed transcriber list.
    pub model: usize,
    /// 0 bottom, 1 centre, 2 top.
    pub placement: usize,
    /// 0 small, 1 medium, 2 large.
    pub size: usize,
    /// 0 creates subtitles; 1 removes selected phrases from a video clip.
    pub mode: usize,
    transcript: Vec<TranscriptLine>,
    transcript_subject: Option<Subject>,
    /// The cue the review table has selected, for the edit field and the
    /// preview jump; None when none is.
    selected_cue: Option<usize>,
    /// The look the cue clips are set in: the base title look's family
    /// and colour at the run's start, then the pickers' choices.
    cue_font: String,
    cue_color: String,
    /// The karaoke highlight, for step 4; gold until the picker says
    /// otherwise. It stays across runs - there is no base look to reset
    /// it from, the way the font and the colour have one.
    highlight_color: String,
    /// The cue entrance, by the animation row: 0 Pop, 1 Fade, 2 Slide,
    /// 3 None. Like the highlight, it stays across runs.
    animation: usize,
    pub running: bool,
    /// How far the running transcription looks.
    pub pace: Pace,
    /// Why the last run failed, when it did.
    pub message: String,
    /// The clip under the running transcription.
    subject: Option<Subject>,
    /// How long each model took this session, for the next run's bar.
    timings: Timings,
}

#[derive(Clone, Debug, PartialEq)]
struct TranscriptLine {
    start: f64,
    end: f64,
    text: String,
    cut: bool,
    /// The words the line was cut from, with their timings - what the
    /// karaoke highlight reads. Empty for a script line and for a whole
    /// segment kept as one line, which get no karaoke effect.
    words: Vec<Word>,
}

impl CaptionsPane {
    /// Applies one message. The studio is the rest of the window; while
    /// this runs the studio's copy of the pane is a blank it must not read.
    pub fn update(&mut self, msg: CaptionsMsg, studio: &mut Studio) {
        match msg {
            // A run in progress is shown as it is, however the panel is
            // asked for again: closing it never stopped the run.
            CaptionsMsg::Open | CaptionsMsg::OpenOnClip(_) if self.running => self.open = true,
            CaptionsMsg::Open => {
                let selected = studio.sole_selection();
                self.open_on(studio, selected, false);
            }
            CaptionsMsg::OpenOnClip(id) => self.open_on(studio, Some(id), true),
            CaptionsMsg::Close => self.open = false,
            CaptionsMsg::VoiceToggled(on) => {
                self.from_voice = on;
                self.message.clear();
                let sources = sources(studio);
                if on
                    && !self
                        .clip
                        .as_ref()
                        .is_some_and(|id| sources.iter().any(|s| &s.id == id))
                {
                    self.clip = sources.first().map(|source| source.id.clone());
                    self.chosen_with = studio.sole_selection();
                }
            }
            CaptionsMsg::SourceChanged(index) => {
                if let Some(source) = usize::try_from(index)
                    .ok()
                    .and_then(|index| sources(studio).into_iter().nth(index))
                {
                    self.clip = Some(source.id);
                    self.chosen_with = studio.sole_selection();
                    self.message.clear();
                }
            }
            CaptionsMsg::TextEdited(text) => self.text = text,
            CaptionsMsg::ModelChanged(index) => self.model = index.max(0) as usize,
            CaptionsMsg::PlacementChanged(index) => {
                self.placement = (index.max(0) as usize).min(2);
            }
            CaptionsMsg::SizeChanged(index) => self.size = (index.max(0) as usize).min(2),
            CaptionsMsg::ModeChanged(index) => {
                self.mode = (index.max(0) as usize).min(1);
                self.from_voice = self.mode == 1;
                self.transcript.clear();
                self.transcript_subject = None;
                self.selected_cue = None;
                self.message.clear();
            }
            CaptionsMsg::TranscriptLineToggled(index) => {
                if let Some(line) = usize::try_from(index)
                    .ok()
                    .and_then(|index| self.transcript.get_mut(index))
                {
                    line.cut = !line.cut;
                }
            }
            CaptionsMsg::ApplyTranscriptEdit => self.apply_transcript_edit(studio),
            CaptionsMsg::ApplyCues => self.apply_cues(studio),
            CaptionsMsg::CueSelected(index) => self.select_cue(index, studio),
            CaptionsMsg::CueTextEdited(text) => {
                if let Some(line) = self
                    .selected_cue
                    .and_then(|index| self.transcript.get_mut(index))
                {
                    line.text = text;
                }
            }
            CaptionsMsg::FontChanged(index) => {
                if let Some(family) = cue_font_options(&self.cue_font)
                    .into_iter()
                    .nth(usize::try_from(index).unwrap_or(usize::MAX))
                {
                    self.cue_font = family;
                }
            }
            CaptionsMsg::TextColorChanged(colour) => self.cue_color = hex_of(colour),
            CaptionsMsg::HighlightColorChanged(colour) => {
                self.highlight_color = hex_of(colour);
            }
            CaptionsMsg::AnimationChanged(index) => {
                self.animation = (index.max(0) as usize).min(3);
            }
            CaptionsMsg::ClearTranscript => {
                self.transcript.clear();
                self.transcript_subject = None;
                self.selected_cue = None;
                self.message.clear();
            }
            CaptionsMsg::Begin => {
                if self.from_voice {
                    self.run_sound(studio);
                } else {
                    self.run_script(studio);
                }
            }
            CaptionsMsg::Cancel => {
                studio.host.transcriber.cancel();
                self.running = false;
                self.open = false;
            }
            CaptionsMsg::Progress(stage) => self.pace.report(stage),
            CaptionsMsg::Finished(result) => {
                self.running = false;
                let subject = self.subject.take();
                if let (Ok(_), Some(subject)) = (&result, &subject) {
                    self.timings
                        .keep(&subject.model, self.pace.learned(subject.windows));
                }
                match result {
                    Ok(segments) if self.mode == 1 => {
                        let Some(subject) = subject else {
                            return;
                        };
                        self.transcript = segments
                            .into_iter()
                            .filter_map(|segment| {
                                let text = segment.text.trim().to_owned();
                                (!text.is_empty() && segment.end > segment.start).then_some(
                                    TranscriptLine {
                                        start: segment.start,
                                        end: segment.end,
                                        text,
                                        cut: false,
                                        words: Vec::new(),
                                    },
                                )
                            })
                            .collect();
                        self.transcript_subject = Some(subject);
                        self.open = true;
                        if self.transcript.is_empty() {
                            self.message = t("captions.nothingSaidClip");
                        }
                    }
                    Ok(segments) => {
                        // A voice run only reaches here: a script applies its
                        // own lines in `run_script`, never through `Finished`.
                        // The cues are shown for a review first; `ApplyCues`
                        // puts them on the timeline.
                        let Some(subject) = subject else {
                            return;
                        };
                        self.transcript = cues_of_segments(&segments);
                        self.transcript_subject = Some(subject);
                        // The look the cues start in: the base title
                        // look's family and colour, or the caption
                        // defaults when there is none. The highlight and
                        // the animation stay as the last run left them.
                        let base = studio.prefs.title_style.clone();
                        self.cue_font = base
                            .as_ref()
                            .map(|style| style.font_family.clone())
                            .unwrap_or_else(|| "Hanken Grotesk".to_owned());
                        self.cue_color = base
                            .as_ref()
                            .map(|style| style.color.clone())
                            .unwrap_or_else(|| "#ffffff".to_owned());
                        if self.highlight_color.is_empty() {
                            self.highlight_color = DEFAULT_HIGHLIGHT.to_owned();
                        }
                        self.selected_cue = None;
                        self.open = true;
                        if self.transcript.is_empty() {
                            self.message = t("captions.nothingSaidClip");
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
            // Asked with the studio unborrowed, on a turn of its own; see
            // `host::after_dialog`.
            CaptionsMsg::ImportSrt => crate::host::pick_files_then(
                t("captions.importSrt"),
                Some((t("captions.subRipSubtitles"), &["srt"])),
                |studio, _, _, paths| {
                    if let Some(path) = paths.into_iter().next() {
                        studio.handle(Msg::Captions(CaptionsMsg::ImportSrtFrom(path)));
                    }
                },
            ),
            CaptionsMsg::ImportSrtFrom(path) => self.import_srt(studio, &path),
            CaptionsMsg::ExportSrt => self.ask_export_srt(studio),
            CaptionsMsg::ExportSrtTo(path) => self.export_srt(studio, &path),
        }
    }

    /// A SubRip file's cues as captions where the file times them, with
    /// the sheet's look: one batch, one undo step. A file that is not
    /// UTF-8 is read as best it can be.
    fn import_srt(&mut self, studio: &mut Studio, path: &Path) {
        let cues = std::fs::read(path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                srt::parse(&String::from_utf8_lossy(&bytes)).map_err(|error| error.to_string())
            });
        let cues = match cues {
            Ok(cues) => cues,
            Err(error) => {
                log::warn!("captions: could not read {}: {error}", path.display());
                self.message = tf("captions.couldNotReadSubtitles", &[&error]);
                return;
            }
        };
        let look = self.look();
        let base = studio.prefs.title_style.clone();
        let commands = srt_commands(cues, look, base.as_ref());
        if commands.is_empty() {
            self.message = t("captions.noSubtitlesInFile");
            return;
        }
        let count = commands.len();
        self.open = false;
        studio.apply(Command::Batch { commands });
        studio.notify(&tf("captions.addedCaptions", &[&count]), false);
    }

    /// Asks where to write the captions, offering the project's folder
    /// and its name. A phone has no such dialog: the file is written
    /// beside the project under a name nothing there has, and handed to
    /// the phone to put where its file manager shows it.
    fn ask_export_srt(&mut self, studio: &mut Studio) {
        if export_lane(studio).is_none() {
            self.message = t("captions.noCaptionsToExport");
            return;
        }
        let name = format!("{}.srt", file_stem(studio));
        // No project, no project folder: Documents, as anywhere else a
        // file is written with nothing better to offer.
        let folder = studio
            .session
            .as_ref()
            .map(|session| PathBuf::from(session.path()))
            .unwrap_or_else(|| PathBuf::from(crate::studio::home_folder("Documents")));
        if cfg!(any(target_os = "android", target_os = "ios")) {
            let path = unique_path(&folder, &name);
            self.export_srt(studio, &path);
            return;
        }
        let title = t("captions.exportSrt");
        let family = t("captions.subRipSubtitles");
        crate::host::after_dialog(
            move || {
                let chosen = crate::platform::save_file(
                    &title,
                    &folder,
                    &name,
                    (family.as_str(), &["srt"]),
                )?;
                Some(
                    if chosen
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("srt"))
                    {
                        chosen
                    } else {
                        chosen.with_extension("srt")
                    },
                )
            },
            |studio, _, _, path| studio.handle(Msg::Captions(CaptionsMsg::ExportSrtTo(path))),
        );
    }

    /// Writes the captions to `path` as SubRip, and says how many and
    /// where - on a phone, once the phone has put the file where it shows
    /// it.
    fn export_srt(&mut self, studio: &mut Studio, path: &Path) {
        let Some(lane) = export_lane(studio) else {
            self.message = t("captions.noCaptionsToExport");
            return;
        };
        let cues = srt::title_cues(studio.timeline(), Some(&lane.track_id));
        if let Err(error) = std::fs::write(path, srt::write(&cues)) {
            log::warn!("captions: could not write {}: {error}", path.display());
            self.message = tf("captions.couldNotWriteSubtitles", &[&error]);
            return;
        }
        log::info!(
            "captions: wrote {} cue(s) to {}",
            cues.len(),
            path.display()
        );
        self.open = false;
        let count = cues.len();
        // Which lane was written goes without saying when it was the
        // selected title's, or the only lane with titles; otherwise the
        // notice says it was the busiest, and how to pick another.
        let busiest = lane.busiest && srt::title_lanes(studio.timeline()) > 1;
        let say = move |studio: &mut Studio, place: &str| {
            // Each key spelled out where it is used: scripts/locales.py
            // finds the strings the source asks for by reading it.
            let said = if busiest {
                tf("captions.exportedBusiestLane", &[&count, &place])
            } else {
                tf("captions.exportedCaptions", &[&count, &place])
            };
            studio.notify(&said, false);
        };
        let written = path.to_path_buf();
        let handed = crate::platform::publish_document(
            path.to_path_buf(),
            "application/x-subrip",
            move |result| {
                crate::host::on_ui(move |studio, _, _| match result {
                    Ok(place) => say(studio, &place),
                    Err(error) => {
                        log::warn!("captions: could not publish {}: {error}", written.display());
                        studio.notify(
                            &tf(
                                "captions.couldNotPublishSubtitles",
                                &[&written.display(), &error],
                            ),
                            true,
                        );
                    }
                });
            },
        );
        if !handed {
            say(studio, &path.display().to_string());
        }
    }

    /// The sheet opened afresh. `clip` is the clip it was opened on, if
    /// any: the switch starts on when that clip has sound, or when a clip's
    /// menu asked for its voice (`voice`), in which case a clip without
    /// sound falls to the first clip that has some.
    fn open_on(&mut self, studio: &Studio, clip: Option<String>, voice: bool) {
        let installed = installed(&studio.settings.transcribers);
        let model = installed.iter().position(|model| model.active).unwrap_or(0);
        let sources = sources(studio);
        let sounding = clip.filter(|id| sources.iter().any(|source| &source.id == id));
        let from_voice = voice || sounding.is_some();
        let clip = sounding.or_else(|| {
            from_voice
                .then(|| sources.first().map(|source| source.id.clone()))
                .flatten()
        });
        *self = CaptionsPane {
            open: true,
            from_voice,
            clip,
            chosen_with: studio.sole_selection(),
            model,
            placement: 0,
            size: 1,
            timings: std::mem::take(&mut self.timings),
            ..CaptionsPane::default()
        };
    }

    /// The clip the panel transcribes, and the one its list shows - the
    /// same answer for both, so it never runs a clip it is not showing.
    /// The editor stays live under the panel: a clip with sound selected
    /// since the panel's clip was chosen is the one meant now, and is
    /// taken. Otherwise the chosen clip while it is still on the timeline
    /// with sound, and none - nothing to run - when it is not.
    fn subject_clip(&self, studio: &Studio, sources: &[Source]) -> Option<String> {
        follow_selection(
            studio.sole_selection(),
            self.chosen_with.as_deref(),
            self.clip.as_deref(),
            |id| sources.iter().any(|source| source.id == id),
        )
    }

    /// A caption's look, by the sheet's rows: where it sits and its size.
    fn look(&self) -> (f64, f64) {
        (
            CAPTION_OFFSETS[self.placement.min(2)],
            CAPTION_SIZES[self.size.min(2)],
        )
    }

    /// The script as titles, one after another from the playhead.
    fn run_script(&mut self, studio: &mut Studio) {
        let lines = script_captions(&self.text);
        if lines.is_empty() {
            self.message = t("captions.nothingToCaptionYet");
            return;
        }
        let look = self.look();
        let mut at = f64::from(studio.playhead);
        let base = studio.prefs.title_style.clone();
        let commands: Vec<Command> = lines
            .into_iter()
            .map(|(text, seconds)| {
                let command = caption_clip(text, at, seconds, look, base.as_ref());
                at += seconds;
                command
            })
            .collect();
        let count = commands.len();
        self.open = false;
        studio.apply(Command::Batch { commands });
        studio.notify(&tf("captions.addedCaptions", &[&count]), false);
    }

    /// The sheet's clip through the transcriber on a worker, reporting into
    /// the sheet as it goes.
    fn run_sound(&mut self, studio: &mut Studio) {
        // The clip the panel shows, and no other: see `subject_clip`.
        let Some(clip) = self
            .subject_clip(studio, &sources(studio))
            .and_then(|id| studio.clip(&id))
            .cloned()
        else {
            self.message = t("captions.clipNoLongerTimeline");
            return;
        };
        if self.mode == 1 {
            if clip.kind != ClipKind::Video {
                self.message = t("captions.transcriptVideoRequired");
                return;
            }
            if clip.speed_curve.is_some() {
                self.message = t("captions.transcriptSpeedCurveUnsupported");
                return;
            }
        }
        let Some(media) = studio.project().media_by_id(&clip.media_id).cloned() else {
            self.message = t("captions.clipHasNoFile");
            return;
        };
        let Some((model, megabytes)) = installed(&studio.settings.transcribers)
            .get(self.model)
            .map(|model| (model.id.clone(), model.megabytes))
        else {
            self.message = t("captions.downloadTranscriberModelFirst");
            return;
        };
        let request = concat_speech::transcribe::TranscribeRequest {
            path: media.path.clone(),
            audio_stream: clip.audio_stream,
            source_start: clip.source_start,
            window: clip.duration * clip.speed,
            model_id: model.clone(),
        };
        let dirs = studio.host.dirs.clone();
        let transcriber = Arc::clone(&studio.host.transcriber);
        // The run as the bar lays it out: the sound read in at a few
        // hundred times its length, the model loaded unless it still is,
        // and whisper a window at a time - at the pace the last run on
        // this model measured, or the estimate by its size.
        let windows = (request.window / 30.0).ceil().max(1.0) as f32;
        let learned = self.timings.get(&model);
        let loaded = studio.host.transcriber.loaded_model().as_deref() == Some(model.as_str());
        let load = if loaded {
            0.0
        } else {
            learned.load.unwrap_or(0.5 + megabytes / 400.0)
        };
        let per_window = learned
            .per_unit
            .unwrap_or(0.3 + megabytes * WINDOW_SECONDS_PER_MB);
        self.pace = Pace::new(vec![
            (Stage::Reading, 0.2 + request.window as f32 / 400.0),
            (Stage::Loading, load),
            (Stage::Working, windows * per_window),
        ]);
        self.subject = Some(Subject {
            clip_id: clip.id.clone(),
            media_id: clip.media_id.clone(),
            track_id: clip.track_id.clone(),
            source_start: clip.source_start,
            duration: clip.duration,
            start: clip.start,
            speed: clip.speed,
            look: self.look(),
            model,
            windows,
        });
        self.running = true;
        self.message.clear();
        let epoch = crate::host::project_epoch();
        spawn_in_project(
            move || {
                transcriber.transcribe(&dirs, &request, move |stage| {
                    on_ui_in_project(epoch, move |studio, _, _| {
                        studio.handle(Msg::Captions(CaptionsMsg::Progress(stage)));
                    });
                })
            },
            |studio, _, _, result| studio.handle(Msg::Captions(CaptionsMsg::Finished(result))),
        );
    }

    /// The sheet as Slint shows it.
    pub fn data(&self, studio: &Studio) -> CaptionsSheetData {
        let sources = if self.open {
            sources(studio)
        } else {
            Vec::new()
        };
        let source = self
            .subject_clip(studio, &sources)
            .and_then(|id| sources.iter().position(|source| source.id == id))
            .map_or(-1, |index| index as i32);
        let labels: Vec<slint::SharedString> = sources
            .into_iter()
            .map(|source| source.label.into())
            .collect();
        CaptionsSheetData {
            open: self.open,
            from_voice: self.from_voice,
            sources: std::rc::Rc::new(slint::VecModel::from(labels)).into(),
            source,
            text: self.text.as_str().into(),
            model: self.model as i32,
            placement: self.placement as i32,
            size: self.size as i32,
            running: self.running,
            progress: self.pace.floor,
            ceiling: self.pace.ceiling,
            step: self.pace.step,
            left: self.pace.left,
            stage: match self.pace.stage() {
                Some(Stage::Loading) => t("captions.loadingTheModel"),
                Some(Stage::Working) => t("captions.transcribing"),
                _ => t("captions.readingTheSound"),
            }
            .into(),
            ready: !installed(&studio.settings.transcribers).is_empty(),
            message: self.message.as_str().into(),
            mode: self.mode as i32,
            transcript: std::rc::Rc::new(slint::VecModel::from(
                self.transcript
                    .iter()
                    .enumerate()
                    .map(|(index, line)| TranscriptLineData {
                        index: index as i32,
                        time: crate::format::running_time(line.start).into(),
                        text: line.text.as_str().into(),
                        cut: line.cut,
                    })
                    .collect::<Vec<_>>(),
            ))
            .into(),
            transcript_ready: !self.transcript.is_empty() && self.mode == 1,
            cues_ready: !self.transcript.is_empty() && self.mode == 0,
            selected: self.selected_cue.map_or(-1, |index| index as i32),
            selected_text: self
                .selected_cue
                .and_then(|index| self.transcript.get(index))
                .map_or("", |line| line.text.as_str())
                .into(),
            font_options: std::rc::Rc::new(slint::VecModel::from(
                cue_font_options(&self.cue_font)
                    .into_iter()
                    .map(slint::SharedString::from)
                    .collect::<Vec<_>>(),
            ))
            .into(),
            font: cue_font_options(&self.cue_font)
                .iter()
                .position(|family| family == &self.cue_font)
                .map_or(0, |index| index as i32),
            text_color: colour_of(&self.cue_color),
            highlight_color: colour_of(&self.highlight_color),
            animation: self.animation as i32,
            cut_count: self.transcript.iter().filter(|line| line.cut).count() as i32,
        }
    }

    fn apply_transcript_edit(&mut self, studio: &mut Studio) {
        let Some(subject) = self.transcript_subject.as_ref() else {
            self.message = t("captions.transcriptOutdated");
            return;
        };
        let Some(clip) = studio.clip(&subject.clip_id) else {
            self.message = t("captions.transcriptOutdated");
            return;
        };
        if clip.media_id != subject.media_id
            || clip.track_id != subject.track_id
            || (clip.source_start - subject.source_start).abs() > 1e-6
            || (clip.duration - subject.duration).abs() > 1e-6
            || (clip.speed - subject.speed).abs() > 1e-9
            || clip.speed_curve.is_some()
        {
            self.message = t("captions.transcriptOutdated");
            return;
        }
        let ranges: Vec<(f64, f64)> = self
            .transcript
            .iter()
            .filter(|line| line.cut)
            .map(|line| {
                (
                    clip.start + line.start / clip.speed,
                    clip.start + line.end / clip.speed,
                )
            })
            .collect();
        let removed = studio.remove_transcript_ranges(&subject.clip_id, &ranges);
        if removed == 0 {
            self.message = t("captions.transcriptNoCuts");
            return;
        }
        self.transcript.clear();
        self.transcript_subject = None;
        self.open = false;
        self.message.clear();
        studio.notify(
            &tf("captions.transcriptCutDone", &[&removed]),
            false,
        );
    }

    /// A reviewed run of cues as title clips, each with its pop keys:
    /// one title per cue on the first free lane above the picture, in one
    /// gesture so the whole run is a single undo step. The clip's id comes
    /// back from its own add, which is why the adds and the keys cannot be
    /// one batch (a batch only reports its last minted id).
    fn apply_cues(&mut self, studio: &mut Studio) {
        let Some(subject) = self.live_cue_subject(studio) else {
            self.message = t("captions.transcriptOutdated");
            return;
        };
        let look = subject.look;
        // The look the cues are set in: the base title look with the
        // pickers' family and colour over it, or the caption defaults
        // with the same overrides when there is no base.
        let base = match studio.prefs.title_style.clone() {
            Some(mut style) => {
                style.font_family = self.cue_font.clone();
                style.color = self.cue_color.clone();
                style
            }
            None => TextStyle {
                font_family: self.cue_font.clone(),
                font_weight: 600.0,
                color: self.cue_color.clone(),
                ..TextStyle::default()
            },
        };
        let animation = self.animation;
        let mut count = 0;
        for line in &self.transcript {
            let start = subject.start + line.start / subject.speed;
            let duration = ((line.end - line.start) / subject.speed).max(0.2);
            let Some(id) = studio.apply_in_gesture(
                AUTOSUB_GESTURE,
                caption_clip(line.text.clone(), start, duration, look, Some(&base)),
            ) else {
                continue;
            };
            for key in animation_keys(animation, &id, duration) {
                studio.apply_in_gesture(AUTOSUB_GESTURE, key);
            }
            // The karaoke highlight rides on the same gesture: the clip
            // is fresh, so its effect list is empty and replacing it
            // with the one karaoke filter is the whole change.
            if let Some(filter) =
                karaoke_filter(&line.words, line.start, line.end, &self.highlight_color)
            {
                studio.apply_in_gesture(
                    AUTOSUB_GESTURE,
                    Command::UpdateClip {
                        clip_id: id.clone(),
                        patch: ClipPatch {
                            video_effects: Some(vec![filter]),
                            ..ClipPatch::default()
                        },
                    },
                );
            }
            count += 1;
        }
        self.transcript.clear();
        self.transcript_subject = None;
        self.selected_cue = None;
        self.open = false;
        self.message.clear();
        if count == 0 {
            studio.notify(&t("captions.nothingSaidClip"), true);
        } else {
            studio.notify(&tf("captions.addedCaptions", &[&count]), false);
        }
    }

    /// A cue row clicked: select it for the edit field, and jump the
    /// preview to the moment it is spoken - the cue's start on the
    /// timeline, from the subject the review is over.
    fn select_cue(&mut self, index: i32, studio: &mut Studio) {
        let Some(index) = usize::try_from(index).ok() else {
            return;
        };
        let Some(line) = self.transcript.get(index) else {
            return;
        };
        self.selected_cue = Some(index);
        if let Some(subject) = self.transcript_subject.as_ref() {
            let at = subject.start + line.start / subject.speed;
            studio.seek(at.max(0.0) as f32);
        }
    }

    /// The subject of the reviewed cues, when the clip it came from is
    /// still on the timeline unchanged: the same check
    /// [`Self::apply_transcript_edit`] makes, since a moved, retimed or
    /// re-cut clip would put the cues in the wrong places.
    fn live_cue_subject(&self, studio: &Studio) -> Option<Subject> {
        let subject = self.transcript_subject.as_ref()?;
        let clip = studio.clip(&subject.clip_id)?;
        if clip.media_id != subject.media_id
            || clip.track_id != subject.track_id
            || (clip.source_start - subject.source_start).abs() > 1e-6
            || (clip.duration - subject.duration).abs() > 1e-6
            || (clip.speed - subject.speed).abs() > 1e-9
            || clip.speed_curve.is_some()
        {
            return None;
        }
        Some(subject.clone())
    }
}

/// Imported cues keep the file's timing, including captions shorter than
/// generated speech segments. The title command applies the engine's minimum
/// clip duration where needed.
fn srt_commands(cues: Vec<srt::Cue>, look: (f64, f64), base: Option<&TextStyle>) -> Vec<Command> {
    cues.into_iter()
        .filter(|cue| !cue.text.trim().is_empty())
        .map(|cue| {
            caption_clip(
                cue.text.trim().to_owned(),
                cue.start,
                cue.end - cue.start,
                look,
                base,
            )
        })
        .collect()
}

/// The lane an SRT export writes out.
struct ExportLane {
    track_id: String,
    /// Chosen as the lane with the most titles, not by a selected title.
    busiest: bool,
}

/// The lane whose titles are written out as captions: the selected
/// title's; or with no title selected, the lane with the most titles -
/// captions come many to a lane, a title card one or two, and nothing
/// else in the document tells them apart (see `srt::busiest_title_lane`).
/// None when the timeline has no title.
fn export_lane(studio: &Studio) -> Option<ExportLane> {
    let selected = studio
        .sole_selection()
        .and_then(|id| studio.clip(&id))
        .filter(|clip| clip.kind == ClipKind::Text)
        .map(|clip| clip.track_id.clone());
    match selected {
        Some(track_id) => Some(ExportLane {
            track_id,
            busiest: false,
        }),
        None => srt::busiest_title_lane(studio.timeline()).map(|track_id| ExportLane {
            track_id,
            busiest: true,
        }),
    }
}

/// The name an export is offered under: the project's, as one file name,
/// or "captions" with no project or no usable name.
fn file_stem(studio: &Studio) -> String {
    let name = studio
        .session
        .as_ref()
        .map(|session| session.settings().name)
        .unwrap_or_default();
    let stem: String = name
        .trim()
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let stem = stem.trim_matches(|c: char| c == '.' || c.is_whitespace());
    if stem.is_empty() {
        "captions".to_owned()
    } else {
        stem.to_owned()
    }
}

/// `name` in `dir`, or `name (2)`, `name (3)`... until nothing there is
/// written over.
fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    let (stem, ext) = match name.rfind('.') {
        Some(dot) if dot > 0 => (&name[..dot], &name[dot..]),
        _ => (name, ""),
    };
    (2..)
        .map(|n| dir.join(format!("{stem} ({n}){ext}")))
        .find(|path| !path.exists())
        .unwrap_or(first)
}

/// [`CaptionsPane::subject_clip`] with the studio taken out: `selected`
/// the sole selection, `chosen_with` the selection when `clip` was chosen,
/// and `sounds` whether a clip is on the timeline with sound.
fn follow_selection(
    selected: Option<String>,
    chosen_with: Option<&str>,
    clip: Option<&str>,
    sounds: impl Fn(&str) -> bool,
) -> Option<String> {
    if let Some(selected) = selected.filter(|id| sounds(id))
        && Some(selected.as_str()) != chosen_with
    {
        return Some(selected);
    }
    clip.filter(|id| sounds(id)).map(str::to_owned)
}

/// A clip on the timeline whose voice can be captioned, as the sheet's
/// list shows it.
struct Source {
    id: String,
    label: String,
}

/// The clips on the current timeline that have sound, in the order they
/// play: an audio clip, or a video whose file has an audio stream.
fn sources(studio: &Studio) -> Vec<Source> {
    let project = studio.project();
    let mut clips: Vec<_> = studio
        .timeline()
        .clips
        .iter()
        .filter(|clip| studio.clip_has_sound(clip))
        .map(|clip| {
            let name = project
                .media_by_id(&clip.media_id)
                .map_or(clip.name.as_str(), |media| media.name.as_str());
            (clip.start, clip.id.clone(), name.to_owned())
        })
        .collect();
    clips.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    clips
        .into_iter()
        .map(|(start, id, name)| Source {
            id,
            label: source_label(&name, start),
        })
        .collect()
}

/// A clip as the list names it: its file and where it starts on the
/// timeline, so two clips of one file are told apart.
fn source_label(name: &str, start: f64) -> String {
    let seconds = start.max(0.0).floor() as u64;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{name} · {hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{name} · {minutes}:{seconds:02}")
    }
}

/// One caption as a title clip: its words, when and for how long, and
/// its look.
fn caption_clip(
    text: String,
    start: f64,
    duration: f64,
    look: (f64, f64),
    base: Option<&TextStyle>,
) -> Command {
    let (offset_y, font_size) = look;
    // The kept look where there is one - family, weight, colours, stroke,
    // shadow, background - at the caption's own size and place; the
    // bundled face otherwise.
    let style = match base {
        Some(base) => TextStyle {
            content: text,
            font_size,
            max_width: 0.0,
            max_height: 0.0,
            ..base.clone()
        },
        None => TextStyle {
            content: text,
            font_family: "Hanken Grotesk".to_owned(),
            font_size,
            font_weight: 600.0,
            ..TextStyle::default()
        },
    };
    Command::AddTextClip {
        track_id: None,
        above: true,
        start,
        style: Some(style),
        duration: Some(duration),
        offset_y: Some(offset_y),
        effects: None,
    }
}

/// A voice run's segments as cue lines, in segment order: each segment's
/// words split by [`words_to_cues`], so one spoken phrase is one cue. A
/// segment with words but no cue (nothing spoken) drops out; a segment
/// with text but no words at all keeps the whole segment as one cue, as
/// the old one-title-per-segment run did.
fn cues_of_segments(segments: &[Segment]) -> Vec<TranscriptLine> {
    let mut lines = Vec::new();
    for segment in segments {
        let cues = words_to_cues(&segment.words);
        if cues.is_empty() {
            let text = segment.text.trim().to_owned();
            if !text.is_empty() && segment.end > segment.start {
                lines.push(TranscriptLine {
                    start: segment.start,
                    end: segment.end,
                    text,
                    cut: false,
                    words: Vec::new(),
                });
            }
            continue;
        }
        lines.extend(cues.into_iter().map(|cue| TranscriptLine {
            start: cue.start,
            end: cue.end,
            text: cue.text,
            cut: false,
            words: cue.words,
        }));
    }
    lines
}

/// A cue clip's pop: scale 0.6 -> 1 and opacity 0 -> 1 over the first
/// [`POP_SECONDS`] of the clip, arriving slow ([`KeyEase::OUT`]) so the
/// cue lands instead of sliding in. The keys at the pop's end restore the
/// clip's own constants, so after the pop the cue sits still. `duration`
/// is the clip's timeline length; a cue shorter than the pop fills it.
/// A cue clip's entrance, by the animation row: 0 Pop (scale + opacity),
/// 1 Fade (opacity), 2 Slide (rise from below + opacity), 3 None. The
/// three that move play over the first [`POP_SECONDS`] of the clip and
/// restore the clip's own constants at the end, so after the entrance the
/// cue sits still. `duration` is the clip's timeline length; a cue shorter
/// than the entrance fills it.
/// The karaoke highlight for one cue clip, from the cue's own words:
/// the boundaries are where each next word starts, as fractions of the
/// cue, and the colour is the sheet's highlight colour. `None` when the
/// cue has no words (a whole segment kept as one line) or no span - such
/// a clip gets its entrance keys but no karaoke effect.
fn karaoke_filter(words: &[Word], start: f64, end: f64, highlight: &str) -> Option<AppliedFilter> {
    let span = end - start;
    if words.is_empty() || span <= 0.0 {
        return None;
    }
    let words = &words[..words.len().min(3)];
    let n = words.len() as f64;
    let boundary = |i: usize| {
        words
            .get(i)
            .map(|w| ((w.start - start) / span).clamp(0.0, 1.0))
            .unwrap_or(1.0)
    };
    let boundary1 = boundary(1);
    let boundary2 = boundary(2).max(boundary1);
    let colour = parse_colour(highlight).unwrap_or_else(|| colour_of(DEFAULT_HIGHLIGHT));
    let mut params = BTreeMap::new();
    params.insert("words".to_owned(), n);
    params.insert("boundary1".to_owned(), boundary1);
    params.insert("boundary2".to_owned(), boundary2);
    params.insert("highlight_r".to_owned(), colour.red() as f64 / 255.0);
    params.insert("highlight_g".to_owned(), colour.green() as f64 / 255.0);
    params.insert("highlight_b".to_owned(), colour.blue() as f64 / 255.0);
    // The important word stays highlighted for the whole cue while the
    // spoken word moves: the longest word, first on a tie - content
    // words run longer than fillers in practice. A one-word cue has
    // nothing to emphasise beyond the spoken word itself.
    let mut important = -1.0;
    if words.len() >= 2 {
        let mut longest = 0usize;
        for (i, word) in words.iter().enumerate() {
            let len = word.text.chars().count();
            if len > longest {
                longest = len;
                important = i as f64;
            }
        }
    }
    params.insert("important".to_owned(), important);
    Some(AppliedFilter {
        id: KARAOKE_EFFECT.to_owned(),
        params,
        enabled: true,
        keys: BTreeMap::new(),
    })
}

fn animation_keys(style: usize, clip_id: &str, duration: f64) -> Vec<Command> {
    let at = if duration > 0.0 {
        (POP_SECONDS / duration).min(1.0)
    } else {
        1.0
    };
    let key = |property: KeyProperty, at: f64, value: f64, ease: KeyEase| Command::SetClipKey {
        clip_id: clip_id.to_owned(),
        property,
        at,
        value,
        ease,
    };
    match style {
        // Pop: scale 0.6 -> 1 and opacity 0 -> 1, arriving slow.
        0 => vec![
            key(KeyProperty::Scale, 0.0, 0.6, KeyEase::LINEAR),
            key(KeyProperty::Scale, at, 1.0, KeyEase::OUT),
            key(KeyProperty::Opacity, 0.0, 0.0, KeyEase::LINEAR),
            key(KeyProperty::Opacity, at, 1.0, KeyEase::OUT),
        ],
        // Fade: opacity 0 -> 1 only.
        1 => vec![
            key(KeyProperty::Opacity, 0.0, 0.0, KeyEase::LINEAR),
            key(KeyProperty::Opacity, at, 1.0, KeyEase::OUT),
        ],
        // Slide: rise from below (offset 0.06 of the frame height) and fade in.
        2 => vec![
            key(KeyProperty::OffsetY, 0.0, 0.06, KeyEase::LINEAR),
            key(KeyProperty::OffsetY, at, 0.0, KeyEase::OUT),
            key(KeyProperty::Opacity, 0.0, 0.0, KeyEase::LINEAR),
            key(KeyProperty::Opacity, at, 1.0, KeyEase::OUT),
        ],
        // None: the cue just appears.
        _ => Vec::new(),
    }
}

/// The faces the font picker offers for the cue being reviewed: the
/// curated Vietnamese-safe bundled faces, with the cue's own family at
/// the end when it is none of these, so the picker never loses the look
/// a run started in.
fn cue_font_options(current: &str) -> Vec<String> {
    let mut options: Vec<String> =
        CUE_FONTS.iter().map(|family| family.to_string()).collect();
    if !current.is_empty() && !options.iter().any(|family| family == current) {
        options.push(current.to_owned());
    }
    options
}

/// Longest a caption line gets before it is wrapped: about what two lines
/// of broadcast subtitle hold, and what a reader takes in at a glance.
const CAPTION_CHARS: usize = 42;

/// A script as caption lines, each with how long it stays up: a line's
/// reading time at [`CHARS_PER_SECOND`], held to one second at least so
/// a short word is not a flicker, and seven at most so a long line does
/// not hang. A line break in the script is a break the author asked for;
/// within a paragraph a sentence is a caption, and a long sentence wraps
/// at its words.
fn script_captions(text: &str) -> Vec<(String, f64)> {
    text.lines()
        .flat_map(sentences)
        .flat_map(|sentence| wrap_caption(&sentence))
        .map(|line| {
            let seconds =
                (line.chars().count() as f64 / f64::from(CHARS_PER_SECOND)).clamp(1.0, 7.0);
            (line, seconds)
        })
        .collect()
}

/// A paragraph's sentences. A full stop, question or exclamation mark ends
/// one when it is followed by space or by the end - so "3.5" and "e.g." hold
/// together - and the CJK marks end one on their own.
fn sentences(paragraph: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut chars = paragraph.chars().peekable();
    while let Some(ch) = chars.next() {
        current.push(ch);
        let ends = match ch {
            '。' | '！' | '？' => true,
            '.' | '!' | '?' => chars.peek().is_none_or(|next| next.is_whitespace()),
            _ => false,
        };
        if ends {
            let sentence = current.trim();
            if !sentence.is_empty() {
                out.push(sentence.to_owned());
            }
            current.clear();
        }
    }
    let rest = current.trim();
    if !rest.is_empty() {
        out.push(rest.to_owned());
    }
    out
}

/// A sentence in lines of at most [`CAPTION_CHARS`], broken between words;
/// a word longer than a line, or a run of CJK with no spaces, is broken
/// where it must be.
fn wrap_caption(sentence: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut line_chars = 0;
    for word in sentence.split_whitespace() {
        let word_chars = word.chars().count();
        if line_chars > 0 && line_chars + 1 + word_chars > CAPTION_CHARS {
            lines.push(std::mem::take(&mut line));
            line_chars = 0;
        }
        if word_chars > CAPTION_CHARS {
            let mut piece = String::new();
            for ch in word.chars() {
                piece.push(ch);
                if piece.chars().count() == CAPTION_CHARS {
                    lines.push(std::mem::take(&mut piece));
                }
            }
            line = piece;
            line_chars = line.chars().count();
            continue;
        }
        if line_chars > 0 {
            line.push(' ');
            line_chars += 1;
        }
        line.push_str(word);
        line_chars += word_chars;
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::{animation_keys, cue_font_options, cues_of_segments, karaoke_filter, script_captions, source_label, Command, KeyEase, KeyProperty, TranscriptLine};
    use concat_speech::transcribe::{Segment, Word};

    #[test]
    fn imported_srt_preserves_short_cues_and_multiline_text() {
        let cues = concat_project::srt::parse(
            "1\n00:00:01,000 --> 00:00:01,100\nOlá\nAção\n\n2\n00:00:02,000 --> 00:00:03,000\nFim\n",
        ).unwrap();
        let commands = super::srt_commands(cues.clone(), (0.35, 0.04), None);
        let mut editor = concat_project::Editor::new();
        editor
            .apply(concat_project::Command::Batch { commands })
            .unwrap();
        let exported = concat_project::srt::write(&concat_project::srt::title_cues(
            editor.project().active(),
            None,
        ));
        assert_eq!(concat_project::srt::parse(&exported).unwrap(), cues);
        assert!(editor.undo());
        assert!(editor.project().active().clips.is_empty());
        assert!(editor.redo());
        assert_eq!(
            concat_project::srt::title_cues(editor.project().active(), None).len(),
            2
        );
    }

    /// The panel floats over a live editor: a clip with sound selected
    /// since the panel's clip was chosen is the one it shows and runs; a
    /// pick from its list holds until the selection moves again; and a
    /// chosen clip gone from the timeline is not run unseen.
    #[test]
    fn the_captioned_clip_follows_a_new_selection_of_a_clip_with_sound() {
        let sounds = |id: &str| ["a", "b", "c"].contains(&id);
        let follow = |selected: Option<&str>, chosen_with: Option<&str>, clip: Option<&str>| {
            super::follow_selection(selected.map(str::to_owned), chosen_with, clip, sounds)
        };
        // Opened on A, A still selected: A.
        assert_eq!(
            follow(Some("a"), Some("a"), Some("a")).as_deref(),
            Some("a")
        );
        // B selected on the timeline since: B.
        assert_eq!(
            follow(Some("b"), Some("a"), Some("a")).as_deref(),
            Some("b")
        );
        // C picked from the list with B selected: C, until the selection moves.
        assert_eq!(
            follow(Some("b"), Some("b"), Some("c")).as_deref(),
            Some("c")
        );
        // A title selected - no sound - leaves the choice alone.
        assert_eq!(
            follow(Some("t"), Some("b"), Some("c")).as_deref(),
            Some("c")
        );
        // The chosen clip deleted: nothing to run, rather than a clip unseen.
        assert_eq!(follow(None, None, Some("gone")), None);
    }

    #[test]
    fn imported_srt_ignores_empty_captions() {
        let cues = vec![concat_project::srt::Cue {
            start: 0.0,
            end: 1.0,
            text: "  ".into(),
        }];
        assert!(super::srt_commands(cues, (0.35, 0.04), None).is_empty());
    }

    /// A clip in the voice list is its file and where it starts, in
    /// minutes and seconds, with hours once there are any.
    #[test]
    fn a_voice_is_named_by_its_file_and_where_it_starts() {
        assert_eq!(source_label("Interview.mp4", 0.0), "Interview.mp4 · 0:00");
        assert_eq!(source_label("Interview.mp4", 72.9), "Interview.mp4 · 1:12");
        assert_eq!(source_label("Talk.wav", 3_725.0), "Talk.wav · 1:02:05");
        assert_eq!(source_label("Talk.wav", -1.0), "Talk.wav · 0:00");
    }

    /// A script becomes one caption per sentence, a hand line break is
    /// kept, a long sentence wraps at its words, and each line is held for
    /// its reading time within one to seven seconds.
    #[test]
    fn a_script_is_cut_into_readable_lines() {
        let lines = script_captions(
            "Hello there. This is version 3.5, mind!\n\nA sentence that runs on for far \
             longer than a caption line has any business running on for. Ok?",
        );
        let text: Vec<&str> = lines.iter().map(|(line, _)| line.as_str()).collect();
        assert_eq!(
            text,
            [
                "Hello there.",
                "This is version 3.5, mind!",
                "A sentence that runs on for far longer",
                "than a caption line has any business",
                "running on for.",
                "Ok?",
            ]
        );
        assert!(
            lines
                .iter()
                .all(|(_, seconds)| (1.0..=7.0).contains(seconds))
        );
        assert_eq!(lines[0].1, 1.0);
        assert!(lines[2].1 > lines[0].1);
        assert!(script_captions("  \n ").is_empty());
        assert_eq!(script_captions("你好。再见！").len(), 2);
    }

    #[test]
    fn voice_segments_become_cues_by_their_words() {
        let segments = vec![
            Segment {
                start: 0.0,
                end: 1.0,
                text: "hello brave world".to_owned(),
                words: vec![
                    Word { start: 0.0, end: 0.3, text: "hello".to_owned() },
                    Word { start: 0.3, end: 0.6, text: "brave".to_owned() },
                    Word { start: 0.6, end: 0.9, text: "world".to_owned() },
                ],
            },
            Segment {
                start: 1.5,
                end: 2.5,
                text: "no words here".to_owned(),
                words: Vec::new(),
            },
        ];
        assert_eq!(
            cues_of_segments(&segments),
            vec![
                TranscriptLine {
                    start: 0.0,
                    end: 0.9,
                    text: "hello brave world".to_owned(),
                    cut: false,
                    words: vec![
                        Word { start: 0.0, end: 0.3, text: "hello".to_owned() },
                        Word { start: 0.3, end: 0.6, text: "brave".to_owned() },
                        Word { start: 0.6, end: 0.9, text: "world".to_owned() },
                    ],
                },
                TranscriptLine {
                    start: 1.5,
                    end: 2.5,
                    text: "no words here".to_owned(),
                    cut: false,
                    words: Vec::new(),
                },
            ]
        );
    }

    fn word(start: f64, end: f64) -> Word {
        Word { start, end, text: String::new() }
    }

    fn word_with_text(start: f64, end: f64, text: &str) -> Word {
        Word { start, end, text: text.to_owned() }
    }

    #[test]
    fn karaoke_filter_follows_the_word_starts() {
        let filter = karaoke_filter(
            &[
                Word { start: 0.0, end: 0.3, text: "hello".to_owned() },
                Word { start: 0.3, end: 0.6, text: "brave".to_owned() },
                Word { start: 0.6, end: 0.9, text: "world".to_owned() },
            ],
            0.0,
            0.9,
            "#ffd60a",
        )
        .expect("three words get a filter");
        assert_eq!(filter.id, super::KARAOKE_EFFECT);
        assert!(filter.enabled);
        assert_eq!(filter.params.len(), 7);
        let p = &filter.params;
        assert!((p["words"] - 3.0).abs() < 1e-9);
        assert!((p["boundary1"] - 1.0 / 3.0).abs() < 1e-9);
        assert!((p["boundary2"] - 2.0 / 3.0).abs() < 1e-9);
        assert!((p["highlight_r"] - 1.0).abs() < 1e-9);
        assert!((p["highlight_g"] - 214.0 / 255.0).abs() < 1e-9);
        assert!((p["highlight_b"] - 10.0 / 255.0).abs() < 1e-9);
        // All three words are five letters, so the tie goes to the first
        // longest word - "hello" - the same rule the tie case in
        // `karaoke_filter_marks_the_longest_word_important` asserts.
        assert_eq!(p["important"], 0.0);
    }

    #[test]
    fn karaoke_filter_two_words_and_one_word() {
        let two = karaoke_filter(&[word(0.0, 0.4), word(0.4, 0.8)], 0.0, 0.8, "#ffffff")
            .expect("two words get a filter");
        assert_eq!(two.params["words"], 2.0);
        assert!((two.params["boundary1"] - 0.5).abs() < 1e-9);
        assert_eq!(two.params["boundary2"], 1.0);
        let one = karaoke_filter(&[word(0.0, 0.5)], 0.0, 0.5, "#ffffff")
            .expect("one word gets a filter");
        assert_eq!(one.params["words"], 1.0);
        assert_eq!(one.params["boundary1"], 1.0);
        assert_eq!(one.params["boundary2"], 1.0);
    }

    #[test]
    fn karaoke_filter_needs_words_and_a_span() {
        assert!(karaoke_filter(&[], 0.0, 0.9, "#ffd60a").is_none());
        assert!(karaoke_filter(&[word(0.0, 0.5)], 0.5, 0.5, "#ffd60a").is_none());
        assert!(karaoke_filter(&[word(0.0, 0.5)], 0.9, 0.0, "#ffd60a").is_none());
    }

    #[test]
    fn karaoke_filter_empty_highlight_falls_back_to_gold() {
        let filter = karaoke_filter(&[word(0.0, 0.5)], 0.0, 0.5, "").expect("one word");
        assert!((filter.params["highlight_r"] - 1.0).abs() < 1e-9);
        assert!((filter.params["highlight_g"] - 214.0 / 255.0).abs() < 1e-9);
        assert!((filter.params["highlight_b"] - 10.0 / 255.0).abs() < 1e-9);
        let junk = karaoke_filter(&[word(0.0, 0.5)], 0.0, 0.5, "not-a-colour").expect("one word");
        assert!((junk.params["highlight_r"] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn karaoke_filter_clamps_boundaries_into_the_cue() {
        // A next word starting past the cue cannot push its boundary
        // past the end, and one starting before the cue cannot push it
        // below the start.
        let late = karaoke_filter(&[word(0.0, 0.1), word(1.2, 1.5)], 0.0, 1.0, "#ffd60a")
            .expect("two words");
        assert_eq!(late.params["boundary1"], 1.0);
        let early = karaoke_filter(&[word(0.0, 0.1), word(-0.5, -0.1)], 0.0, 1.0, "#ffd60a")
            .expect("two words");
        assert_eq!(early.params["boundary1"], 0.0);
        // Boundaries stay in order even for out-of-order words.
        let mixed =
            karaoke_filter(&[word(0.0, 0.1), word(0.8, 0.9), word(0.6, 0.7)], 0.0, 1.0, "#ffd60a")
                .expect("three words");
        assert_eq!(mixed.params["boundary1"], 0.8);
        assert_eq!(mixed.params["boundary2"], 0.8);
    }

    #[test]
    fn karaoke_filter_marks_the_longest_word_important() {
        let filter = karaoke_filter(
            &[
                word_with_text(0.0, 0.2, "a"),
                word_with_text(0.2, 0.5, "brave"),
                word_with_text(0.5, 0.7, "cat"),
            ],
            0.0,
            1.0,
            "#ffd60a",
        )
        .expect("three words");
        assert_eq!(filter.params["important"], 1.0);
        // A tie goes to the first longest word.
        let tied = karaoke_filter(
            &[word_with_text(0.0, 0.3, "brave"), word_with_text(0.3, 0.6, "world")],
            0.0,
            1.0,
            "#ffd60a",
        )
        .expect("two words");
        assert_eq!(tied.params["important"], 0.0);
        // One word has nothing to emphasise beyond the spoken word.
        let one = karaoke_filter(&[word_with_text(0.0, 0.5, "hello")], 0.0, 0.5, "#ffd60a")
            .expect("one word");
        assert_eq!(one.params["important"], -1.0);
    }

    #[test]
    fn animation_keys_pop_over_the_first_half_second() {
        let at = 0.5 / 0.9;
        assert_eq!(
            animation_keys(0, "c1", 0.9),
            vec![
                Command::SetClipKey {
                    clip_id: "c1".to_owned(),
                    property: KeyProperty::Scale,
                    at: 0.0,
                    value: 0.6,
                    ease: KeyEase::LINEAR,
                },
                Command::SetClipKey {
                    clip_id: "c1".to_owned(),
                    property: KeyProperty::Scale,
                    at,
                    value: 1.0,
                    ease: KeyEase::OUT,
                },
                Command::SetClipKey {
                    clip_id: "c1".to_owned(),
                    property: KeyProperty::Opacity,
                    at: 0.0,
                    value: 0.0,
                    ease: KeyEase::LINEAR,
                },
                Command::SetClipKey {
                    clip_id: "c1".to_owned(),
                    property: KeyProperty::Opacity,
                    at,
                    value: 1.0,
                    ease: KeyEase::OUT,
                },
            ]
        );
    }

    #[test]
    fn animation_keys_fill_a_short_cue() {
        let keys = animation_keys(0, "c2", 0.3);
        let ends: Vec<f64> = keys
            .iter()
            .filter_map(|key| match key {
                Command::SetClipKey { at, .. } if *at > 0.0 => Some(*at),
                _ => None,
            })
            .collect();
        assert_eq!(ends, vec![1.0, 1.0]);
    }

    #[test]
    fn animation_keys_fade_is_opacity_only() {
        let keys = animation_keys(1, "c3", 0.9);
        assert_eq!(keys.len(), 2);
        assert!(keys.iter().all(|key| matches!(
            key,
            Command::SetClipKey { property: KeyProperty::Opacity, .. }
        )));
    }

    #[test]
    fn animation_keys_slide_rises_from_below() {
        let keys = animation_keys(2, "c4", 0.9);
        let offsets: Vec<(f64, f64)> = keys
            .iter()
            .filter_map(|key| match key {
                Command::SetClipKey {
                    property: KeyProperty::OffsetY,
                    at,
                    value,
                    ..
                } => Some((*at, *value)),
                _ => None,
            })
            .collect();
        assert_eq!(offsets, vec![(0.0, 0.06), (0.5 / 0.9, 0.0)]);
    }

    #[test]
    fn animation_keys_none_is_empty() {
        assert!(animation_keys(3, "c5", 0.9).is_empty());
        assert!(animation_keys(99, "c5", 0.9).is_empty());
    }

    #[test]
    fn cue_font_options_keep_the_curated_faces_and_the_current_one() {
        let curated = vec![
            "Be Vietnam Pro".to_owned(),
            "Fraunces".to_owned(),
            "Hanken Grotesk".to_owned(),
            "Inter".to_owned(),
        ];
        assert_eq!(cue_font_options("Hanken Grotesk"), curated);
        assert_eq!(cue_font_options(""), curated);
        let mut with_project = curated.clone();
        with_project.push("Some Project Font".to_owned());
        assert_eq!(cue_font_options("Some Project Font"), with_project);
    }
}
