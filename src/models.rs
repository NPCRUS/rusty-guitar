use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};

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