//! toidはlive codingツールを提供します。
//! パフォーマンス向上のため、toidはrustで実装されていますが、
//! 通常はPython3などのスクリプト言語にバインディングして使うことを想定しています。

pub mod data;
pub mod music_state_manager;
pub mod new_state_management;
pub mod portaudio_outputter;
pub mod reducers;
pub mod state_management;
pub mod states;
pub mod stores;

#[cfg(test)]
mod tests;
