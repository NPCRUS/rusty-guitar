#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod chord;
mod models;
mod state;

use std::collections::HashSet;
use std::ops::Range;
use std::os::raw::c_ushort;
use eframe::{Frame, Storage};
use eframe::egui::*;
use eframe::egui::panel::Side;
use env_logger::Builder;
use crate::chord::{Chord, ChordDrawResult, draw_chord};
use itertools::Itertools;
use log::{debug, LevelFilter};
use crate::models::{Note, Song};
use crate::state::{Msg, run_messages, State, Tab};

const STORAGE_KEY: &str = "state";

fn main() -> Result<(), eframe::Error> {
    Builder::from_default_env().filter_level(LevelFilter::Debug).init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        follow_system_theme: true,
        app_id: Some("MyGuitar".to_owned()),
        initial_window_size: Some(vec2(1920.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "MyGuitar",
        options,
        Box::new(|_cc| {
             _cc.storage
                 .map(|s| s.get_string(STORAGE_KEY).unwrap_or("{}".to_owned()))
                 .map(|str| serde_json::from_str::<State>(&str).unwrap_or(State::default()))
                 .map(|s| Box::new(s))
                 .unwrap()
        }),
    )
}

impl eframe::App for State {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let mut messages: Vec<Msg> = vec![];

        TopBottomPanel::top("tabs").show_separator_line(true).exact_height(30.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Chords, "Chords");
                ui.selectable_value(&mut self.selected_tab, Tab::Songs, "Songs");
            });
        });

        match self.selected_tab {
            Tab::Chords => chords_section(self, &mut messages, ctx),
            Tab::Songs => songs_section(self, &mut messages, ctx)
        }

        run_messages(self, &messages)
    }

    fn save(&mut self, _storage: &mut dyn Storage) {
        _storage.set_string(STORAGE_KEY, serde_json::to_string(&self).unwrap())
    }
}

fn chords_section(state: &mut State, messages: &mut Vec<Msg>, ctx: &Context) {
    SidePanel::new(Side::Left, "search").show(ctx, |ui| {
        ui.text_edit_singleline(&mut state.chord_search_input);
        ui.separator();

        let chords_prepared = state.chords
            .iter()
            .map(|chord| &chord.name)
            .cloned()
            .collect::<HashSet<String>>()
            .into_iter()
            .filter(|chord_name| chord_name.contains(&state.chord_search_input))
            .sorted();

        for chord_name in chords_prepared {
            let label = SelectableLabel::new(state.selected_chord == chord_name, &chord_name);
            if ui.add(label).clicked() {
                messages.push(Msg::SelectChord(chord_name.clone()));
            }
        }
    });

    CentralPanel::default().show(ctx, |ui| {
        if state.selected_chord == "" {
            ui.label("Select a chord to continue please");
        } else {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&state.selected_chord).font(FontId::proportional(24.0)));
                if ui.button("+").clicked() {
                    messages.push(Msg::AddEmptyChord(state.selected_chord.clone()));
                }
            });
            ui.horizontal(|ui| {
                for chord in state.chords.iter_mut().filter(|chord| chord.name == state.selected_chord) {
                    match draw_chord(ctx, ui, chord) {
                        ChordDrawResult::Nothing => {}
                        ChordDrawResult::Remove => {
                            messages.push(Msg::DeleteChord(chord.id));
                        }
                    }
                }
            });
        }
    });
}

fn songs_section(state: &mut State, messages: &mut Vec<Msg>, ctx: &Context) {
    SidePanel::new(Side::Left, "search").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.song_search_input);
            if ui.button("+").clicked() && state.song_search_input.len() > 0 {
                messages.push(Msg::AddEmptySong(state.song_search_input.clone()));
            }
        });
        ui.separator();

        let songs_prepared = state.songs
            .iter()
            .map(|song| &song.name)
            .cloned()
            .collect::<HashSet<String>>()
            .into_iter()
            .filter(|song_name| song_name.contains(&state.song_search_input))
            .sorted();

        for song_name in songs_prepared {
            let label = SelectableLabel::new(state.selected_song == song_name, &song_name);
            if ui.add(label).clicked() {
                messages.push(Msg::SelectSong(song_name.clone()));
            }
        }
    });

    CentralPanel::default().show(ctx, |ui| {
        if state.selected_song == "" {
            ui.label("Select a song to continue please");
        } else {
            let song = state.songs.iter_mut().find(|s| s.name == state.selected_song).unwrap();

            if ui.text_edit_singleline(&mut song.name).changed() {
                messages.push(Msg::SelectSong(song.name.clone()));
            }
            ui.separator();
            let text_edit_output = TextEdit::multiline(&mut song.text)
                .min_size(ui.available_size())
                .show(ui);

            if let Some(Some(cursor)) = text_edit_output.cursor_range.map(|cr| cr.single()) {
                let cursor_position = cursor.ccursor.index;
                let possible_chord_str = extract_word_from_cursor_position(&song.text, cursor_position);

                // TODO: how to solve cloning???
                let chords_binding = state.chords.to_vec();
                let found_chords_to_read: Vec<&Chord> = chords_binding.iter().filter(|chord| chord.name == possible_chord_str).sorted_by_key(|c| c.id).collect();

                // display chord list
                let preference = song.preferences.get(possible_chord_str);
                let mut found_chords: Vec<&mut Chord> = state.chords.iter_mut().filter(|chord| chord.name == possible_chord_str).sorted_by_key(|c| c.id).collect();
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
                                                messages.push(Msg::InsertSongPreference(song.name.clone(), (*next_chord).clone()));
                                                // song.preferences.insert(next_chord.name.clone(), next_chord.id);
                                            },
                                            None => {
                                                let first_chord = found_chords_to_read.first().unwrap();
                                                messages.push(Msg::InsertSongPreference(song.name.clone(), (*first_chord).clone()));
                                                // song.preferences.insert(first_chord.name.clone(), first_chord.id);
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
                                    messages.push(Msg::AddEmptyChord(possible_chord_str.clone().to_string()));
                                }
                            });
                    },
                    None => ()
                }
            }
        }
    });
}

fn extract_word_from_cursor_position(text: &String, cursor_position: usize) -> &str {
    if text.len() == 0 {
        ""
    } else {
        let chars: Vec<char> = text.chars().collect();
        let stop_chars = [' ', '\n', '\t'];
        // debug!("{}", cursor_position);

        let mut start_idx = if cursor_position >= chars.len() {
            chars.len() - 1
        } else {
            cursor_position
        };
        // debug!("start_idx: {}, len: {}", start_idx, chars.len());
        while !(start_idx < 1 || stop_chars.contains(&chars[start_idx - 1])) {
            start_idx = start_idx - 1;
        }

        let mut end_idx = if cursor_position >= chars.len() {
            chars.len() - 1
        } else {
            cursor_position
        };
        // debug!("end_idx: {}, len: {}", end_idx, chars.len());
        while !(end_idx == chars.len() - 1 || stop_chars.contains(&chars[end_idx])) {
            end_idx = end_idx + 1;
        }

        // debug!("calc: {}, start_idx: {}, end_idx: {}", text.char_range(Range { start: start_idx, end: end_idx }), start_idx, end_idx);
        text.char_range(Range { start: start_idx, end: end_idx })
    }
}

fn is_like_chord(possible_chord: &str) -> bool {
    let notes_chars = Note::all().map(|n| n.to_string().chars().next().unwrap());
    possible_chord.starts_with(|c| {
        notes_chars.contains(&c)
    })
}