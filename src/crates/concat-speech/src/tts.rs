// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Text-to-speech: text in, a narration WAV in the project folder out.
//!
//! One family only: **VieNeu-TTS v3 Turbo**, 25 Vietnamese speakers, read
//! by a small Python server on this machine (see [`VIENEU_ID`]) rather
//! than in-process, because the model is not sherpa-onnx and no
//! in-process engine can load it. The editor starts the server on first
//! use and reuses it while it answers; the voices are the model's own
//! presets, addressed by name. The older families - Kokoro, Pocket TTS,
//! VITS (VIVOS) and Chatterbox - were removed in 2026-11: their engines,
//! voice tables and model downloads are gone, and only VieNeu remains.
//!
//! Two pieces:
//!
//! 1. **Models.** The thirteen VieNeu files, downloaded on demand from
//!    HuggingFace into `<app data>/tts-models/<id>/`, or copied from the
//!    trial install when it is already on this machine. Streamed to a
//!    staging folder and renamed - a torn download must never look
//!    usable. The Python environment the sidecar runs in is created on
//!    demand the same way.
//! 2. **Synthesis.** Each request writes a WAV into the project's
//!    `audio/` folder (not `cache/` - cache is regenerable, narration the
//!    user placed on the timeline is not) and returns its path for the
//!    caller to import like any other media file. A read goes a sentence
//!    at a time so a long narration reports progress and can be
//!    cancelled between sentences.

use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use concat_host::{AppDirs, SingleFlight, projects};
use serde::{Deserialize, Serialize};

use crate::{DownloadProgress, JobStage};

/// Which network a bundle is, and so which voices it reads with. Only
/// one family remains: VieNeu. The enum stays because the settings panel
/// and the speech sheet still ask a model which family it belongs to.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Family {
    /// VieNeu-TTS v3 Turbo: 25 built-in Vietnamese speakers, read by a
    /// small Python server on this machine (the model is not sherpa-onnx,
    /// so no in-process engine can load it).
    #[default]
    VieNeu,
}

/// The family a model id belongs to: VieNeu, whichever bundle is asked
/// about. Any id outside the known table is refused by [`model_dir`]
/// before the family matters.
pub fn family_of(_id: &str) -> Family {
    Family::VieNeu
}

/// One downloadable bundle. Sizes are approximate, for progress bars when
/// the server declines to send a Content-Length.
struct KnownModel {
    id: &'static str,
    label: &'static str,
    /// One line for the settings row: what this size trades away.
    blurb: &'static str,
    approx_bytes: u64,
    /// What the finished download must hash to. Empty where the download
    /// is checked file by file instead; see [`concat_host::models`].
    sha256: &'static str,
}

/// The VieNeu bundle's id in the models table.
pub const VIENEU_ID: &str = "vieneu-v3-turbo-int8";
/// The port the VieNeu sidecar listens on, localhost only.
pub const VIENEU_PORT: u16 = 8471;
/// The Python server the editor starts for VieNeu reads, shipped with the
/// app and written next to the models on first use.
const VIENEU_SERVER_SCRIPT: &str = include_str!("vieneu-server.py");
/// Both folders together, for the settings row.
const VIENEU_TOTAL_BYTES: u64 = 256068309;

/// One file of a VieNeu install: where it lands locally, where it
/// comes from upstream, and the size it must arrive at. There is no
/// single archive to hash, so each file is checked against its size.
struct VieneuFile {
    /// `onnx` or `codec`: the folder inside the model dir.
    local: &'static str,
    /// The repo the file comes from.
    repo: &'static str,
    /// The folder inside the repo, empty for the root.
    upstream: &'static str,
    name: &'static str,
    size: u64,
}

const VIENEU_FILES: &[VieneuFile] = &[
    VieneuFile { local: "onnx", repo: "pnnbao-ump/VieNeu-TTS-v3-Turbo", upstream: "onnx_int8", name: "config.json", size: 2152 },
    VieneuFile { local: "onnx", repo: "pnnbao-ump/VieNeu-TTS-v3-Turbo", upstream: "onnx_int8", name: "tokenizer.json", size: 22320 },
    VieneuFile { local: "onnx", repo: "pnnbao-ump/VieNeu-TTS-v3-Turbo", upstream: "onnx_int8", name: "vieneu_prefill.onnx", size: 1090823 },
    VieneuFile { local: "onnx", repo: "pnnbao-ump/VieNeu-TTS-v3-Turbo", upstream: "onnx_int8", name: "vieneu_decode_step.onnx", size: 1062040 },
    VieneuFile { local: "onnx", repo: "pnnbao-ump/VieNeu-TTS-v3-Turbo", upstream: "onnx_int8", name: "vieneu_acoustic_cached.onnx", size: 7207223 },
    VieneuFile { local: "onnx", repo: "pnnbao-ump/VieNeu-TTS-v3-Turbo", upstream: "onnx_int8", name: "vieneu_backbone_shared.data", size: 103891968 },
    VieneuFile { local: "onnx", repo: "pnnbao-ump/VieNeu-TTS-v3-Turbo", upstream: "onnx_int8", name: "vieneu_v3_heads.npz", size: 52219622 },
    VieneuFile { local: "codec", repo: "OpenMOSS-Team/MOSS-Audio-Tokenizer-Nano-ONNX", upstream: "", name: "moss_audio_tokenizer_decode_full.onnx", size: 681902 },
    VieneuFile { local: "codec", repo: "OpenMOSS-Team/MOSS-Audio-Tokenizer-Nano-ONNX", upstream: "", name: "moss_audio_tokenizer_decode_shared.data", size: 44198912 },
    VieneuFile { local: "codec", repo: "OpenMOSS-Team/MOSS-Audio-Tokenizer-Nano-ONNX", upstream: "", name: "moss_audio_tokenizer_decode_step.onnx", size: 351400 },
    VieneuFile { local: "codec", repo: "OpenMOSS-Team/MOSS-Audio-Tokenizer-Nano-ONNX", upstream: "", name: "codec_browser_onnx_meta.json", size: 17036 },
    VieneuFile { local: "codec", repo: "OpenMOSS-Team/MOSS-Audio-Tokenizer-Nano-ONNX", upstream: "", name: "moss_audio_tokenizer_encode.onnx", size: 815775 },
    VieneuFile { local: "codec", repo: "OpenMOSS-Team/MOSS-Audio-Tokenizer-Nano-ONNX", upstream: "", name: "moss_audio_tokenizer_encode.data", size: 44507136 },
];

