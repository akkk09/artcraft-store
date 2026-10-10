//! Audio decoding, WAV parser, and synthetic audio generation for QuietCraft.
//!
//! Provides a pure-Rust RIFF WAV reader (16-bit PCM, 24-bit PCM, 32-bit float)
//! and sample converters for feeding the silence detection engine.

use std::io::{Cursor, Read};

/// Decoded audio buffer containing normalized mono float samples (-1.0 .. 1.0).
#[derive(Debug, Clone, PartialEq)]
pub struct AudioBuffer {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>, // mono downmixed
    pub duration: f64,
}

impl AudioBuffer {
    pub fn new(sample_rate: u32, channels: u16, samples: Vec<f32>) -> Self {
        let dur = if sample_rate > 0 {
            samples.len() as f64 / sample_rate as f64
        } else {
            0.0
        };
        Self {
            sample_rate,
            channels,
            samples,
            duration: dur,
        }
    }
}

/// Parse a standard RIFF WAV byte slice into an `AudioBuffer`.
pub fn parse_wav(bytes: &[u8]) -> Result<AudioBuffer, String> {
    if bytes.len() < 44 {
        return Err("WAV data too short (less than 44 bytes)".to_string());
    }

    let mut cursor = Cursor::new(bytes);
    let mut header = [0u8; 12];
    cursor
        .read_exact(&mut header)
        .map_err(|e| format!("Failed to read RIFF header: {e}"))?;

    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
        return Err("Invalid WAV signature: expected RIFF/WAVE".to_string());
    }

    let mut audio_format = 1u16; // 1 = PCM, 3 = IEEE Float
    let mut num_channels = 1u16;
    let mut sample_rate = 44100u32;
    let mut bits_per_sample = 16u16;
    let mut data_bytes = Vec::new();

    // Iterate through chunks
    while (cursor.position() as usize) + 8 <= bytes.len() {
        let mut chunk_id = [0u8; 4];
        let mut chunk_size_bytes = [0u8; 4];
        if cursor.read_exact(&mut chunk_id).is_err() || cursor.read_exact(&mut chunk_size_bytes).is_err() {
            break;
        }
        let chunk_size = u32::from_le_bytes(chunk_size_bytes) as usize;

        if &chunk_id == b"fmt " {
            if chunk_size < 16 {
                return Err("Corrupt fmt chunk: smaller than 16 bytes".to_string());
            }
            let mut fmt_data = vec![0u8; chunk_size];
            cursor
                .read_exact(&mut fmt_data)
                .map_err(|e| format!("Failed to read fmt chunk: {e}"))?;

            audio_format = u16::from_le_bytes([fmt_data[0], fmt_data[1]]);
            num_channels = u16::from_le_bytes([fmt_data[2], fmt_data[3]]);
            sample_rate = u32::from_le_bytes([fmt_data[4], fmt_data[5], fmt_data[6], fmt_data[7]]);
            bits_per_sample = u16::from_le_bytes([fmt_data[14], fmt_data[15]]);
        } else if &chunk_id == b"data" {
            let available = bytes.len() - (cursor.position() as usize);
            let read_size = chunk_size.min(available);
            data_bytes.resize(read_size, 0);
            cursor
                .read_exact(&mut data_bytes)
                .map_err(|e| format!("Failed to read data chunk: {e}"))?;
            // Padding byte if chunk size is odd
            if chunk_size % 2 == 1 && (cursor.position() as usize) < bytes.len() {
                let _ = cursor.read_exact(&mut [0u8; 1]);
            }
            break; // Finished reading data
        } else {
            // Skip unknown chunk
            let new_pos = cursor.position() + (chunk_size as u64);
            cursor.set_position(new_pos);
            if chunk_size % 2 == 1 {
                cursor.set_position(cursor.position() + 1);
            }
        }
    }

    if data_bytes.is_empty() {
        return Err("No audio data chunk found in WAV".to_string());
    }

    if num_channels == 0 {
        return Err("Zero audio channels specified in WAV".to_string());
    }

    // Convert raw PCM/Float bytes into mono downmixed f32 samples
    let mono_samples = decode_pcm_to_mono(&data_bytes, audio_format, num_channels, bits_per_sample)?;
    Ok(AudioBuffer::new(sample_rate, num_channels, mono_samples))
}

