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