/// The models the settings panel offers: VieNeu, and nothing else.
const KNOWN_MODELS: &[KnownModel] = &[KnownModel {
    id: VIENEU_ID,
    label: "VieNeu-TTS v3 Turbo (Vietnamese)",
    blurb: "25 Vietnamese voices, the most natural local read. Runs through a small server on this PC.",
    approx_bytes: VIENEU_TOTAL_BYTES,
    // Thirteen files, each checked against its own size in
    // `download_vieneu`; no single archive, so no hash here.
    sha256: "",
}];

/// The base of the VieNeu voices' ids: the 25 VieNeu speakers live here,
/// in the order of the model's own preset list, and the server is asked
/// for them by name. The third column is what the voice picker shows
/// under the name: gender, region, style.
pub const VIENEU_BASE: i32 = 4000;
/// The 25 preset voices: id, name, and the gender, region and style
/// line the voice picker shows under the name.
pub const VIENEU_VOICES: &[(i32, &str, &str)] = &[
    (4000, "Adam bựa", "Nam · Bắc · tự nhiên"),
    (4001, "Trúc Ly", "Nữ · Bắc · tự nhiên"),
    (4002, "Thiện Minh", "Nam · Bắc · kể chuyện"),
    (4003, "Mai Anh", "Nữ · Bắc · tin tức"),
    (4004, "Hải Đăng", "Nam · Bắc · tự nhiên"),
    (4005, "Thùy Dung", "Nữ · Nam · tin tức"),
    (4006, "Thiền Tâm Đức", "Nam · Bắc · kể chuyện"),
    (4007, "Ngọc Huyền", "Nữ · Bắc · tự nhiên"),
    (4008, "Quang Sơn", "Nam · Trung · tự nhiên"),
    (4009, "Ngọc Trân", "Nữ · Trung · tự nhiên"),
    (4010, "Minh Đức", "Nam · Bắc · tin tức"),
    (4011, "Phạm Tuyên", "Nam · Bắc · tự nhiên"),
    (4012, "Thái Sơn", "Nam · Nam · kể chuyện"),
    (4013, "Xuân Vĩnh", "Nam · Bắc · tự nhiên"),
    (4014, "Thanh Bình", "Nam · Bắc · kể chuyện"),
    (4015, "Ngọc Linh", "Nữ · Bắc · kể chuyện"),
    (4016, "Đoan Trang", "Nữ · Bắc · tự nhiên"),
    (4017, "Thục Đoan", "Nữ · Nam · kể chuyện"),
    (4018, "Minh Triết", "Nam · Nam · tin tức"),
    (4019, "Mỹ Duyên", "Nữ · Nam · đọc truyện"),
    (4020, "Quỳnh Anh", "Nữ · Bắc · đọc truyện"),
    (4021, "Đức Trí", "Nam · Nam · đọc truyện"),
    (4022, "Kim Thanh", "Nữ · Nam · đọc truyện"),
    (4023, "Adam", "Nam · Nam · tự nhiên"),
    (4024, "Quốc Tuấn", "Nam · Bắc · tự nhiên"),
];

/// The voice the sheet opens on: the model's own default.
pub const VIENEU_DEFAULT: i32 = 4004;

/// Which family a voice id belongs to: VieNeu for one of its 25 presets,
/// nothing for anything else.
pub fn voice_family(voice: i32) -> Option<Family> {
    if VIENEU_VOICES.iter().any(|(id, _, _)| *id == voice) {
        Some(Family::VieNeu)
    } else {
        None
    }
}

/// The upstream name of a voice, for the log line and the file name.
fn voice_name(voice: i32) -> Option<&'static str> {
    VIENEU_VOICES
        .iter()
        .find(|(id, _, _)| *id == voice)
        .map(|(_, name, _)| *name)
}

/// What the voice picker shows under a VieNeu voice's name, if it is one.
pub fn vieneu_voice_detail(name: &str) -> Option<&'static str> {
    VIENEU_VOICES
        .iter()
        .find(|(_, voice, _)| *voice == name)
        .map(|(_, _, detail)| *detail)
}

fn known(id: &str) -> Option<&'static KnownModel> {
    KNOWN_MODELS.iter().find(|model| model.id == id)
}

/// Where downloaded models live: `<app data>/tts-models/<id>/`.
fn models_dir(dirs: &AppDirs) -> Result<PathBuf, String> {
    let dir = dirs.data.join("tts-models");
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("could not create {}: {error}", dir.display()))?;
    Ok(dir)
}

fn model_dir(dirs: &AppDirs, id: &str) -> Result<PathBuf, String> {
    // Ids come from our own table; anything else is a bug, not input.
    known(id).ok_or_else(|| format!("unknown model {id:?}"))?;
    Ok(models_dir(dirs)?.join(id))
}

/// A bundle counts as downloaded once its files are in place: the app's
/// own copy, or the trial install it falls back to.
fn model_downloaded(dirs: &AppDirs, id: &str) -> bool {
    model_dir(dirs, id).is_ok() && vieneu_model_dirs(dirs).is_some()
}

/// One model, as the settings panel shows it.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    /// The bundle id.
    pub id: String,
    /// Which family, and so which voices it reads with.
    pub family: Family,
    /// Display name.
    pub label: String,
    /// One line on what this size trades away.
    pub blurb: String,
    /// Approximate download size in bytes, for display.
    pub size_bytes: u64,
    /// Whether the bundle is unpacked on disk.
    pub downloaded: bool,
}

/// One speaker.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VoiceInfo {
    /// The voice id, what synthesis wants back: one of the VieNeu
    /// presets, 4000..=4024.
    pub id: i32,
    /// The voice name, e.g. "Hải Đăng"; the picker shows it as it is.
    pub name: String,
    /// The family whose models read with this voice.
    pub family: Family,
    /// A picture of the voice, as a path to an image file, for the voice
    /// picker's avatar: none of the stock voices has one, and the picker
    /// draws a letter in a colour instead. For a voice someone adds later
    /// with a picture of their own.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
}

