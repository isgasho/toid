use serde::{Deserialize, Serialize};

use super::Phrase;

#[derive(Serialize, Deserialize, Clone)]
pub struct Track {
    pub phrase: Phrase,
    pub sf2_name: Option<String>,
    pub vol: f32, // 0.0 ~ 1.0
    pub pan: f32, // -1.0(L) ~ 1.0(R)
}
