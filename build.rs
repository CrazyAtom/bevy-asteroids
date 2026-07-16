use std::f32::consts::TAU;
use std::fs;
use std::io::Write;
use std::path::Path;

const SR: u32 = 22_050;

fn main() {
    let dir = Path::new("assets/sounds");
    fs::create_dir_all(dir).unwrap();
    // (이름, 지속시간초, 생성기)
    write_wav(dir, "fire.wav", synth(0.15, |t| 600.0 - 400.0 * t / 0.15, 0.4, false));
    write_wav(dir, "explosion.wav", synth(0.4, |_| 0.0, 0.6, true));
    write_wav(dir, "ufo_fire.wav", synth(0.2, |t| 300.0 + 200.0 * (t * 30.0).sin(), 0.3, false));
    write_wav(dir, "pickup.wav", synth(0.2, |t| 500.0 + 600.0 * t / 0.2, 0.3, false));
    write_wav(dir, "special.wav", synth(0.5, |t| 200.0 + 100.0 * (t * 8.0).sin(), 0.5, true));
    write_wav(dir, "hyperspace.wav", synth(0.3, |t| 800.0 - 700.0 * t / 0.3, 0.3, false));
    write_wav(dir, "game_over.wav", synth(0.6, |t| 300.0 - 200.0 * t / 0.6, 0.4, false));

    // ── 배경 음악: 절차적 칩튠 루프(다성 시퀀서) ──
    let music_dir = Path::new("assets/music");
    fs::create_dir_all(music_dir).unwrap();
    write_music(music_dir, "title.wav", title_track());

    // ── 스프라이트: assets/sprites/src/*.svg → assets/sprites/*.png ──
    rasterize_sprites();
    println!("cargo:rerun-if-changed=assets/sprites/src");
    println!("cargo:rerun-if-changed=build.rs");
}

const SPRITE_SCALE: f32 = 4.0; // SVG viewBox 대비 렌더 배율(선명도)

/// assets/sprites/src 의 모든 SVG를 viewBox의 SPRITE_SCALE배 해상도 PNG로 굽는다.
fn rasterize_sprites() {
    use resvg::{tiny_skia, usvg};
    let src = Path::new("assets/sprites/src");
    let out = Path::new("assets/sprites");
    if !src.exists() {
        return; // 소스 없으면 조용히 통과
    }
    fs::create_dir_all(out).unwrap();
    let opt = usvg::Options::default();
    for entry in fs::read_dir(src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("svg") {
            continue;
        }
        let data = fs::read(&path).unwrap();
        let tree = usvg::Tree::from_data(&data, &opt).unwrap();
        let size = tree.size();
        let w = (size.width() * SPRITE_SCALE).ceil() as u32;
        let h = (size.height() * SPRITE_SCALE).ceil() as u32;
        let mut pixmap = tiny_skia::Pixmap::new(w, h).unwrap();
        let ts = tiny_skia::Transform::from_scale(SPRITE_SCALE, SPRITE_SCALE);
        resvg::render(&tree, ts, &mut pixmap.as_mut());
        let stem = path.file_stem().unwrap().to_str().unwrap();
        pixmap.save_png(out.join(format!("{stem}.png"))).unwrap();
    }
}

/// dur초 동안 freq(t)Hz 톤(+옵션 노이즈)에 선형 감쇠 엔벨로프를 씌운 PCM(i16) 샘플 생성.
fn synth(dur: f32, freq: impl Fn(f32) -> f32, amp: f32, noise: bool) -> Vec<i16> {
    let n = (SR as f32 * dur) as usize;
    let mut out = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    let mut seed = 0x1234_5678u32;
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let env = (1.0 - t / dur).max(0.0);
        let f = freq(t);
        phase += TAU * f / SR as f32;
        let tone = phase.sin();
        let sample = if noise {
            // xorshift 노이즈
            seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5;
            let nz = (seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
            0.5 * tone + 0.5 * nz
        } else {
            tone
        };
        out.push((sample * env * amp * i16::MAX as f32) as i16);
    }
    out
}