/// A recording to read in the voice of. VieNeu reads only its own preset
/// voices and ignores this; the shape stays so older callers - and
/// older saved requests - still deserialize.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    /// The media file.
    pub path: String,
    /// Seconds into the file the voice is heard from.
    #[serde(default)]
    pub start: f64,
}

/// What the settings panel and the speech dialog show.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TtsStatus {
    /// Where models are stored, so the settings panel can say so.
    pub models_dir: String,
    /// Every model the panel offers.
    pub models: Vec<ModelStatus>,
    /// Every speaker on offer.
    pub voices: Vec<VoiceInfo>,
}

/// What to say, and where the file goes.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpeakRequest {
    /// Which model to use, e.g. "vieneu-v3-turbo-int8".
    pub model_id: String,
    /// The voice, from the voices table; it must belong to the model's
    /// family.
    pub voice: i32,
    /// Ignored by VieNeu, which reads only its own preset voices. Kept
    /// so older callers still deserialize.
    #[serde(default)]
    pub reference: Option<Reference>,
    /// What to say.
    pub text: String,
    /// Speaking rate; 1.0 is the voice's natural pace. Ignored by
    /// VieNeu, which reads at its own pace.
    pub speed: f32,
    /// Ignored by VieNeu. Kept so older callers still deserialize.
    #[serde(default)]
    pub pauses: Option<f32>,
    /// Ignored by VieNeu. Kept so older callers still deserialize.
    #[serde(default)]
    pub steps: Option<i32>,
    /// The project folder the WAV should land in.
    pub project: String,
}

/// What synthesis produced.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpeakResult {
    /// Absolute path of the written WAV, ready for the media import path.
    pub path: String,
    /// Seconds of audio, so the caller can say something useful.
    pub duration: f64,
}

/// A loaded model: VieNeu reads through its sidecar, so no network is
/// loaded here, only the port the server answers on.
enum Engine {
    /// VieNeu reads through its sidecar: no network is loaded here, only
    /// the port the server answers on.
    VieNeu { port: u16 },
}

struct CachedEngine {
    model_id: String,
    tts: Engine,
}

/// The one synthesis and the one download that can run at a time, plus the
/// loaded engine.
///
/// The sidecar is started once and kept while it answers; the cached
/// port is what a read and a preview share. The mutex is held for the
/// whole synthesis, which is what lets [`Speech::delete_model`] use
/// `try_lock` to refuse deleting a model mid-use instead of yanking
/// mapped files out from under the session.
pub struct Speech {
    gate: Arc<SingleFlight>,
    downloads: Arc<SingleFlight>,
    engine: Mutex<Option<CachedEngine>>,
}

impl Default for Speech {
    fn default() -> Self {
        Self::new()
    }
}

impl Speech {
    /// An idle speech engine, loading nothing until asked to speak.
    pub fn new() -> Self {
        Self {
            gate: Arc::new(SingleFlight::new()),
            downloads: Arc::new(SingleFlight::new()),
            engine: Mutex::new(None),
        }
    }

    /// Models and voices, for the settings panel and the speech dialog.
    pub fn status(dirs: &AppDirs) -> Result<TtsStatus, String> {
        let dir = models_dir(dirs)?;
        Ok(TtsStatus {
            models_dir: dir.to_string_lossy().into_owned(),
            models: KNOWN_MODELS
                .iter()
                .map(|model| ModelStatus {
                    id: model.id.to_owned(),
                    family: family_of(model.id),
                    label: model.label.to_owned(),
                    blurb: model.blurb.to_owned(),
                    size_bytes: model.approx_bytes,
                    downloaded: model_downloaded(dirs, model.id),
                })
                .collect(),
            voices: VIENEU_VOICES
                .iter()
                .map(|(id, name, _)| VoiceInfo {
                    id: *id,
                    name: (*name).to_owned(),
                    family: Family::VieNeu,
                    avatar: None,
                })
                .collect(),
        })
    }

    /// Downloads the VieNeu bundle: its thirteen files and the Python
    /// environment the sidecar runs in. Blocks for the whole download:
    /// run it on its own thread.
    pub fn download_model(
        &self,
        dirs: &AppDirs,
        id: &str,
        mut progress: impl FnMut(DownloadProgress),
    ) -> Result<(), String> {
        let job = self.downloads.begin("voice model download")?;
        let cancel = job.cancel_flag();
        // Only a known id reaches the download; anything else is refused
        // here rather than halfway through.
        model_dir(dirs, id)?;
        download_vieneu(dirs, cancel, &mut progress)
    }

    /// Asks the running download to stop. Idle is a harmless no-op.
    pub fn cancel_download(&self) {
        self.downloads.cancel();
    }

    /// Removes a downloaded model folder, unless the engine is speaking from it.
    pub fn delete_model(&self, dirs: &AppDirs, id: &str) -> Result<(), String> {
        let dir = model_dir(dirs, id)?;
        if !dir.exists() {
            return Ok(());
        }
        let mut engine = self
            .engine
            .try_lock()
            .map_err(|_| "speech is being generated - wait for it or cancel it first".to_owned())?;
        if engine.as_ref().is_some_and(|cached| cached.model_id == id) {
            *engine = None;
        }
        std::fs::remove_dir_all(&dir)
            .map_err(|error| format!("could not remove {}: {error}", dir.display()))
    }

    /// The model still loaded from the last read, which the next read
    /// with it does not load again. None while a read holds it.
    pub fn loaded_model(&self) -> Option<String> {
        let engine = self.engine.try_lock().ok()?;
        engine.as_ref().map(|cached| cached.model_id.clone())
    }

