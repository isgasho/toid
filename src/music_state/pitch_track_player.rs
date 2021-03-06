use std::collections::{BTreeMap, BTreeSet};
use std::f32::consts::PI;
use std::iter::Iterator;
use std::ops::Bound::{Excluded, Included};
use std::sync::Arc;

use log::{error, warn};

use super::super::data::music_info::{Beat, Instrument, PitchNote, Track};
use super::super::music_state::effects::{Effect, EffectInfo};
use super::super::resource_management::resource_manager::ResourceManager;

fn tri(x: f32) -> f32 {
    let x = (x - 0.5 * PI) % (2.0 * PI);
    (x - PI).abs() / PI * 2.0 - 1.0
}

fn saw(x: f32) -> f32 {
    let x = x % (2.0 * PI);
    x / PI - 1.0
}

pub struct PitchTrackPlayer {
    wave_length: u64,
    played_notes: BTreeMap<u64, Vec<(u64, PitchNote)>>,
    effect_infos: Vec<EffectInfo>,
    effects: Vec<Box<dyn Effect + Sync + Send>>,
}

impl PitchTrackPlayer {
    pub fn new() -> Self {
        Self {
            wave_length: 512,
            played_notes: BTreeMap::new(),
            effect_infos: vec![],
            effects: vec![],
        }
    }

    pub fn clean(&mut self) {
        self.played_notes = BTreeMap::new();
    }

