# Proto Editor

> Proto Editor is a free, open-source video editor for Windows, built for Vietnamese creators.
> Fork of [jub0t/Concat](https://github.com/jub0t/Concat) (AGPL-3.0), developed toward CapCut parity.

Proto Editor là trình biên tập video mã nguồn mở, fork từ [Concat](https://github.com/jub0t/Concat) của jub0t (giấy phép AGPL-3.0), đang được phát triển theo hướng tương đương CapCut.

## Tính năng

- **AutoSub** — tự động tạo phụ đề từ giọng nói (Whisper, chạy local), chia cue theo từ, highlight karaoke từng từ, 9:16 safe frame, font tiếng Việt đầy đủ dấu.
- **Giọng đọc tiếng Việt VieNeu** — TTS tiếng Việt giọng Bắc (mặc định), chỉ giữ lại VieNeu, các model giọng cũ đã được gỡ bỏ.
- **700+ hiệu ứng / transition / text / sticker** — filter điện ảnh, wipe, dissolve, chữ động, sticker vector và PNG trong suốt.
- **Tách nền** — AI Cutout, Chroma Key, Luma Key (đang phát triển).
- **Timeline kiểu CapCut** — Alt + lăn chuột để cuộn ngang, kéo thả clip, keyframe.
- **Xuất video** — H.264/H.265 qua FFmpeg, xuất 9:16 cho TikTok/Reels/Shorts.

## Cài đặt (dùng bản dựng sẵn)

Tải bản release mới nhất (file `.zip`, gồm app + FFmpeg + model Whisper) từ mục **Releases** hoặc từ Google Drive (xem link trong phần mô tả repo). Giải nén và chạy `Proto Editor.exe`. Không cần cài đặt.

Lưu ý: lần đầu dùng giọng VieNeu, app sẽ tải model từ Hugging Face (cần mạng). Windows SmartScreen có thể cảnh báo vì app chưa được ký — chọn "More info" → "Run anyway".

## Build từ source

Yêu cầu: Rust toolchain (stable), Windows 10/11.

```powershell
git clone https://github.com/ngaymaisetothon0-bit/proto-editor.git
cd proto-editor
cargo build --release
```

File chạy nằm tại `target/release/concat.exe` (đổi tên thành `Proto Editor.exe` khi đóng gói).

### File assets nhị phân (bắt buộc để build)

Repo này chỉ chứa file text (do giới hạn của GitHub). Các file nhị phân — font chữ, icon, model tách nền — nằm trong file riêng:

**[proto-editor-assets.zip (Drive)](https://drive.google.com/file/d/1hXczxQAVZWpC3Wtj82JFa29UwUai-2Ai/view?usp=drivesdk)**

Tải về, giải nén **vào đúng thư mục gốc của repo** (giữ nguyên cấu trúc thư mục bên trong zip), rồi mới `cargo build`. Thiếu các file này, build sẽ lỗi (font nhúng) hoặc tính năng tách nền không chạy (model ONNX).

## Giấy phép

Proto Editor được phát hành theo **GNU Affero General Public License v3.0 (AGPL-3.0)** — xem file [LICENSE](LICENSE). Bạn được tự do dùng, sửa, chia sẻ, nhưng mọi bản sửa đổi công khai đều phải giữ giấy phép AGPL-3.0 và công bố source.

Font chữ kèm theo (Inter, Fraunces, Be Vietnam Pro, ...) theo giấy phép SIL Open Font License 1.1. Xem [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Ghi công

Dựa trên [Concat](https://github.com/jub0t/Concat) của jub0t — cảm ơn tác giả gốc và cộng đồng mã nguồn mở.
