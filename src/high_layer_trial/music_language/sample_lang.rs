use std::sync::Arc;

use super::super::super::data::music_info::{Beat, SampleNote, SamplePhrase};
use super::super::super::music_state::states::{MusicState, MusicStateEvent};
use super::super::super::music_state::wave_reader::{WaveReader, WaveReaderEvent};
use super::super::super::players::player::Player;
use super::send_phrase::send_sample_phrase;

pub fn parse_sample_lang(s: String) -> SamplePhrase {
    let mut now: Beat = Beat::from(0);
    let length_unit: Beat = Beat::from(0.5);
    let mut phrase = SamplePhrase::new();

    phrase = phrase.set_length(Beat::from(s.len() as f32 / 2.0));

    for c in s.chars() {
        let sound = match c {
            'x' => Some('x'.to_string()),
            'o' => Some('o'.to_string()),
            '-' => Some('-'.to_string()),
            _ => None,
        };

        match sound {
            Some(sound) => {
                let note = SampleNote { sound, start: now };
                phrase = phrase.add_note(note);
            }
            None => {}
        }

        now = now + length_unit;
    }

    phrase
}

pub fn send_sample_lang(
    phrase_string: String,
    section_beat: Beat,
    phrase_name: String,
    sample_name: String,
    vol: f32,
    pan: f32,
    player: Arc<
        dyn Player<MusicState, MusicStateEvent, WaveReader, (Vec<i16>, Vec<i16>), WaveReaderEvent>,
    >,
) -> Result<(), String> {
    send_sample_phrase(
        parse_sample_lang(phrase_string),
        section_beat,
        phrase_name,
        sample_name,
        vol,
        pan,
        player,
    )?;
    Ok(())
}