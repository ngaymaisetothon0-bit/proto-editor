// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Words into short caption cues, for AutoSub.
//!
//! Both functions are pure, so they are tested here without whisper: the
//! token walk lives in [`crate::transcribe`], which hands over plain
//! (text, t0, t1) triples, and the splitter takes finished [`Word`]s.
//!
//! A cue is what one title clip shows: a few words, short enough to read at
//! a glance and gone before the next idea starts - the TikTok caption shape
//! from the captions skill (1-3 words a line, about a second a cue, never
//! still for more than two seconds, never split mid-word).

use serde::Serialize;

use crate::transcribe::Word;

/// Most words a cue holds: three, so a line stays readable at a glance.
const MAX_WORDS: usize = 3;
/// Most characters a cue holds, counted in characters (not bytes - Vietnamese
/// is multi-byte): about what two lines of broadcast subtitle hold, the same
/// cap the captions sheet wraps at.
const MAX_CHARS: usize = 42;
/// Longest a cue may run, in seconds: past this a line has hung long enough
/// to feel still, even with karaoke moving inside it.
const MAX_SPAN: f64 = 2.0;
/// A silence this long between two words is a phrase break, not a pause
/// inside one.
const MAX_GAP: f64 = 0.5;
/// The cue's target length, in seconds: at about three words a second this
/// still packs the three-word cue (~0.9 s); slow speech breaks earlier.
/// A single slow word may run past it alone - it is a target, not a cap.
const SOFT_SPAN: f64 = 1.0;

/// One short caption line cut from words: what one title clip shows.
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Cue {
    /// Seconds from the window's start.
    pub start: f64,
    /// Seconds from the window's start.
    pub end: f64,
    /// The words, joined with single spaces.
    pub text: String,
    /// The words the line was cut from, with their timings - what the
    /// karaoke highlight reads to know when each word is spoken.
    #[serde(default)]
    pub words: Vec<Word>,
}