    /// Loads the engine for `model_id` when it is not the one cached, and
    /// leaves it cached: the load a read and a preview share. The lock is
    /// held only for the load itself, so the caller takes it again to use
    /// what was loaded.
    fn ensure_engine(
        &self,
        dirs: &AppDirs,
        model_id: &str,
        progress: &mut dyn FnMut(JobStage),
    ) -> Result<(), String> {
        let dir = model_dir(dirs, model_id)?;
        if !model_downloaded(dirs, model_id) {
            return Err(format!(
                "model {model_id} is not downloaded - see Settings > Speech"
            ));
        }
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| "speech state poisoned".to_owned())?;
        let stale = engine
            .as_ref()
            .is_none_or(|cached| cached.model_id != model_id);
        if !stale {
            return Ok(());
        }
        // Load before overwriting: a failed load keeps the old engine.
        progress(JobStage::Loading);
        let loading = std::time::Instant::now();
        log::info!("tts: loading {model_id} from {}", dir.display());
        let port = self.ensure_vieneu_sidecar(dirs)?;
        log::info!(
            "tts: {model_id} loaded in {:.1}s",
            loading.elapsed().as_secs_f32()
        );
        *engine = Some(CachedEngine {
            model_id: model_id.to_owned(),
            tts: Engine::VieNeu { port },
        });
        Ok(())
    }

    /// The sidecar a VieNeu read goes through, starting it when it is not
    /// already answering: the server keeps running after the first read so
    /// the next one does not pay the engine load again.
    fn ensure_vieneu_sidecar(&self, dirs: &AppDirs) -> Result<u16, String> {
        if vieneu_health(VIENEU_PORT) {
            return Ok(VIENEU_PORT);
        }
        let Some((onnx_dir, codec_dir)) = vieneu_model_dirs(dirs) else {
            return Err(format!(
                "model {VIENEU_ID} is not downloaded - see Settings > Speech"
            ));
        };
        let Some(python) = vieneu_python(dirs) else {
            return Err(
                "VieNeu's Python environment is missing - download the model in Settings > Speech"
                    .to_owned(),
            );
        };
        let home = dirs.data.join("vieneu");
        std::fs::create_dir_all(&home)
            .map_err(|error| format!("could not create {}: {error}", home.display()))?;
        // The server script ships with the app; the copy on disk is what
        // runs, refreshed when the shipped one changes.
        let script = home.join("vieneu-server.py");
        let current = std::fs::read_to_string(&script).unwrap_or_default();
        if current != VIENEU_SERVER_SCRIPT {
            std::fs::write(&script, VIENEU_SERVER_SCRIPT)
                .map_err(|error| format!("could not write {}: {error}", script.display()))?;
        }
        let log = std::fs::File::create(home.join("server.log"))
            .map_err(|error| format!("could not create the VieNeu server log: {error}"))?;
        let err = log
            .try_clone()
            .map_err(|error| format!("could not create the VieNeu server log: {error}"))?;
        std::process::Command::new(&python)
            .arg(&script)
            .arg("--port")
            .arg(VIENEU_PORT.to_string())
            .arg("--onnx-dir")
            .arg(&onnx_dir)
            .arg("--codec-dir")
            .arg(&codec_dir)
            .stdout(log)
            .stderr(err)
            .spawn()
            .map_err(|error| format!("could not start the VieNeu server: {error}"))?;
        // The engine takes seconds to load; wait for it to answer.
        for _ in 0..90 {
            if vieneu_health(VIENEU_PORT) {
                log::info!("tts: VieNeu sidecar answering on port {VIENEU_PORT}");
                return Ok(VIENEU_PORT);
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        Err("the VieNeu server did not answer - see the server log in the app data folder"
            .to_owned())
    }

    /// Reads `text` through the VieNeu sidecar, a sentence at a time so a
    /// long narration reports progress and can be cancelled between
    /// sentences, and joins the WAVs into one.
    fn vieneu_speak(
        &self,
        port: u16,
        voice: &str,
        text: &str,
        cancel: &AtomicBool,
        progress: &mut dyn FnMut(JobStage),
    ) -> Result<Vec<u8>, String> {
        let chunks = sentence_chunks(text);
        let total = chunks.len().max(1);
        let mut wavs = Vec::with_capacity(chunks.len());
        for (index, chunk) in chunks.iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                return Err("speech generation cancelled".to_owned());
            }
            progress(JobStage::Working {
                done: index as f32 / total as f32,
                next: (index as f32 + 1.0) / total as f32,
            });
            wavs.push(vieneu_speak_one(port, voice, chunk)?);
        }
        concat_wavs(&wavs)
    }

    /// A VieNeu read, through the sidecar and into `file`.
    fn read_vieneu_to_file(
        &self,
        port: u16,
        voice: &str,
        text: &str,
        file: &Path,
        cancel: &AtomicBool,
        progress: &mut dyn FnMut(JobStage),
        started: &std::time::Instant,
    ) -> Result<SpeakResult, String> {
        progress(JobStage::Reading);
        let wav = self.vieneu_speak(port, voice, text, cancel, progress)?;
        if cancel.load(Ordering::Relaxed) {
            return Err("speech generation cancelled".to_owned());
        }
        std::fs::write(file, &wav)
            .map_err(|error| format!("could not write {}: {error}", file.display()))?;
        let duration = wav_duration(&wav);
        log::info!(
            "tts: wrote {} ({duration:.2}s at 48000 Hz) in {:.1}s",
            file.display(),
            started.elapsed().as_secs_f32()
        );
        Ok(SpeakResult {
            path: file.to_string_lossy().into_owned(),
            duration,
        })
    }

    /// Synthesizes one narration clip into `<project>/audio/`. Blocks for
    /// the whole synthesis: run it on its own thread. `progress` is told
    /// each stage as it begins, and in the reading itself how far it is,
    /// a sentence at a time.
    pub fn speak(
        &self,
        dirs: &AppDirs,
        request: &SpeakRequest,
        mut progress: impl FnMut(JobStage) + Send + 'static,
    ) -> Result<SpeakResult, String> {
        let job = self.gate.begin("speech generation")?;
        let cancel = job.cancel_handle();
        let started = std::time::Instant::now();

        let text = request.text.trim();
        if text.is_empty() {
            return Err("nothing to say: the text is empty".to_owned());
        }
        let family = family_of(&request.model_id);
        log::info!(
            "tts: {} reading {} characters with voice {} ({})",
            request.model_id,
            text.chars().count(),
            request.voice,
            voice_name(request.voice).unwrap_or("?"),
        );
        match voice_family(request.voice) {
            Some(owner) if owner == family => {}
            Some(_) => {
                return Err(format!(
                    "voice {} does not belong to {}",
                    request.voice, request.model_id
                ));
            }
            None => return Err(format!("unknown voice {}", request.voice)),
        }
        if !model_downloaded(dirs, &request.model_id) {
            return Err(format!(
                "model {} is not downloaded - see Settings > Speech",
                request.model_id
            ));
        }

        // The WAV goes in the project so it travels (and dies) with it - and
        // in `audio/`, not `cache/`, because a clip on the timeline points at
        // it: cache is for things that can be regenerated.
        let root = Path::new(&request.project);
        if !projects::is_project(root) {
            return Err(format!("{} is not a project folder", request.project));
        }
        let out_dir = root.join("audio");
        std::fs::create_dir_all(&out_dir)
            .map_err(|error| format!("could not create {}: {error}", out_dir.display()))?;

        self.ensure_engine(dirs, &request.model_id, &mut progress)?;
        let engine = self
            .engine
            .lock()
            .map_err(|_| "speech state poisoned".to_owned())?;
        let Some(cached) = engine.as_ref() else {
            return Err("the voice model was removed - download it again".to_owned());
        };
        let Engine::VieNeu { port } = &cached.tts;

        // Wall-clock millis plus process id: unique enough for files created
        // by one human clicking a button, and stable for the media bin name.
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_millis())
            .unwrap_or(0);
        let voice_name = voice_name(request.voice).unwrap_or("voice");
        let file = out_dir.join(format!("speech-{voice_name}-{stamp}.wav"));

        self.read_vieneu_to_file(
            *port,
            voice_name,
            text,
            &file,
            &cancel,
            &mut progress,
            &started,
        )
    }

    /// Reads one short sentence in a voice so it can be heard before a
    /// whole narration is generated: the same engine and speaker as a read,
    /// written to `<app data>/tts-preview/voice-<id>.wav` and played back
    /// from there.
    pub fn preview_voice(
        &self,
        dirs: &AppDirs,
        model_id: &str,
        voice: i32,
        text: &str,
    ) -> Result<PathBuf, String> {
        let job = self.gate.begin("voice preview")?;
        let cancel = job.cancel_handle();
        let text = text.trim();
        if text.is_empty() {
            return Err("nothing to preview: the text is empty".to_owned());
        }
        let family = family_of(model_id);
        match voice_family(voice) {
            Some(owner) if owner == family => {}
            Some(_) => {
                return Err(format!("voice {voice} does not belong to {model_id}"));
            }
            None => return Err(format!("unknown voice {voice}")),
        }
        self.ensure_engine(dirs, model_id, &mut |_| {})?;
        let engine = self
            .engine
            .lock()
            .map_err(|_| "speech state poisoned".to_owned())?;
        let Some(cached) = engine.as_ref() else {
            return Err("the voice model was removed - download it again".to_owned());
        };
        let Engine::VieNeu { port } = &cached.tts;
        let port = *port;
        let out_dir = dirs.data.join("tts-preview");
        std::fs::create_dir_all(&out_dir)
            .map_err(|error| format!("could not create {}: {error}", out_dir.display()))?;
        let file = out_dir.join(format!("voice-{voice}.wav"));
        let name = voice_name(voice).unwrap_or("voice");
        let wav = self.vieneu_speak(port, name, text, &cancel, &mut |_| {})?;
        if cancel.load(Ordering::Relaxed) {
            return Err("voice preview cancelled".to_owned());
        }
        std::fs::write(&file, &wav)
            .map_err(|error| format!("could not write {}: {error}", file.display()))?;
        Ok(file)
    }

    /// Asks the running synthesis to stop at the next sentence boundary.
    pub fn cancel(&self) {
        self.gate.cancel();
    }

    /// Whether a synthesis is running.
    pub fn is_busy(&self) -> bool {
        self.gate.is_busy()
    }
}