fn decode_pcm_to_mono(
    data: &[u8],
    audio_format: u16,
    channels: u16,
    bits_per_sample: u16,
) -> Result<Vec<f32>, String> {
    let ch = channels as usize;
    let mut mono = Vec::new();

    if audio_format == 1 {
        // Integer PCM
        match bits_per_sample {
            16 => {
                let sample_count = data.len() / 2;
                let frame_count = sample_count / ch;
                mono.reserve(frame_count);
                for frame in 0..frame_count {
                    let mut sum = 0.0f32;
                    for c in 0..ch {
                        let idx = (frame * ch + c) * 2;
                        let s = i16::from_le_bytes([data[idx], data[idx + 1]]) as f32 / 32768.0;
                        sum += s;
                    }
                    mono.push(sum / (ch as f32));
                }
            }
            24 => {
                let sample_count = data.len() / 3;
                let frame_count = sample_count / ch;
                mono.reserve(frame_count);
                for frame in 0..frame_count {
                    let mut sum = 0.0f32;
                    for c in 0..ch {
                        let idx = (frame * ch + c) * 3;
                        let s = (i32::from_le_bytes([0, data[idx], data[idx + 1], data[idx + 2]]) >> 8)
                            as f32
                            / 8388608.0;
                        sum += s;
                    }
                    mono.push(sum / (ch as f32));
                }
            }
            8 => {
                let sample_count = data.len();
                let frame_count = sample_count / ch;
                mono.reserve(frame_count);
                for frame in 0..frame_count {
                    let mut sum = 0.0f32;
                    for c in 0..ch {
                        let idx = frame * ch + c;
                        let s = (data[idx] as f32 - 128.0) / 128.0;
                        sum += s;
                    }
                    mono.push(sum / (ch as f32));
                }
            }
            other => return Err(format!("Unsupported PCM bit depth: {other}-bit")),
        }
    } else if audio_format == 3 {
        // IEEE Float
        if bits_per_sample == 32 {
            let sample_count = data.len() / 4;
            let frame_count = sample_count / ch;
            mono.reserve(frame_count);
            for frame in 0..frame_count {
                let mut sum = 0.0f32;
                for c in 0..ch {
                    let idx = (frame * ch + c) * 4;
                    let s = f32::from_le_bytes([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]]);
                    sum += s;
                }
                mono.push(sum / (ch as f32));
            }
        } else {
            return Err(format!("Unsupported IEEE float bit depth: {bits_per_sample}-bit"));
        }
    } else {
        return Err(format!("Unsupported WAV audio format code: {audio_format}"));
    }

    Ok(mono)
}

/// Create a 16-bit mono PCM WAV file in memory (useful for testing or exporting synthesized audio).
pub fn create_test_wav(sample_rate: u32, samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::new();
    let num_samples = samples.len() as u32;
    let byte_rate = sample_rate * 2;
    let data_size = num_samples * 2;
    let file_size = 36 + data_size;

    // RIFF Header
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&file_size.to_le_bytes());
    out.extend_from_slice(b"WAVE");

    // fmt chunk
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // Chunk size
    out.extend_from_slice(&1u16.to_le_bytes());  // Audio format 1 = PCM
    out.extend_from_slice(&1u16.to_le_bytes());  // 1 channel
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());  // Block align
    out.extend_from_slice(&16u16.to_le_bytes()); // Bits per sample

    // data chunk
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_size.to_le_bytes());

    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let val = (clamped * 32767.0) as i16;
        out.extend_from_slice(&val.to_le_bytes());
    }

    out
}

/// Generate synthetic speech-like bursts and silence gaps for unit and integration testing.
/// `intervals`: slice of (duration_seconds, amplitude_0_to_1)
pub fn generate_synthetic_audio(sample_rate: u32, intervals: &[(f64, f32)]) -> Vec<f32> {
    let mut samples = Vec::new();
    let freq = 220.0f64; // Fundamental frequency (Hz)
    let mut phase = 0.0f64;

    for &(dur, amp) in intervals {
        let count = (dur * sample_rate as f64).round() as usize;
        for _ in 0..count {
            if amp > 0.0001 {
                let val = (phase * 2.0 * std::f64::consts::PI).sin() as f32 * amp;
                samples.push(val);
                phase += freq / (sample_rate as f64);
                if phase > 1.0 {
                    phase -= 1.0;
                }
            } else {
                samples.push(0.0);
            }
        }
    }

    samples
}