/// Whisper's tokens into words.
///
/// Tokens arrive as raw byte pieces, and the bytes join before anything is
/// decoded: a Vietnamese letter is several bytes, and whisper's BPE can
/// split it across two tokens. Decoding each token alone would turn both
/// halves into U+FFFD - the bytes must join first, then the whole word is
/// decoded once (`String::from_utf8_lossy`, so a stray bad byte still yields
/// a word instead of failing the run).
///
/// A token is one BPE piece: " Huy" starts a word, "ền" continues it, ","
/// sticks to the word before it. So a new word starts at a leading space
/// byte, after a whitespace-only token (a boundary with no text of its own),
/// or when no word is open; anything else appends to the current word.
/// Tokens whose trimmed bytes are empty, or that look special
/// (`[_TT_123]`, `[_SOT_]`, `[_EOT_]`, `[_BLANK_]`...), are skipped.
///
/// Times come in centiseconds, like the segment's own. A token with no
/// usable time (t0 < 0, or t1 <= t0 - whisper emits t1 == t0 on some
/// pieces) leaves its word untimed, and untimed words are filled by an even
/// share of the gap between their timed neighbours - the segment's edges at
/// the ends, the whole segment when nothing is timed. Times are clamped
/// into the segment and kept in order.
pub fn words_from_tokens(tokens: Vec<(Vec<u8>, i64, i64)>, seg_start: f64, seg_end: f64) -> Vec<Word> {
    // Group token bytes into words, tracking each word's valid bounds.
    let mut groups: Vec<(Vec<Vec<u8>>, Vec<(f64, f64)>)> = Vec::new();
    let mut cur: Option<(Vec<Vec<u8>>, Vec<(f64, f64)>)> = None;
    let mut boundary = false;
    for (bytes, t0, t1) in tokens {
        if !bytes.is_empty() && bytes.iter().all(|byte| byte.is_ascii_whitespace()) {
            // An all-whitespace token: a word boundary, no text of its own.
            if let Some(group) = cur.take() {
                groups.push(group);
            }
            boundary = true;
            continue;
        }
        let trimmed = bytes.trim_ascii();
        if trimmed.is_empty() || trimmed.starts_with(b"[_") {
            continue;
        }
        let bounds = if t0 >= 0 && t1 > t0 {
            Some((t0 as f64 / 100.0, t1 as f64 / 100.0))
        } else {
            None
        };
        let starts_word = cur.is_none() || boundary || bytes.first() == Some(&b' ');
        if starts_word {
            if let Some(group) = cur.take() {
                groups.push(group);
            }
            cur = Some((vec![bytes], bounds.into_iter().collect()));
        } else {
            let group = cur.as_mut().expect("a word is open for a continuation");
            group.0.push(bytes);
            if let Some(bound) = bounds {
                group.1.push(bound);
            }
        }
        boundary = false;
    }
    if let Some(group) = cur.take() {
        groups.push(group);
    }

    let mut words: Vec<Word> = Vec::new();
    let mut timed: Vec<bool> = Vec::new();
    for (parts, bounds) in groups {
        // Decode once, on the joined bytes: a letter split across two
        // tokens comes back whole.
        let joined: Vec<u8> = parts.concat();
        let text = String::from_utf8_lossy(&joined).trim().to_owned();
        if text.is_empty() {
            continue;
        }
        if !bounds.is_empty() {
            let start = bounds.iter().map(|bound| bound.0).fold(f64::INFINITY, f64::min);
            let end = bounds.iter().map(|bound| bound.1).fold(f64::NEG_INFINITY, f64::max);
            if end > start {
                words.push(Word { start, end, text });
                timed.push(true);
                continue;
            }
        }
        words.push(Word { start: 0.0, end: 0.0, text });
        timed.push(false);
    }

    // Fill untimed words by an even share of the gap between timed anchors.
    let n = words.len();
    if n == 0 {
        return words;
    }
    let timed_idx: Vec<usize> =
        timed.iter().enumerate().filter_map(|(i, t)| t.then_some(i)).collect();
    if timed_idx.is_empty() {
        let step = (seg_end - seg_start) / n as f64;
        for (i, word) in words.iter_mut().enumerate() {
            word.start = seg_start + step * i as f64;
            word.end = seg_start + step * (i + 1) as f64;
        }
    } else {
        let first = timed_idx[0];
        if first > 0 {
            let step = (words[first].start - seg_start) / first as f64;
            for i in 0..first {
                words[i].start = seg_start + step * i as f64;
                words[i].end = seg_start + step * (i + 1) as f64;
            }
        }
        for pair in timed_idx.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if b - a > 1 {
                // Split among the untimed words between, not counting the anchors.
                let step = (words[b].start - words[a].end) / (b - a - 1) as f64;
                for i in (a + 1)..b {
                    words[i].start = words[a].end + step * (i - a - 1) as f64;
                    words[i].end = words[a].end + step * (i - a) as f64;
                }
            }
        }
        let last = *timed_idx.last().expect("a timed word exists");
        if last + 1 < n {
            let step = (seg_end - words[last].end) / (n - 1 - last) as f64;
            for i in (last + 1)..n {
                words[i].start = words[last].end + step * (i - last - 1) as f64;
                words[i].end = words[last].end + step * (i - last) as f64;
            }
        }
    }
    // Clamp into the segment and keep the order: a filled time never runs
    // before the word ahead of it.
    let mut prev_end = seg_start;
    for word in &mut words {
        word.start = word.start.max(prev_end).min(seg_end);
        word.end = word.end.max(word.start).min(seg_end);
        prev_end = word.end;
    }
    words
}