/// Whether the sidecar answers on `port`.
fn vieneu_health(port: u16) -> bool {
    ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(2))
        .timeout_read(std::time::Duration::from_secs(2))
        .build()
        .get(&format!("http://127.0.0.1:{port}/health"))
        .call()
        .is_ok_and(|response| response.status() == 200)
}

/// Whether every file of one folder of a VieNeu install is in place, at
/// the size it arrived at.
fn vieneu_files_complete(dir: &Path, local: &str) -> bool {
    VIENEU_FILES
        .iter()
        .filter(|file| file.local == local)
        .all(|file| {
            std::fs::metadata(dir.join(file.name))
                .is_ok_and(|meta| meta.len() == file.size)
        })
}

/// Where a VieNeu install's two folders are: the app's own copy first,
/// then the trial install from 2026-11-13, which stays usable until the
/// app's copy is downloaded.
fn vieneu_model_dirs(dirs: &AppDirs) -> Option<(PathBuf, PathBuf)> {
    let app = model_dir(dirs, VIENEU_ID).ok()?;
    if vieneu_files_complete(&app.join("onnx"), "onnx")
        && vieneu_files_complete(&app.join("codec"), "codec")
    {
        return Some((app.join("onnx"), app.join("codec")));
    }
    trial_vieneu_dirs()
}

fn trial_vieneu_dirs() -> Option<(PathBuf, PathBuf)> {
    let trial = PathBuf::from(r"D:\Apps\VieNeuTest\models");
    if vieneu_files_complete(&trial.join("v3-turbo-int8"), "onnx")
        && vieneu_files_complete(&trial.join("moss-codec"), "codec")
    {
        return Some((trial.join("v3-turbo-int8"), trial.join("moss-codec")));
    }
    None
}

/// The Python that can run the sidecar: the app's own environment first,
/// then the trial one from 2026-11-13.
fn vieneu_python(dirs: &AppDirs) -> Option<PathBuf> {
    let app = dirs
        .data
        .join("vieneu")
        .join("venv")
        .join("Scripts")
        .join("python.exe");
    if app.is_file() {
        return Some(app);
    }
    let trial = PathBuf::from(r"D:\Apps\VieNeuTest\venv\Scripts\python.exe");
    if trial.is_file() {
        return Some(trial);
    }
    None
}

