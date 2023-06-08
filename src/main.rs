#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod theory;
mod chord;

use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::ops::Range;
use std::process::id;
use eframe::{Frame, Storage};
use serde::{Deserialize, Serialize};
use eframe::egui::*;
use eframe::egui::panel::Side;
use eframe::epaint::ahash::{HashMap, HashMapExt};
use crate::chord::{Chord, ChordDrawResult, draw_chord, NotePos};
use itertools::Itertools;
use log::debug;
use crate::theory::Note;

const STORAGE_KEY: &str = "state";

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        follow_system_theme: true,
        app_id: Some("MyGuitar".to_owned()),
        initial_window_size: Some(vec2(1920.0, 590.0)),
        ..Default::default()
    };

    eframe::run_native(
        "My guitar App",
        options,
        Box::new(|_cc| {
             _cc.storage
                 .map(|s| s.get_string(STORAGE_KEY).unwrap_or("{}".to_owned()))
                 .map(|str| serde_json::from_str::<MyGuitarApp>(&str).unwrap_or(MyGuitarApp::default()))
                 .map(|s| Box::new(s))
                 .unwrap()
        }),
    )
}

#[derive(Serialize, Deserialize)]
struct Song {
    name: String,
    text: String,
    preferences: HashMap<String, i32>
}

