use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};
use crate::chord::NotePos;

#[derive(Debug, Clone, PartialEq)]
pub enum Note {
    A, B, C, D, E, F, G
}

impl Note {
    pub fn all() -> [Note; 7] {
        use Note::*;
        return [A, B, C, D, E, F, G];
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chord {
    pub id: i32,
    pub name: String,
    pub notes: Vec<NotePos>
}

impl Chord {
    pub fn empty(id: i32, name: String) -> Self {
        Self {
            id,
            name,
            notes: vec![]
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Song {
    pub(crate) name: String,
    pub(crate) text: String,
    pub(crate) preferences: HashMap<String, i32>
}

impl Song {
    pub(crate) fn empty(name: String) -> Self {
        Song {
            name,
            text: "".to_owned(),
            preferences: HashMap::new()
        }
    }
}