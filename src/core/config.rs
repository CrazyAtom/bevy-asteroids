//! 모든 튜닝 상수 — 도메인별 서브모듈로 분리하되, 전부 재수출해
//! 기존 `crate::core::config::상수` 경로를 그대로 유지한다.
//! 밸런스/튜닝은 전부 여기(하위 도메인 파일)서.

mod boss;
mod combat;
mod render;
mod ship;
mod stage;
mod world;

pub use boss::*;
pub use combat::*;
pub use render::*;
pub use ship::*;
pub use stage::*;
pub use world::*;