fn write_wav(dir: &Path, name: &str, samples: Vec<i16>) {
    let path = dir.join(name);
    let data_len = (samples.len() * 2) as u32;
    let mut f = fs::File::create(&path).unwrap();
    // RIFF/WAVE 헤더 (PCM, mono, 16-bit)
    f.write_all(b"RIFF").unwrap();
    f.write_all(&(36 + data_len).to_le_bytes()).unwrap();
    f.write_all(b"WAVEfmt ").unwrap();
    f.write_all(&16u32.to_le_bytes()).unwrap();       // fmt chunk size
    f.write_all(&1u16.to_le_bytes()).unwrap();        // PCM
    f.write_all(&1u16.to_le_bytes()).unwrap();        // mono
    f.write_all(&SR.to_le_bytes()).unwrap();          // sample rate
    f.write_all(&(SR * 2).to_le_bytes()).unwrap();    // byte rate
    f.write_all(&2u16.to_le_bytes()).unwrap();        // block align
    f.write_all(&16u16.to_le_bytes()).unwrap();       // bits per sample
    f.write_all(b"data").unwrap();
    f.write_all(&data_len.to_le_bytes()).unwrap();
    for s in samples {
        f.write_all(&s.to_le_bytes()).unwrap();
    }
}

// ── 절차적 칩튠 음악 시퀀서 ─────────────────────────────────────────────
// 여러 보이스(베이스·아르페지오·리드)를 마디 단위로 렌더·믹싱해 심리스 루프 WAV를
// 만든다. 노트별 엔벨로프(어택→서스테인→릴리스)가 경계에서 0에 수렴해 클릭과
// 루프 이음새 잡음을 없앤다.

/// 한 음: 반음 오프셋(A4=0 기준) + 길이(비트). `semi == REST`면 쉼표.
#[derive(Clone, Copy)]
struct Note {
    semi: i32,
    beats: f32,
}
const REST: i32 = i32::MIN;

fn n(semi: i32, beats: f32) -> Note {
    Note { semi, beats }
}

#[derive(Clone, Copy)]
enum Wave {
    Pulse(f32), // duty(0~1): 칩튠 사각/펄스파
    Triangle,   // 둥근 저음
}

struct Voice<'a> {
    notes: &'a [Note],
    wave: Wave,
    octave: i32, // 옥타브 오프셋(±12 semitone 단위)
    amp: f32,
}

/// A4=440Hz 기준 반음 오프셋 → 주파수.
fn note_hz(semi: i32) -> f32 {
    440.0 * 2f32.powf(semi as f32 / 12.0)
}

/// 위상(사이클 단위) → 펄스파. phase.fract() < duty 이면 +1, 아니면 -1.
fn pulse(phase: f32, duty: f32) -> f32 {
    if phase.rem_euclid(1.0) < duty {
        1.0
    } else {
        -1.0
    }
}

/// 위상(사이클 단위) → 삼각파(-1~+1).
fn triangle(phase: f32) -> f32 {
    4.0 * (phase.rem_euclid(1.0) - 0.5).abs() - 1.0
}

/// 한 보이스를 total_samples 길이 버퍼에 렌더(노트별 ASR 엔벨로프).
fn render_voice(v: &Voice, bpm: f32, total_samples: usize) -> Vec<f32> {
    let beat_secs = 60.0 / bpm;
    let mut out = vec![0.0f32; total_samples];
    let mut cursor = 0usize;
    for note in v.notes {
        let dur = (note.beats * beat_secs * SR as f32).round() as usize;
        if note.semi != REST && dur > 0 {
            let hz = note_hz(note.semi + v.octave * 12);
            let dur_t = dur as f32 / SR as f32;
            let attack = 0.005f32.min(dur_t * 0.25);
            let release = 0.03f32.min(dur_t * 0.4);
            let mut phase = 0.0f32;
            for i in 0..dur {
                let idx = cursor + i;
                if idx >= total_samples {
                    break;
                }
                let t = i as f32 / SR as f32;
                let env = if t < attack {
                    t / attack
                } else if t > dur_t - release {
                    ((dur_t - t) / release).max(0.0)
                } else {
                    1.0
                };
                let s = match v.wave {
                    Wave::Pulse(d) => pulse(phase, d),
                    Wave::Triangle => triangle(phase),
                };
                out[idx] += s * env * v.amp;
                phase += hz / SR as f32;
            }
        }
        cursor += dur;
    }
    out
}

