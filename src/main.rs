#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod theory;
mod chord;

use std::collections::HashSet;
use std::ops::Range;
use eframe::Frame;
use eframe::egui::*;
use eframe::egui::panel::Side;
use crate::chord::{Chord, ChordDrawResult, draw_chord, NotePos};
use itertools::Itertools;
use log::debug;
use crate::theory::Note;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(vec2(1920.0, 1080.0)),
        ..Default::default()
    };

    let chord = Chord {
        id: 1,
        name: "Dmaj7".to_owned(),
        notes: vec![(5, 5), (7, 4), (6, 3), (7, 2)]
    };
    let chords: Vec<Chord> = vec![chord, Chord::empty(2, "Cmaj7".to_owned()), Chord::empty(3, "Dmin7".to_owned()), Chord::empty(4, "Dmaj7".to_owned())];
    let songs: Vec<Song> = vec![Song {
        name: "How deep is your love".to_owned(),
        text: "How deep is your love\n              Dmaj7\nIs your love, how deep is your love".to_owned()
    }];
    // TODO: perform locally
    let state = Box::<MyGuitarApp>::new(MyGuitarApp{
        chords,
        songs,

        selected_tab: Tab::Chords,
        selected_chord: "".to_owned(),
        chord_search_input: "".to_owned(),

    });

    eframe::run_native(
        "My guitar App",
        options,
        Box::new(|_cc| state),
    )
}

struct Song {
    name: String,
    text: String
}

struct MyGuitarApp {
    chords: Vec<Chord>,
    songs: Vec<Song>,

    selected_tab: Tab,
    selected_chord: String,
    chord_search_input: String
}

#[derive(PartialEq)]
enum Tab {
    Chords,
    Songs,
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
                            ui.label(RichText::new(self.selected_chord.clone()).font(FontId::proportional(24.0)));
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
                    let text_edit_output = TextEdit::multiline(&mut self.songs.first_mut().unwrap().text)
                        .min_size(ui.available_size())
                        .show(ui);

                    if let Some(Some(cursor)) = text_edit_output.cursor_range.map(|cr| cr.single()) {
                        let cursor_position = cursor.ccursor.index;
                        let range = Range { start: cursor_position, end: cursor_position + 10 };
                        let possible_chord_str = self.songs.first().unwrap().text.char_range(range)
                            .clone()
                            .replace("\n", " ")
                            .split(' ')
                            .next().unwrap().trim().to_owned();

                        // display chord list
                        let found_chord: Option<&mut Chord> = self.chords.iter_mut().find(|chord| chord.name == possible_chord_str);
                        let chord_drawing_position = text_edit_output.text_clip_rect.min + Vec2::new(350.0, cursor.rcursor.row as f32 * 10.0);
                        match found_chord {
                            Some(chord) => {
                                Window::new(&chord.name)
                                    .fixed_pos(chord_drawing_position)
                                    .show(ctx, |ui| {
                                        ui.horizontal(|ui| {
                                            draw_chord(ctx, ui, chord);
                                        });
                                    });
                            },
                            None if is_like_chord(&possible_chord_str) => {
                                Window::new(&possible_chord_str).show(ctx, |ui| {
                                    if(ui.button("Create")).clicked() {
                                        add_empty_chord(&mut self.chords, &possible_chord_str)
                                    }
                                });
                            },
                            None => {

                            }
                        }
                    }
                }
            }
        });
    }
}

fn add_empty_chord(chords: &mut Vec<Chord>, name: &String) {
    let last_id = chords.last().map(|c| c.id).unwrap_or(0);
    chords.push(Chord::empty(last_id + 1, name.clone()))
}

fn is_like_chord(str: &String) -> bool {
    let notes_chars = Note::all().map(|n| n.to_string().chars().next().unwrap());
    str.starts_with(|c| {
        notes_chars.contains(&c)
    })
}