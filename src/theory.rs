use std::fmt;

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