/// 여러 보이스를 믹싱 → 0.9 헤드룸 정규화 → i16 PCM. 모든 보이스는 같은 총 비트
/// 길이를 가정(정수 마디)한다.
fn render_track(voices: &[Voice], bpm: f32) -> Vec<i16> {
    let beat_secs = 60.0 / bpm;
    let total_beats = voices
        .iter()
        .map(|v| v.notes.iter().map(|note| note.beats).sum::<f32>())
        .fold(0.0f32, f32::max);
    let total_samples = (total_beats * beat_secs * SR as f32).round() as usize;
    let mut mix = vec![0.0f32; total_samples];
    for v in voices {
        for (i, &s) in render_voice(v, bpm, total_samples).iter().enumerate() {
            mix[i] += s;
        }
    }
    let peak = mix.iter().fold(1e-6f32, |m, &s| m.max(s.abs()));
    let gain = 0.9 / peak;
    mix.iter().map(|&s| (s * gain * i16::MAX as f32) as i16).collect()
}

/// 트랙 WAV 저장 전 불변식 검증: 루프 이음새(시작·끝 진폭≈0)와 최소 길이.
fn write_music(dir: &Path, name: &str, samples: Vec<i16>) {
    let thresh = (0.03 * i16::MAX as f32) as i16;
    assert!(!samples.is_empty(), "{name}: 빈 트랙");
    assert!(samples.len() > SR as usize, "{name}: 트랙이 1초 미만");
    assert!(samples[0].abs() <= thresh, "{name}: 시작 진폭이 커 루프 클릭 발생");
    assert!(
        samples[samples.len() - 1].abs() <= thresh,
        "{name}: 끝 진폭이 커 루프 클릭 발생"
    );
    write_wav(dir, name, samples);
}

/// 타이틀 곡: C장조 I–vi–IV–V(C–Am–F–G) 진행, 은은한 어트랙트. 100 BPM, 16비트.
fn title_track() -> Vec<i16> {
    // 베이스(삼각파, 2옥타브 아래): 코드 루트를 4분음표로.
    let bass = [
        n(3, 1.0), n(3, 1.0), n(3, 1.0), n(3, 1.0), // C
        n(0, 1.0), n(0, 1.0), n(0, 1.0), n(0, 1.0), // Am
        n(8, 1.0), n(8, 1.0), n(8, 1.0), n(8, 1.0), // F
        n(10, 1.0), n(10, 1.0), n(10, 1.0), n(10, 1.0), // G
    ];
    // 아르페지오(펄스파): 각 코드 트라이어드+옥타브를 8분음표로 상행.
    let arp = [
        n(3, 0.5), n(7, 0.5), n(10, 0.5), n(15, 0.5), n(3, 0.5), n(7, 0.5), n(10, 0.5), n(15, 0.5), // C
        n(0, 0.5), n(3, 0.5), n(7, 0.5), n(12, 0.5), n(0, 0.5), n(3, 0.5), n(7, 0.5), n(12, 0.5), // Am
        n(8, 0.5), n(12, 0.5), n(15, 0.5), n(20, 0.5), n(8, 0.5), n(12, 0.5), n(15, 0.5), n(20, 0.5), // F
        n(10, 0.5), n(14, 0.5), n(17, 0.5), n(22, 0.5), n(10, 0.5), n(14, 0.5), n(17, 0.5), n(22, 0.5), // G
    ];
    render_track(
        &[
            Voice { notes: &bass, wave: Wave::Triangle, octave: -2, amp: 0.6 },
            Voice { notes: &arp, wave: Wave::Pulse(0.5), octave: 0, amp: 0.32 },
        ],
        100.0,
    )
}
