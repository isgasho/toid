use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::beat::Beat;
use super::note::Note;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Phrase<N: Note + Eq + Ord + Clone> {
    pub notes: BTreeMap<Beat, BTreeSet<N>>,
    pub length: Beat,
}

impl<N: Note + Eq + Ord + Clone> Phrase<N> {
    pub fn new() -> Self {
        Self {
            notes: BTreeMap::new(),
            length: Beat::from(0),
        }
    }

    pub fn add_note(&self, note: N) -> Self {
        let mut new_notes = self.notes.clone();
        let mut new_note_set;
        if self.notes.contains_key(&note.get_start()) {
            new_note_set = self.notes[&note.get_start()].clone();
        } else {
            new_note_set = BTreeSet::new();
        }
        new_note_set.insert(note.clone());
        new_notes.insert(note.get_start(), new_note_set);
        Phrase {
            notes: new_notes,
            length: self.length,
        }
    }

    pub fn set_length(&self, length: Beat) -> Self {
        Phrase {
            notes: self.notes.clone(),
            length: length,
        }
    }

    pub fn note_vec(&self) -> Vec<N> {
        let mut vec = vec![];
        for (_, notes_in_time) in self.notes.iter() {
            for note in notes_in_time.iter() {
                vec.push(note.clone());
            }
        }
        vec
    }
}

impl<N: Note + Eq + Ord + Clone> PartialEq for Phrase<N> {
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length && self.notes == other.notes
    }
}

impl<N: Note + Eq + Ord + Clone> Eq for Phrase<N> {}

#[cfg(test)]
mod tests {
    use super::super::{Beat, Pitch, PitchNote};
    use super::*;

    #[test]
    fn test_eq() {
        let phrase1 = Phrase::new();

        let phrase1 = phrase1.add_note(PitchNote {
            pitch: Pitch::from(60),
            start: Beat::from(0.0),
            duration: Beat::from(1.0),
        });
        let phrase1 = phrase1.add_note(PitchNote {
            pitch: Pitch::from(62),
            start: Beat::from(1.0),
            duration: Beat::from(1.0),
        });
        let phrase2 = phrase1.clone();
        let phrase3 = phrase2.add_note(PitchNote {
            pitch: Pitch::from(64),
            start: Beat::from(2.0),
            duration: Beat::from(1.0),
        });

        assert_eq!(phrase1, phrase2);
        assert_ne!(phrase1, phrase3);
    }
}