/// Words into short cues, in time order.
///
/// Greedy: keep adding words while the cue holds at most [`MAX_WORDS`]
/// words, [`MAX_CHARS`] characters, [`MAX_SPAN`] seconds, and the gap to the
/// next word stays under [`MAX_GAP`] - and break after a sentence-ending
/// word, or when the cue would run past the [`SOFT_SPAN`] target with two
/// words already in it. A word longer than every cap still gets its own cue:
/// a cue is never split mid-word.
pub fn words_to_cues(words: &[Word]) -> Vec<Cue> {
    if words.is_empty() {
        return Vec::new();
    }
    let mut runs: Vec<Vec<usize>> = Vec::new();
    let mut cur: Vec<usize> = vec![0];
    for i in 1..words.len() {
        let prev = &words[*cur.last().expect("a word is in the cue")];
        let text: String = cur
            .iter()
            .map(|&j| words[j].text.as_str())
            .chain(std::iter::once(words[i].text.as_str()))
            .collect::<Vec<_>>()
            .join(" ");
        let span = words[i].end - words[cur[0]].start;
        let gap = words[i].start - prev.end;
        let ends_sentence = prev
            .text
            .chars()
            .last()
            .is_some_and(|c| matches!(c, '.' | '!' | '?' | '…'));
        if cur.len() >= MAX_WORDS
            || text.chars().count() > MAX_CHARS
            || span > MAX_SPAN
            || gap > MAX_GAP
            || ends_sentence
            || (span > SOFT_SPAN && cur.len() >= 2)
        {
            runs.push(std::mem::take(&mut cur));
            cur = vec![i];
        } else {
            cur.push(i);
        }
    }
    runs.push(cur);
    runs
        .into_iter()
        .map(|run| Cue {
            start: words[run[0]].start,
            end: words[*run.last().expect("a word is in the cue")].end,
            text: run.iter().map(|&j| words[j].text.as_str()).collect::<Vec<_>>().join(" "),
            words: run.iter().map(|&j| words[j].clone()).collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(text: &str, start: f64, end: f64) -> Word {
        Word { start, end, text: text.to_owned() }
    }

    fn tokens(pieces: &[(&str, i64, i64)]) -> Vec<(Vec<u8>, i64, i64)> {
        pieces.iter().map(|(text, t0, t1)| (text.as_bytes().to_vec(), *t0, *t1)).collect()
    }

    /// Cues compare with a 1e-9 tolerance on their times: cue ends come from
    /// word ends, and word ends come from float arithmetic (the round-14
    /// float lesson - never `==` on a computed float). Text still compares
    /// exactly.
    fn assert_cues_eq(got: &[Cue], want: &[Cue]) {
        assert_eq!(got.len(), want.len(), "different cue counts: {got:?} vs {want:?}");
        for (got, want) in got.iter().zip(want.iter()) {
            assert!((got.start - want.start).abs() < 1e-9, "cue start differs: {got:?} vs {want:?}");
            assert!((got.end - want.end).abs() < 1e-9, "cue end differs: {got:?} vs {want:?}");
            assert_eq!(got.text, want.text, "cue text differs: {got:?} vs {want:?}");
            assert_eq!(got.words.len(), want.words.len(), "different word counts: {got:?} vs {want:?}");
            for (gw, ww) in got.words.iter().zip(want.words.iter()) {
                assert!((gw.start - ww.start).abs() < 1e-9, "word start differs: {got:?} vs {want:?}");
                assert!((gw.end - ww.end).abs() < 1e-9, "word end differs: {got:?} vs {want:?}");
                assert_eq!(gw.text, ww.text, "word text differs: {got:?} vs {want:?}");
            }
        }
    }

    #[test]
    fn tokens_group_into_words() {
        assert_eq!(
            words_from_tokens(tokens(&[(" Hello", 0, 50), (" world", 50, 120)]), 0.0, 1.2),
            vec![word("Hello", 0.0, 0.5), word("world", 0.5, 1.2)]
        );
    }

    #[test]
    fn subword_pieces_join_into_one_word() {
        assert_eq!(
            words_from_tokens(tokens(&[(" Huy", 0, 30), ("ền", 30, 60), (" Ấn", 60, 100)]), 0.0, 1.0),
            vec![word("Huyền", 0.0, 0.6), word("Ấn", 0.6, 1.0)]
        );
    }

    #[test]
    fn punctuation_sticks_to_the_word_before_it() {
        assert_eq!(
            words_from_tokens(tokens(&[(" Ấn", 0, 50), (",", 50, 60), (" đây", 60, 100)]), 0.0, 1.0),
            vec![word("Ấn,", 0.0, 0.6), word("đây", 0.6, 1.0)]
        );
    }

    #[test]
    fn special_tokens_are_skipped() {
        assert_eq!(
            words_from_tokens(tokens(&[("[_TT_123]", 0, 0), (" Hello", 0, 50), ("[_EOT_]", -1, -1)]), 0.0, 0.5),
            vec![word("Hello", 0.0, 0.5)]
        );
    }

    #[test]
    fn a_whitespace_token_is_a_word_boundary() {
        assert_eq!(
            words_from_tokens(tokens(&[("Hello", 0, 50), (" ", 50, 55), ("world", 55, 100)]), 0.0, 1.0),
            vec![word("Hello", 0.0, 0.5), word("world", 0.55, 1.0)]
        );
    }

    #[test]
    fn untimed_words_share_the_segment_evenly() {
        assert_eq!(
            words_from_tokens(tokens(&[(" Hello", -1, -1), (" world", -1, -1)]), 0.0, 2.0),
            vec![word("Hello", 0.0, 1.0), word("world", 1.0, 2.0)]
        );
    }

    #[test]
    fn an_untimed_word_fills_the_gap_between_timed_ones() {
        assert_eq!(
            words_from_tokens(tokens(&[(" A", 0, 50), (" B", -1, -1), (" C", 100, 150)]), 0.0, 1.5),
            vec![word("A", 0.0, 0.5), word("B", 0.5, 1.0), word("C", 1.0, 1.5)]
        );
    }

    #[test]
    fn a_zero_length_token_has_no_time() {
        assert_eq!(
            words_from_tokens(tokens(&[(" A", 50, 50), (" B", 100, 150)]), 0.0, 1.5),
            vec![word("A", 0.0, 1.0), word("B", 1.0, 1.5)]
        );
    }

    #[test]
    fn a_vietnamese_letter_split_across_two_tokens_survives() {
        // "ế" (U+1EBF) is three bytes; whisper's BPE can split it across two
        // tokens. Decoding each token alone would turn both halves into
        // U+FFFD - the bytes must join before the word is decoded. This is
        // the exact failure the real-audio test caught on S2.wav.
        let tokens = vec![
            (b" ti".to_vec(), 0, 30),
            (vec![0xE1, 0xBA], 30, 45),
            (vec![0xBF, b'n', b'g'], 45, 60),
        ];
        assert_eq!(
            words_from_tokens(tokens, 0.0, 0.6),
            vec![word("tiếng", 0.0, 0.6)]
        );
    }

    #[test]
    fn no_tokens_means_no_words() {
        assert!(words_from_tokens(Vec::new(), 0.0, 1.0).is_empty());
    }

    #[test]
    fn empty_and_untimed_edges_fill_from_the_segment() {
        assert_eq!(
            words_from_tokens(tokens(&[("", 0, 10), (" Hi", 10, 50)]), 0.0, 0.5),
            vec![word("Hi", 0.1, 0.5)]
        );
        assert_eq!(
            words_from_tokens(tokens(&[(" A", -1, -1), (" B", 50, 100)]), 0.0, 1.0),
            vec![word("A", 0.0, 0.5), word("B", 0.5, 1.0)]
        );
        assert_eq!(
            words_from_tokens(tokens(&[(" A", 0, 50), (" B", -1, -1)]), 0.0, 1.0),
            vec![word("A", 0.0, 0.5), word("B", 0.5, 1.0)]
        );
    }

    #[test]
    fn three_words_per_cue() {
        // 3.0 * 0.3 is 0.8999999999999999 in float, not 0.9 - hence the
        // tolerance compare (see `assert_cues_eq`).
        let words: Vec<Word> =
            (0..6).map(|i| word(&format!("w{i}"), i as f64 * 0.3, (i + 1) as f64 * 0.3)).collect();
        assert_cues_eq(
            &words_to_cues(&words),
            &[
                Cue {
                    start: 0.0,
                    end: 0.9,
                    text: "w0 w1 w2".to_owned(),
                    words: vec![word("w0", 0.0, 0.3), word("w1", 0.3, 0.6), word("w2", 0.6, 0.9)],
                },
                Cue {
                    start: 0.9,
                    end: 1.8,
                    text: "w3 w4 w5".to_owned(),
                    words: vec![word("w3", 0.9, 1.2), word("w4", 1.2, 1.5), word("w5", 1.5, 1.8)],
                },
            ],
        );
    }

    #[test]
    fn a_cue_holds_at_most_42_characters() {
        let words = vec![
            word("abcdefghij", 0.0, 0.3),
            word("klmnopqrst", 0.3, 0.6),
            word("uvwxyzabcd", 0.6, 0.9),
            word("efghijklmn", 0.9, 1.2),
        ];
        assert_eq!(
            words_to_cues(&words),
            vec![
                Cue {
                    start: 0.0,
                    end: 0.9,
                    text: "abcdefghij klmnopqrst uvwxyzabcd".to_owned(),
                    words: vec![
                        word("abcdefghij", 0.0, 0.3),
                        word("klmnopqrst", 0.3, 0.6),
                        word("uvwxyzabcd", 0.6, 0.9),
                    ],
                },
                Cue {
                    start: 0.9,
                    end: 1.2,
                    text: "efghijklmn".to_owned(),
                    words: vec![word("efghijklmn", 0.9, 1.2)],
                },
            ]
        );
    }

    #[test]
    fn a_sentence_end_breaks_the_cue() {
        let words = vec![
            word("Hello", 0.0, 0.3),
            word("world.", 0.3, 0.6),
            word("Next", 0.6, 0.9),
            word("day", 0.9, 1.2),
        ];
        assert_eq!(
            words_to_cues(&words),
            vec![
                Cue {
                    start: 0.0,
                    end: 0.6,
                    text: "Hello world.".to_owned(),
                    words: vec![word("Hello", 0.0, 0.3), word("world.", 0.3, 0.6)],
                },
                Cue {
                    start: 0.6,
                    end: 1.2,
                    text: "Next day".to_owned(),
                    words: vec![word("Next", 0.6, 0.9), word("day", 0.9, 1.2)],
                },
            ]
        );
    }

    #[test]
    fn a_pause_breaks_the_cue() {
        let words = vec![word("a", 0.0, 0.3), word("b", 0.9, 1.2), word("c", 1.2, 1.5)];
        assert_eq!(
            words_to_cues(&words),
            vec![
                Cue {
                    start: 0.0,
                    end: 0.3,
                    text: "a".to_owned(),
                    words: vec![word("a", 0.0, 0.3)],
                },
                Cue {
                    start: 0.9,
                    end: 1.5,
                    text: "b c".to_owned(),
                    words: vec![word("b", 0.9, 1.2), word("c", 1.2, 1.5)],
                },
            ]
        );
    }

    #[test]
    fn a_long_word_is_never_split() {
        let words = vec![
            word("supercalifragilisticexpialidocious", 0.0, 2.5),
            word("x", 2.5, 2.8),
        ];
        assert_eq!(
            words_to_cues(&words),
            vec![
                Cue {
                    start: 0.0,
                    end: 2.5,
                    text: "supercalifragilisticexpialidocious".to_owned(),
                    words: vec![word("supercalifragilisticexpialidocious", 0.0, 2.5)],
                },
                Cue {
                    start: 2.5,
                    end: 2.8,
                    text: "x".to_owned(),
                    words: vec![word("x", 2.5, 2.8)],
                },
            ]
        );
    }

    #[test]
    fn a_cue_aims_for_about_a_second() {
        let words = vec![word("a", 0.0, 0.4), word("b", 0.4, 0.8), word("c", 0.8, 1.2)];
        assert_eq!(
            words_to_cues(&words),
            vec![
                Cue {
                    start: 0.0,
                    end: 0.8,
                    text: "a b".to_owned(),
                    words: vec![word("a", 0.0, 0.4), word("b", 0.4, 0.8)],
                },
                Cue {
                    start: 0.8,
                    end: 1.2,
                    text: "c".to_owned(),
                    words: vec![word("c", 0.8, 1.2)],
                },
            ]
        );
    }

    #[test]
    fn no_words_means_no_cues() {
        assert!(words_to_cues(&[]).is_empty());
        assert_eq!(
            words_to_cues(&[word("only", 0.0, 0.4)]),
            vec![Cue {
                start: 0.0,
                end: 0.4,
                text: "only".to_owned(),
                words: vec![word("only", 0.0, 0.4)],
            }]
        );
    }
}
