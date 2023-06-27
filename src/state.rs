use crate::chord::Chord;
use crate::models::Song;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub chords: Vec<Chord>,
    pub songs: Vec<Song>,

    pub selected_tab: Tab,
    pub selected_chord: String,
    pub chord_search_input: String,
    pub selected_song: String,
    pub song_search_input: String
}

impl State {
    pub(crate) fn default() -> Self {
        State {
            chords: vec![],
            songs: vec![],

            selected_tab: Tab::Chords,
            selected_chord: "".to_owned(),
            chord_search_input: "".to_owned(),
            selected_song: "".to_owned(),
            song_search_input: "".to_owned()
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum Tab {
    Chords,
    Songs,
}

pub enum Msg {
    DeleteChord(i32),
    AddEmptyChord(String)
}

fn run_message(state: &mut State, msg: &Msg) {
    match msg {
        Msg::DeleteChord(id) => {
            ()
        },
        Msg::AddEmptyChord(name) => {
            let last_id = state.chords.last().map(|c| c.id).unwrap_or(0);
            state.chords.push(Chord::empty(last_id + 1, name.clone()));
        }
    }
}

pub fn run_messages(state: &mut State, messages: &Vec<Msg>) {
    for message in messages.iter() {
        run_message(state, message)
    }
}