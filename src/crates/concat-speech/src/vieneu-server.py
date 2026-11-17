# -*- coding: utf-8 -*-
"""VieNeu-TTS v3 Turbo sidecar for Proto Editor.

Runs the int8 ONNX engine on CPU and serves POST /speak. The editor
(concat-speech) starts this when a VieNeu read is requested, polls
GET /health until the engine is loaded, then reuses the server for the
whole session. Localhost only - never expose this port.

Why a sidecar: VieNeu-TTS is not a sherpa-onnx model, so the editor's
in-process TTS cannot load it. The Python SDK (vieneu 3.8.4) runs it
torch-free on CPU via ONNX Runtime instead.

Why the codec_dir patch: the SDK fetches model files one by one from the
Hugging Face blob cache, which splits them across directories - but the
ONNX graphs reference their external .data files by relative path, so
ONNX Runtime refuses to load them. The editor therefore keeps the 7
backbone files and the 6 codec files each in one real folder and passes
both folders in here (onnx_dir is a supported argument; codec_dir is
injected into the engine constructor, which the SDK's V3Turbo class
never forwards).
"""
import argparse
import os
import sys
import tempfile

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ap = argparse.ArgumentParser()
ap.add_argument("--port", type=int, default=8471)
ap.add_argument("--onnx-dir", required=True)
ap.add_argument("--codec-dir", required=True)
args = ap.parse_args()

from vieneu._v3_turbo_engine.onnx_runtime_lite import OnnxV3LiteEngine

_orig_init = OnnxV3LiteEngine.__init__


def _patched_init(self, *a, **k):
    k["codec_dir"] = args.codec_dir
    return _orig_init(self, *a, **k)


OnnxV3LiteEngine.__init__ = _patched_init

from vieneu import Vieneu
from fastapi import FastAPI
from fastapi.responses import Response

tts = Vieneu(precision="int8", onnx_dir=args.onnx_dir)
app = FastAPI()


@app.get("/health")
def health():
    return {"ok": True}


@app.get("/voices")
def voices():
    return [{"name": name, "label": label} for label, name in tts.list_preset_voices()]


@app.post("/speak")
def speak(body: dict):
    voice = body.get("voice") or "Hải Đăng"
    text = (body.get("text") or "").strip()
    if not text:
        return Response(content="empty text", status_code=400)
    audio = tts.infer(text, voice=voice)
    tmp = tempfile.NamedTemporaryFile(suffix=".wav", delete=False)
    tmp.close()
    try:
        tts.save(audio, tmp.name)
        with open(tmp.name, "rb") as f:
            data = f.read()
    finally:
        os.unlink(tmp.name)
    return Response(content=data, media_type="audio/wav")


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="127.0.0.1", port=args.port, log_level="warning")