/// A Python that can build the environment: the winget install first,
/// then whatever `python` on PATH turns out to be. The Store stub does
/// not count: it answers `--version` with an install prompt, not a
/// version.
fn find_system_python() -> Option<PathBuf> {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        for version in ["Python312", "Python313", "Python311", "Python310"] {
            let candidate = PathBuf::from(&local)
                .join("Programs")
                .join("Python")
                .join(version)
                .join("python.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let output = std::process::Command::new("python")
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    if String::from_utf8_lossy(&output.stdout).starts_with("Python 3.") {
        Some(PathBuf::from("python"))
    } else {
        None
    }
}

/// Fetches the thirteen VieNeu files and the Python environment that
/// reads them. The files arrive one by one from upstream (there is no
/// archive to unpack); a copy from the trial install on this machine,
/// when it is already there, skips the download. The Python environment
/// is created only when neither the app's own nor the trial one is on
/// disk. The download runs under the same SingleFlight as the other
/// models: `download_model` began the job before calling here.
fn download_vieneu(
    dirs: &AppDirs,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(DownloadProgress),
) -> Result<(), String> {
    let destination = model_dir(dirs, VIENEU_ID)?;
    if vieneu_files_complete(&destination.join("onnx"), "onnx")
        && vieneu_files_complete(&destination.join("codec"), "codec")
    {
        return Ok(());
    }
    let parent = models_dir(dirs)?;
    let staging = parent.join(format!(".staging-{VIENEU_ID}"));
    let result = (|| {
        let _ = std::fs::remove_dir_all(&staging);
        std::fs::create_dir_all(staging.join("onnx"))
            .and_then(|()| std::fs::create_dir_all(staging.join("codec")))
            .map_err(|error| format!("could not create {}: {error}", staging.display()))?;
        if let Some((onnx, codec)) = trial_vieneu_dirs() {
            // The trial install is already verified file by file; copying
            // it is a download that cannot fail on the network.
            let mut done: u64 = 0;
            for file in VIENEU_FILES {
                let from = if file.local == "onnx" { &onnx } else { &codec };
                std::fs::copy(from.join(file.name), staging.join(file.local).join(file.name))
                    .map_err(|error| {
                        format!("could not copy {}: {error}", file.name)
                    })?;
                done += file.size;
                progress(DownloadProgress {
                    id: VIENEU_ID.to_owned(),
                    received: done,
                    total: VIENEU_TOTAL_BYTES,
                    unpacking: false,
                    done: false,
                });
            }
        } else {
            let mut done: u64 = 0;
            for file in VIENEU_FILES {
                let url = if file.upstream.is_empty() {
                    format!(
                        "https://huggingface.co/{}/resolve/main/{}",
                        file.repo, file.name
                    )
                } else {
                    format!(
                        "https://huggingface.co/{}/resolve/main/{}/{}",
                        file.repo, file.upstream, file.name
                    )
                };
                let partial = staging.join(file.local).join(file.name);
                let base = done;
                let (received, _) = concat_host::models::download(
                    &url,
                    &partial,
                    file.size,
                    cancel,
                    "download cancelled",
                    &mut |received, _| {
                        progress(DownloadProgress {
                            id: VIENEU_ID.to_owned(),
                            received: base + received,
                            total: VIENEU_TOTAL_BYTES,
                            unpacking: false,
                            done: false,
                        });
                    },
                )?;
                done += received;
            }
        }
        if !vieneu_files_complete(&staging.join("onnx"), "onnx")
            || !vieneu_files_complete(&staging.join("codec"), "codec")
        {
            return Err("the VieNeu files did not arrive complete".to_owned());
        }
        let _ = std::fs::remove_dir_all(&destination);
        std::fs::rename(&staging, &destination)
            .map_err(|error| format!("could not finish {}: {error}", destination.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    result?;
    ensure_vieneu_python(dirs, progress)?;
    progress(DownloadProgress {
        id: VIENEU_ID.to_owned(),
        received: VIENEU_TOTAL_BYTES,
        total: VIENEU_TOTAL_BYTES,
        unpacking: false,
        done: true,
    });
    Ok(())
}

/// The Python environment the sidecar runs in, creating it when neither
/// the app's own nor the trial one is on disk.
fn ensure_vieneu_python(
    dirs: &AppDirs,
    progress: &mut dyn FnMut(DownloadProgress),
) -> Result<(), String> {
    if vieneu_python(dirs).is_some() {
        return Ok(());
    }
    progress(DownloadProgress {
        id: VIENEU_ID.to_owned(),
        received: VIENEU_TOTAL_BYTES,
        total: VIENEU_TOTAL_BYTES,
        unpacking: true,
        done: false,
    });
    let Some(system) = find_system_python() else {
        return Err(
            "VieNeu needs Python 3.12 to read - install it with `winget install Python.Python.3.12`, then download the model again"
                .to_owned(),
        );
    };
    let venv = dirs.data.join("vieneu").join("venv");
    let result = (|| {
        let status = std::process::Command::new(&system)
            .arg("-m")
            .arg("venv")
            .arg(&venv)
            .output()
            .map_err(|error| format!("could not create the VieNeu Python environment: {error}"))?;
        if !status.status.success() {
            return Err("could not create the VieNeu Python environment".to_owned());
        }
        let python = venv.join("Scripts").join("python.exe");
        let status = std::process::Command::new(&python)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("vieneu==3.8.4")
            .output()
            .map_err(|error| format!("could not install VieNeu: {error}"))?;
        if !status.status.success() {
            return Err("could not install VieNeu into its Python environment".to_owned());
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&venv);
    }
    result
}

/// The stretches a VieNeu read is sent in: sentences, kept whole while
/// they fit the server's chunk, split at commas past that. The server
/// chunks on its own too, but a sentence at a time is what the progress
/// bar and the cancel flag can see.
fn sentence_chunks(text: &str) -> Vec<String> {
    const MAX_CHUNK: usize = 200;
    fn push(chunk: &str, chunks: &mut Vec<String>) {
        let chunk = chunk.trim();
        if chunk.is_empty() {
            return;
        }
        if chunk.chars().count() <= MAX_CHUNK {
            chunks.push(chunk.to_owned());
            return;
        }
        // Too long for one read: split at commas, packing what fits.
        let mut current = String::new();
        for piece in chunk.split(',') {
            let piece = piece.trim();
            if piece.is_empty() {
                continue;
            }
            if !current.is_empty()
                && current.chars().count() + piece.chars().count() + 2 > MAX_CHUNK
            {
                chunks.push(std::mem::take(&mut current));
            }
            if !current.is_empty() {
                current.push_str(", ");
            }
            current.push_str(piece);
        }
        if !current.trim().is_empty() {
            chunks.push(current);
        }
    }
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    for part in text.split_inclusive(['.', '!', '?', '。', '！', '？', '\n']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if !current.is_empty()
            && current.chars().count() + part.chars().count() + 1 > MAX_CHUNK
        {
            push(&current, &mut chunks);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(part);
    }
    push(&current, &mut chunks);
    if chunks.is_empty() && !text.trim().is_empty() {
        chunks.push(text.trim().to_owned());
    }
    chunks
}

/// One sentence through the sidecar: the WAV bytes back, or the reason
/// it did not answer.
fn vieneu_speak_one(port: u16, voice: &str, text: &str) -> Result<Vec<u8>, String> {
    let body = serde_json::json!({ "voice": voice, "text": text }).to_string();
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout_read(std::time::Duration::from_secs(300))
        .build();
    let mut reader = agent
        .post(&format!("http://127.0.0.1:{port}/speak"))
        .set("Content-Type", "application/json")
        .send_string(&body)
        .map_err(|error| format!("the VieNeu server did not answer: {error}"))?
        .into_reader();
    let mut wav = Vec::new();
    reader
        .read_to_end(&mut wav)
        .map_err(|error| format!("the VieNeu server's answer could not be read: {error}"))?;
    if wav.len() < 44 {
        return Err("the VieNeu server returned no audio".to_owned());
    }
    Ok(wav)
}

/// Joins the WAVs one read produced into a single WAV: every chunk is
/// the server's 48 kHz mono 16-bit, so only the samples are kept and one
/// header is written for all of them.
fn concat_wavs(chunks: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    let mut samples: Vec<u8> = Vec::new();
    let mut format: Option<(u16, u32, u16)> = None;
    for chunk in chunks {
        let (channels, rate, bits, data) = wav_samples(chunk)?;
        if format.is_some_and(|held| held != (channels, rate, bits)) {
            return Err("the VieNeu server changed audio format mid-read".to_owned());
        }
        format = Some((channels, rate, bits));
        samples.extend_from_slice(data);
    }
    let Some((channels, rate, bits)) = format else {
        return Err("the VieNeu server returned no audio".to_owned());
    };
    let mut wav = Vec::with_capacity(44 + samples.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + samples.len() as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&rate.to_le_bytes());
    let bytes_per_second = rate * u32::from(channels) * u32::from(bits) / 8;
    wav.extend_from_slice(&bytes_per_second.to_le_bytes());
    wav.extend_from_slice(&(channels * bits / 8).to_le_bytes());
    wav.extend_from_slice(&bits.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(samples.len() as u32).to_le_bytes());
    wav.extend_from_slice(&samples);
    Ok(wav)
}

/// A WAV's format and its sample bytes: the `fmt ` chunk, then the `data`
/// chunk, found by scanning rather than assumed at fixed offsets.
fn wav_samples(wav: &[u8]) -> Result<(u16, u32, u16, &[u8]), String> {
    if wav.len() < 44 || &wav[0..4] != b"RIFF" || &wav[8..12] != b"WAVE" {
        return Err("the VieNeu server returned something that is not a WAV".to_owned());
    }
    let mut offset = 12;
    let mut format = None;
    while offset + 8 <= wav.len() {
        let id = &wav[offset..offset + 4];
        let size =
            u32::from_le_bytes(wav[offset + 4..offset + 8].try_into().unwrap()) as usize;
        let start = offset + 8;
        let end = (start + size).min(wav.len());
        if id == b"fmt " && size >= 16 {
            let channels = u16::from_le_bytes(wav[start..start + 2].try_into().unwrap());
            let rate = u32::from_le_bytes(wav[start + 4..start + 8].try_into().unwrap());
            let bits = u16::from_le_bytes(wav[start + 14..start + 16].try_into().unwrap());
            format = Some((channels, rate, bits));
        } else if id == b"data" {
            let Some(format) = format else {
                return Err("the VieNeu server's WAV has data before its format".to_owned());
            };
            return Ok((format.0, format.1, format.2, &wav[start..end]));
        }
        offset = start + size + (size % 2);
    }
    Err("the VieNeu server's WAV has no audio in it".to_owned())
}

/// Seconds of audio in a WAV, from its own header.
fn wav_duration(wav: &[u8]) -> f64 {
    wav_samples(wav).map_or(0.0, |(channels, rate, bits, data)| {
        let bytes_per_second = f64::from(rate) * f64::from(channels) * f64::from(bits) / 8.0;
        data.len() as f64 / bytes_per_second.max(1.0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The VieNeu table holds the model's own 25 presets, in its order,
    /// with ids nothing else uses and the model's default among them.
    #[test]
    fn the_vieneu_table_holds_the_models_own_presets() {
        assert_eq!(VIENEU_VOICES.len(), 25);
        assert_eq!(VIENEU_VOICES[0].1, "Adam bựa");
        assert_eq!(VIENEU_VOICES[2].1, "Thiện Minh");
        assert_eq!(VIENEU_VOICES[4].1, "Hải Đăng");
        assert_eq!(VIENEU_VOICES[24].1, "Quốc Tuấn");
        assert_eq!(VIENEU_DEFAULT, 4004);
        let mut ids: Vec<i32> = VIENEU_VOICES.iter().map(|(id, _, _)| *id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 25);
        assert!(ids
            .iter()
            .all(|id| (VIENEU_BASE..VIENEU_BASE + 25).contains(id)));
        assert_eq!(family_of(VIENEU_ID), Family::VieNeu);
        assert_eq!(voice_family(4000), Some(Family::VieNeu));
        assert_eq!(voice_family(4024), Some(Family::VieNeu));
        assert_eq!(voice_name(4002), Some("Thiện Minh"));
        assert_eq!(
            vieneu_voice_detail("Thiện Minh"),
            Some("Nam · Bắc · kể chuyện")
        );
        assert_eq!(vieneu_voice_detail("af_heart"), None);
    }

    /// A VieNeu read is sent a sentence at a time: marks stay on their
    /// sentence, and a stretch with no mark at all is one read.
    #[test]
    fn vieneu_reads_are_split_into_sentences() {
        // Short sentences pack into one read while they fit the chunk.
        assert_eq!(
            sentence_chunks("Ngày xửa ngày xưa. Trong một ngôi làng nhỏ, ven rừng, có một chàng trai."),
            vec!["Ngày xửa ngày xưa. Trong một ngôi làng nhỏ, ven rừng, có một chàng trai."]
        );
        // Past the chunk, the second sentence goes on its own.
        assert_eq!(
            sentence_chunks("Ngày xửa ngày xưa, trong một ngôi làng nhỏ ven rừng, có một chàng trai trẻ mang trong mình một bí mật không ai ngờ tới. Chàng trai ấy mỗi sáng đều ra bờ suối gánh nước, chiều về lại ngồi bên hiên nhà nghe bà kể chuyện xưa."),
            vec![
                "Ngày xửa ngày xưa, trong một ngôi làng nhỏ ven rừng, có một chàng trai trẻ mang trong mình một bí mật không ai ngờ tới.",
                "Chàng trai ấy mỗi sáng đều ra bờ suối gánh nước, chiều về lại ngồi bên hiên nhà nghe bà kể chuyện xưa."
            ]
        );
        assert_eq!(sentence_chunks("no marks at all"), vec!["no marks at all"]);
        assert!(sentence_chunks("").is_empty());
        assert!(sentence_chunks("   ").is_empty());
    }

    /// The WAVs one read produced join into a single WAV at the same rate.
    #[test]
    fn vieneu_wavs_join_into_one() {
        fn wav(samples: &[i16]) -> Vec<u8> {
            let mut out = Vec::new();
            out.extend_from_slice(b"RIFF");
            out.extend_from_slice(&(36 + samples.len() as u32 * 2).to_le_bytes());
            out.extend_from_slice(b"WAVEfmt ");
            out.extend_from_slice(&16u32.to_le_bytes());
            out.extend_from_slice(&1u16.to_le_bytes());
            out.extend_from_slice(&1u16.to_le_bytes());
            out.extend_from_slice(&48000u32.to_le_bytes());
            out.extend_from_slice(&96000u32.to_le_bytes());
            out.extend_from_slice(&2u16.to_le_bytes());
            out.extend_from_slice(&16u16.to_le_bytes());
            out.extend_from_slice(b"data");
            out.extend_from_slice(&(samples.len() as u32 * 2).to_le_bytes());
            for sample in samples {
                out.extend_from_slice(&sample.to_le_bytes());
            }
            out
        }
        let joined = concat_wavs(&[wav(&[1, 2, 3]), wav(&[4, 5])]).unwrap();
        let (_, rate, _, data) = wav_samples(&joined).unwrap();
        assert_eq!(rate, 48000);
        assert_eq!(data.len(), 10);
        assert_eq!(wav_duration(&joined), 5.0 / 48000.0);
        assert!(concat_wavs(&[vec![0; 10]]).is_err());
        assert!(concat_wavs(&[]).is_err());
    }

    /// A read through a running sidecar returns a WAV: skipped when no
    /// server answers.
    #[test]
    fn a_read_through_a_running_sidecar_is_a_wav() {
        if !vieneu_health(VIENEU_PORT) {
            return;
        }
        let wav = vieneu_speak_one(VIENEU_PORT, "Hải Đăng", "Xin chào.").unwrap();
        let (_, rate, _, data) = wav_samples(&wav).unwrap();
        assert_eq!(rate, 48000);
        assert!(!data.is_empty());
    }

    /// A voice belongs to one family, and a model says which it reads
    /// with: only VieNeu's 25 presets belong to any family now.
    #[test]
    fn a_voice_belongs_to_one_family_and_a_model_says_which_it_reads_with() {
        assert_eq!(family_of(VIENEU_ID), Family::VieNeu);
        assert_eq!(family_of("vieneu-v3-turbo-int8"), Family::VieNeu);
        assert_eq!(voice_family(4000), Some(Family::VieNeu));
        assert_eq!(voice_family(4024), Some(Family::VieNeu));
        assert_eq!(voice_family(3000), None);
        assert_eq!(voice_family(3), None);
        assert_eq!(voice_family(53), None);
        assert_eq!(voice_family(-1), None);
        assert_eq!(voice_name(4000), Some("Adam bựa"));
        assert_eq!(voice_name(4024), Some("Quốc Tuấn"));
        assert_eq!(voice_name(999), None);
        assert_eq!(voice_name(3000), None);
        // Every preset has a name and belongs to VieNeu.
        for (id, name, _) in VIENEU_VOICES {
            assert_eq!(voice_name(*id), Some(*name));
            assert_eq!(voice_family(*id), Some(Family::VieNeu));
        }
        for model in KNOWN_MODELS {
            assert!(known(model.id).is_some());
            assert_eq!(family_of(model.id), Family::VieNeu);
            assert!(model.approx_bytes > 0);
        }
    }

    /// A voice preview is a short WAV on disk, named by the voice; a voice
    /// that belongs to no family refuses. Skipped wherever the VieNeu
    /// model is not on disk.
    #[test]
    fn a_voice_preview_is_a_short_wav_when_the_model_is_downloaded() {
        let tts = Speech::new();
        assert!(tts
            .preview_voice(
                &AppDirs::under(&std::env::temp_dir()),
                VIENEU_ID,
                999,
                "Xin chào."
            )
            .is_err());
        let Ok(dirs) = AppDirs::locate() else { return };
        if !model_downloaded(&dirs, VIENEU_ID) {
            return;
        }
        let file = tts
            .preview_voice(&dirs, VIENEU_ID, 4004, "Xin chào.")
            .expect("the preview reads");
        assert!(file.is_file());
        assert!(file.to_string_lossy().ends_with("voice-4004.wav"));
    }

    /// A whole Vietnamese read goes through when the model is downloaded.
    /// Skipped wherever it is not on disk.
    #[test]
    fn a_vietnamese_read_goes_through_when_the_model_is_downloaded() {
        let Ok(dirs) = AppDirs::locate() else { return };
        if !model_downloaded(&dirs, VIENEU_ID) {
            return;
        }
        let project =
            std::env::temp_dir().join(format!("concat-tts-project-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&project);
        std::fs::create_dir_all(&project).expect("project dir");
        std::fs::write(project.join("concat.json"), "{}").expect("manifest");
        let speech = Speech::new();
        let result = speech.speak(
            &dirs,
            &SpeakRequest {
                model_id: VIENEU_ID.to_owned(),
                voice: 4004,
                reference: None,
                text: "Xin chào Việt Nam".to_owned(),
                speed: 1.0,
                pauses: None,
                steps: None,
                project: project.to_string_lossy().into_owned(),
            },
            |_| {},
        );
        let summary = result.expect("the Vietnamese read goes through");
        assert!(std::path::Path::new(&summary.path).is_file());
        let _ = std::fs::remove_dir_all(&project);
    }
}
