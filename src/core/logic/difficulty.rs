//! 웨이브 번호 기반 난이도 공식(소행성 수/속도, UFO 빈도/구성).

/// 난이도 계산용 0-기반 지수. 게임은 웨이브1부터 시작하므로 웨이브1이
/// 기준선(지수 0)이 되도록 1을 뺀다. → 첫 웨이브 = 클래식 기본 난이도(소행성 4개).
fn difficulty_index(wave: u32) -> u32 {
    wave.saturating_sub(1)
}

pub fn asteroid_count_for_wave(wave: u32) -> usize {
    (crate::core::config::BASE_ASTEROIDS + difficulty_index(wave) as usize)
        .min(crate::core::config::MAX_ASTEROIDS)
}

pub fn asteroid_speed_scale_for_wave(wave: u32) -> f32 {
    (1.0 + difficulty_index(wave) as f32 * 0.065).min(2.0)
}

pub fn ufo_interval_for_wave(wave: u32) -> f32 {
    (crate::core::config::UFO_SPAWN_INTERVAL_BASE - difficulty_index(wave) as f32 * 0.8)
        .max(crate::core::config::UFO_SPAWN_INTERVAL_MIN)
}

pub fn small_ufo_probability_for_wave(wave: u32) -> f32 {
    (0.2 + difficulty_index(wave) as f32 * 0.05).min(0.9)
}

/// 웨이브1은 UFO 없이 소행성만 등장(초반 학습 구간). 웨이브2부터 UFO 활성화.
pub fn ufo_active_for_wave(wave: u32) -> bool {
    wave >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_scaling_formulas() {
        // 웨이브1 = 기준선(지수 0): 기본4개, 상한10
        assert_eq!(asteroid_count_for_wave(1), 3);
        assert_eq!(asteroid_count_for_wave(4), 6); // 웨이브4 → 지수3 → 3+3
        assert_eq!(asteroid_count_for_wave(50), 10); // 상한
        // 속도 배수: 웨이브↑ → 증가, 웨이브1은 정확히 1.0
        assert!(asteroid_speed_scale_for_wave(6) > asteroid_speed_scale_for_wave(1));
        assert_eq!(asteroid_speed_scale_for_wave(1), 1.0);
        // UFO 간격: 웨이브↑ → 감소, 하한 존재
        assert!(ufo_interval_for_wave(6) < ufo_interval_for_wave(1));
        assert!(ufo_interval_for_wave(100) >= 5.0);
        // 소형 확률: 웨이브↑ → 증가, [0,1]
        assert!(small_ufo_probability_for_wave(11) > small_ufo_probability_for_wave(1));
        assert!(small_ufo_probability_for_wave(100) <= 1.0);

        // 캡 경계값 고정: 큰 웨이브에서 상한/하한이 정확히 걸리는지
        assert_eq!(asteroid_count_for_wave(1000), 10);            // 개수 상한
        assert_eq!(asteroid_speed_scale_for_wave(1000), 2.0);     // 속도 배수 상한
        assert_eq!(ufo_interval_for_wave(1000), 5.0);             // UFO 간격 하한
        assert_eq!(small_ufo_probability_for_wave(1000), 0.9);    // 소형 확률 상한

        // 웨이브1은 UFO 미등장, 웨이브2부터 활성화
        assert!(!ufo_active_for_wave(1));
        assert!(ufo_active_for_wave(2));
    }
}