    pub fn play(
        &mut self,
        track: &Track<PitchNote>,
        resource_manager: Arc<ResourceManager>,
        cum_current_samples: &u64,
        cum_current_beats: &Beat,
        current_bpm: &f32,
    ) -> (Vec<f32>, Vec<f32>) {
        let mut left_wave: Vec<f32> = Vec::new();
        let mut right_wave: Vec<f32> = Vec::new();
        left_wave.resize(self.wave_length as usize, 0.0);
        right_wave.resize(self.wave_length as usize, 0.0);

        let cum_next_samples = cum_current_samples + self.wave_length;
        let cum_next_beats =
            *cum_current_beats + Beat::from(self.wave_length as f32 * current_bpm / 44100.0 / 60.0);

        // 付け加えるnotesをリストアップする。
        // self.played_notesに加える。
        if track.phrase.length > Beat::from(0) {
            let rep_current_beats = *cum_current_beats % track.phrase.length;
            let rep_next_beats = cum_next_beats % track.phrase.length;

            if rep_current_beats < rep_next_beats {
                for (&start, new_notes) in track
                    .phrase
                    .notes
                    .range((Included(rep_current_beats), Excluded(rep_next_beats)))
                {
                    let cum_start_samples = ((start - rep_current_beats).to_f32() * 44100.0 * 60.0
                        / current_bpm) as u64
                        + cum_current_samples;
                    self.register_notes(new_notes, &current_bpm, &cum_start_samples);
                }
            } else {
                for (&start, new_notes) in track
                    .phrase
                    .notes
                    .range((Included(rep_current_beats), Excluded(track.phrase.length)))
                {
                    let cum_start_samples = ((start - rep_current_beats).to_f32() * 44100.0 * 60.0
                        / current_bpm) as u64
                        + cum_current_samples;
                    self.register_notes(new_notes, &current_bpm, &cum_start_samples);
                }
                for (&start, new_notes) in track
                    .phrase
                    .notes
                    .range((Included(Beat::from(0)), Excluded(rep_next_beats)))
                {
                    let cum_start_samples = ((track.phrase.length + start - rep_current_beats)
                        .to_f32()
                        * 44100.0
                        * 60.0
                        / current_bpm) as u64
                        + cum_current_samples;

                    self.register_notes(new_notes, &current_bpm, &cum_start_samples);
                }
            }
        }

        // self.played_notesのを鳴らす
        match &track.instrument {
            Instrument::Sin => {
                for (&cum_end_samples, notes) in self.played_notes.iter() {
                    for (cum_start_samples, note) in notes.iter() {
                        let herts_par_sample = note.pitch.get_hertz() / 44100.0;
                        let start_idx = if *cum_start_samples <= *cum_current_samples {
                            0
                        } else {
                            (cum_start_samples - cum_current_samples) as usize
                        };
                        let end_idx = if cum_end_samples >= cum_next_samples {
                            self.wave_length as usize
                        } else {
                            (cum_end_samples - cum_current_samples) as usize
                        };

                        for i in start_idx..end_idx {
                            let x = (cum_current_samples + i as u64 - cum_start_samples) as f32
                                * herts_par_sample;
                            let x = x * 2.0 * PI;
                            let addition = x.sin() * 0.3 * track.vol;
                            left_wave[i] = left_wave[i] + (1.0 - track.pan) * addition;
                            right_wave[i] = right_wave[i] + (1.0 + track.pan) * addition;
                        }
                    }
                }
            }
            Instrument::Tri => {
                for (&cum_end_samples, notes) in self.played_notes.iter() {
                    for (cum_start_samples, note) in notes.iter() {
                        let herts_par_sample = note.pitch.get_hertz() / 44100.0;
                        let start_idx = if *cum_start_samples <= *cum_current_samples {
                            0
                        } else {
                            (cum_start_samples - cum_current_samples) as usize
                        };
                        let end_idx = if cum_end_samples >= cum_next_samples {
                            self.wave_length as usize
                        } else {
                            (cum_end_samples - cum_current_samples) as usize
                        };

                        for i in start_idx..end_idx {
                            let x = (cum_current_samples + i as u64 - cum_start_samples) as f32
                                * herts_par_sample;
                            let x = x * 2.0 * (PI as f32);
                            let addition = tri(x) * 0.3 * track.vol;
                            left_wave[i] = left_wave[i] + (1.0 - track.pan) * addition;
                            right_wave[i] = right_wave[i] + (1.0 + track.pan) * addition;
                        }
                    }
                }
            }
            Instrument::Saw => {
                for (&cum_end_samples, notes) in self.played_notes.iter() {
                    for (cum_start_samples, note) in notes.iter() {
                        let herts_par_sample = note.pitch.get_hertz() / 44100.0;
                        let start_idx = if *cum_start_samples <= *cum_current_samples {
                            0
                        } else {
                            (cum_start_samples - cum_current_samples) as usize
                        };
                        let end_idx = if cum_end_samples >= cum_next_samples {
                            self.wave_length as usize
                        } else {
                            (cum_end_samples - cum_current_samples) as usize
                        };

                        for i in start_idx..end_idx {
                            let x = (cum_current_samples + i as u64 - cum_start_samples) as f32
                                * herts_par_sample;
                            let x = x * 2.0 * (PI as f32);
                            let addition = saw(x) * 0.3 * track.vol;
                            left_wave[i] = left_wave[i] + (1.0 - track.pan) * addition;
                            right_wave[i] = right_wave[i] + (1.0 + track.pan) * addition;
                        }
                    }
                }
            }
            Instrument::SF2(sf2_name, preset_idx) => {
                let sf2 = resource_manager.get_sf2(sf2_name.to_string());
                match sf2 {
                    Ok(sf2) => {
                        for (&cum_end_samples, notes) in self.played_notes.iter() {
                            for (cum_start_samples, note) in notes.iter() {
                                let start_idx = if *cum_start_samples <= *cum_current_samples {
                                    0
                                } else {
                                    (cum_start_samples - cum_current_samples) as usize
                                };
                                let end_idx = if cum_end_samples >= cum_next_samples {
                                    self.wave_length as usize
                                } else {
                                    (cum_end_samples - cum_current_samples) as usize
                                };

                                let start_idx_for_sample = (cum_current_samples + start_idx as u64
                                    - cum_start_samples)
                                    as usize;
                                let end_idx_for_sample = (cum_current_samples + end_idx as u64
                                    - cum_start_samples)
                                    as usize;

                                let sample_data = sf2.get_samples(
                                    *preset_idx,
                                    note.pitch.get_u8_pitch(),
                                    start_idx_for_sample,
                                    end_idx_for_sample,
                                );
                                match sample_data {
                                    Ok(sample_data) => {
                                        for (i, j) in (start_idx..end_idx).enumerate() {
                                            let addition = sample_data[i] * 0.5 * track.vol;
                                            left_wave[j] =
                                                left_wave[j] + (1.0 - track.pan) * addition;
                                            right_wave[j] =
                                                right_wave[j] + (1.0 + track.pan) * addition;
                                        }
                                    }
                                    Err(e) => {
                                        // TODO:
                                        error!("error {}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("sf2 error {}", e);
                    }
                }
            }
            _ => warn!("instrument is not for pitch track"),
        };

        // Effect更新
        if self.effect_infos != track.effects {
            self.effect_infos = track.effects.clone();
            let mut effects = vec![];
            for efi in self.effect_infos.iter() {
                effects.push(efi.get_effect(Arc::clone(&resource_manager)));
            }
            self.effects = effects;
        }

        // Effect
        for effect in self.effects.iter_mut() {
            let (l, r) = effect.effect(&left_wave, &right_wave);
            left_wave = l;
            right_wave = r;
        }

        // 使ったself.played_notesのノートを消す
        for cum_note_samples in *cum_current_samples..cum_next_samples {
            self.played_notes.remove(&cum_note_samples);
        }

        (left_wave, right_wave)
    }

    fn register_notes(
        &mut self,
        notes: &BTreeSet<PitchNote>,
        current_bpm: &f32,
        cum_start_samples: &u64,
    ) {
        for &note in notes.iter() {
            let cum_end_samples =
                cum_start_samples + (note.duration.to_f32() * 44100.0 * 60.0 / current_bpm) as u64;

            if self.played_notes.contains_key(&cum_end_samples) {
                match self.played_notes.get_mut(&cum_end_samples) {
                    Some(notes_in_cum_end_samples) => {
                        notes_in_cum_end_samples.push((*cum_start_samples, note));
                    }
                    None => {
                        error!("get_mut failed");
                    }
                };
            } else {
                self.played_notes
                    .insert(cum_end_samples, vec![(*cum_start_samples, note)]);
            }
        }
    }
}