impl Song {
    fn empty(name: String) -> Self {
        Song {
            name,
            text: "".to_owned(),
            preferences: HashMap::new()
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
enum Tab {
    Chords,
    Songs,
}

#[derive(Serialize, Deserialize)]
struct MyGuitarApp {
    chords: Vec<Chord>,
    songs: Vec<Song>,

    selected_tab: Tab,
    selected_chord: String,
    chord_search_input: String,
    selected_song: String,
    song_search_input: String
}

impl MyGuitarApp {
    fn default() -> Self {
        MyGuitarApp {
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

impl eframe::App for MyGuitarApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        TopBottomPanel::top("tabs").show_separator_line(true).exact_height(30.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Chords, "Chords");
                ui.selectable_value(&mut self.selected_tab, Tab::Songs, "Songs");
            });
        });

        SidePanel::new(Side::Left, "search").show(ctx, |ui| {
            match self.selected_tab {
                Tab::Chords => {
                    ui.text_edit_singleline(&mut self.chord_search_input);
                    ui.separator();

                    let chords_prepared = self.chords
                        .iter()
                        .map(|chord| &chord.name)
                        .cloned()
                        .collect::<HashSet<String>>()
                        .into_iter()
                        .filter(|chord_name| chord_name.contains(&self.chord_search_input))
                        .sorted();

                    for chord_name in chords_prepared {
                        let label = SelectableLabel::new(self.selected_chord == chord_name, &chord_name);
                        if ui.add(label).clicked() {
                            self.selected_chord = chord_name.clone();
                        }
                    }
                }
                Tab::Songs => {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.song_search_input);
                        if ui.button("+").clicked() && self.song_search_input.len() > 0 {
                            self.songs.push(Song::empty(self.song_search_input.clone()))
                        }
                    });
                    ui.separator();

                    let songs_prepared = self.songs
                        .iter()
                        .map(|song| &song.name)
                        .cloned()
                        .collect::<HashSet<String>>()
                        .into_iter()
                        .filter(|song_name| song_name.contains(&self.song_search_input))
                        .sorted();

                    for song_name in songs_prepared {
                        let label = SelectableLabel::new(self.selected_song == song_name, &song_name);
                        if ui.add(label).clicked() {
                            self.selected_song = song_name.clone();
                        }
                    }
                }
            }
        });

        CentralPanel::default().show(ctx, |ui| {
            match self.selected_tab {
                Tab::Chords => {
                    if self.selected_chord == "" {
                        ui.label("Select a chord to continue please");
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&self.selected_chord).font(FontId::proportional(24.0)));
                            if ui.button("+").clicked() {
                                add_empty_chord(&mut self.chords, &self.selected_chord);
                            }
                        });
                        ui.horizontal(|ui| {
                            let mut chords_to_delete: Vec<i32> = vec![];
                            for chord in self.chords.iter_mut().filter(|chord| chord.name == self.selected_chord) {
                                match draw_chord(ctx, ui, chord) {
                                    ChordDrawResult::Nothing => {}
                                    ChordDrawResult::Remove => {
                                        chords_to_delete.push(chord.id);
                                    }
                                }
                            }

                            // delete chords if needed
                            self.chords.retain(|chord| !chords_to_delete.contains(&chord.id))
                        });
                    }
                }
                Tab::Songs => {
                    if self.selected_song == "" {
                        ui.label("Select a song to continue please");
                    } else {
                        let song = self.songs.iter_mut().find(|s| s.name == self.selected_song).unwrap();

                        if ui.text_edit_singleline(&mut song.name).changed() {
                            self.selected_song = song.name.clone();
                        }
                        ui.separator();
                        let text_edit_output = TextEdit::multiline(&mut song.text)
                            .min_size(ui.available_size())
                            .show(ui);

                        if let Some(Some(cursor)) = text_edit_output.cursor_range.map(|cr| cr.single()) {
                            let cursor_position = cursor.ccursor.index;
                            let possible_chord_str = extract_word_from_cursor_position(&song.text, cursor_position);

                            // TODO: how to solve cloning???
                            let chords_binding = self.chords.to_vec();
                            let found_chords_to_read: Vec<&Chord> = chords_binding.iter().filter(|chord| chord.name == possible_chord_str).sorted_by_key(|c| c.id).collect();

                            // display chord list
                            let preference = song.preferences.get(possible_chord_str);
                            let mut found_chords: Vec<&mut Chord> = self.chords.iter_mut().filter(|chord| chord.name == possible_chord_str).sorted_by_key(|c| c.id).collect();
                            let target_chord: Option<&mut &mut Chord> = found_chords.iter_mut().find(|chord| preference.map_or(true, |p| *p == chord.id));
                            let chord_drawing_position = text_edit_output.text_clip_rect.min + Vec2::new(350.0, cursor.rcursor.row as f32 * 10.0);
                            match target_chord {
                                Some(chord) => {
                                    Window::new(&chord.name)
                                        .current_pos(chord_drawing_position)
                                        .show(ctx, |ui| {
                                            ui.horizontal(|ui| {
                                                draw_chord(ctx, ui, chord);
                                                if found_chords_to_read.len() > 1 && ui.button(">").clicked() {
                                                    match found_chords_to_read.iter().find(|c| c.id > chord.id) {
                                                        Some(next_chord) => {
                                                            song.preferences.insert(next_chord.name.clone(), next_chord.id);
                                                        },
                                                        None => {
                                                            let first_chord = found_chords_to_read.first().unwrap();
                                                            song.preferences.insert(first_chord.name.clone(), first_chord.id);
                                                        }
                                                    }

                                                }
                                            });
                                        });
                                },
                                None if is_like_chord(possible_chord_str) => {
                                    Window::new(possible_chord_str)
                                        .fixed_pos(chord_drawing_position)
                                        .show(ctx, |ui| {
                                            if(ui.button("Create")).clicked() {
                                                add_empty_chord(&mut self.chords, &possible_chord_str.to_owned())
                                            }
                                        });
                                },
                                None => ()
                            }
                        }
                    }
                }
            }
        });
    }

    fn save(&mut self, _storage: &mut dyn Storage) {
        _storage.set_string(STORAGE_KEY, serde_json::to_string(&self).unwrap())
    }
}

fn extract_word_from_cursor_position(text: &String, cursor_position: usize) -> &str {
    let chars: Vec<char> = text.chars().collect();
    let mut start_idx = cursor_position;
    let stop_chars = [' ', '\n', '\t'];
    while !stop_chars.contains(&chars[start_idx - 1]) {
        start_idx = start_idx - 1;
    }
    let mut end_idx = cursor_position;
    while !stop_chars.contains(&chars[end_idx]) {
        end_idx = end_idx + 1;
    }
    text.char_range(Range { start: start_idx, end: end_idx})
}

fn add_empty_chord(chords: &mut Vec<Chord>, name: &String) {
    let last_id = chords.last().map(|c| c.id).unwrap_or(0);
    chords.push(Chord::empty(last_id + 1, name.clone()))
}

fn is_like_chord(str: &str) -> bool {
    let notes_chars = Note::all().map(|n| n.to_string().chars().next().unwrap());
    str.starts_with(|c| {
        notes_chars.contains(&c)
    })
}