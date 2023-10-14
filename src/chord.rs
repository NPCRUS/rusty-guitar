

use std::ops::Div;
use eframe::egui::*;
use eframe::epaint::CircleShape;
use crate::models::Note;

const STRING_NUMBER: i32 = 6;

// TODO: extract into configuration
const HEIGHT: f32 = 160.0;
const WIDTH: f32 = 220.0;
const LEFT_PADDING: f32 = 10.0;
const RIGHT_PADDING: f32 = 10.0;
const TOP_PADDING: f32 = 15.0;
const BOTTOM_PADDING: f32 = 30.0;

const STRING_THICKNESS: f32 = 1.0;
const FRET_THICKNESS: f32 = 1.0;
const STRING_EXTRACTION_SPACE: f32 = 17.5;

const NOTES: [Note; 7] = [Note::A, Note::B, Note::C, Note::D, Note::E, Note::F, Note::G];

// x,y: x - fret, y: string
pub type NotePos = (i32, i32);

enum NoteExtraction {
    Note(NotePos),
    Muted
}

pub struct ChordResponse {
    pub response: Response,
    pub is_deleted: bool
}

pub fn draw_chord(ctx: &Context, ui: &mut Ui, notes: &mut Vec<NotePos>) -> ChordResponse {
    let mut is_deleted = false;
    let (response, painter)= ui.allocate_painter(Vec2::new(WIDTH, HEIGHT), Sense::click());
    let rect = response.rect;

    let fill = ctx.style().visuals.panel_fill;
    let color = ctx.style().visuals.text_color();
    let distance_between_strings = (HEIGHT - (TOP_PADDING + BOTTOM_PADDING)) / (STRING_NUMBER - 1) as f32;
    // filter open strings
    let frets: Vec<i32> = notes.iter().filter(|(fret, _)| *fret != 0).map(|(x, _)| *x).collect();
    let min_fret = if frets.len() == 0 {
        1
    } else {
        *frets.iter().min().unwrap()
    };

    let max_fret = if frets.len() == 0 {
        3
    } else {
        let max_fret = *frets.iter().max().unwrap();
        if max_fret - min_fret < 3 {
            min_fret + 2
        } else {
            max_fret
        }
    };
    let fret_amount = max_fret - min_fret + 2;
    let fret_distance = (WIDTH - LEFT_PADDING - STRING_EXTRACTION_SPACE - RIGHT_PADDING) / (fret_amount - 1) as f32;

    // draw strings

    for s in 1..(STRING_NUMBER + 1) {
        let y = s as f32 * distance_between_strings - distance_between_strings + TOP_PADDING;

        // draw muted string
        let is_muted = notes.iter().filter(|(_, y)| *y == s).count() == 0;
        if is_muted {
            draw_note_extraction(&painter, fill, color, rect.min + Vec2::new(LEFT_PADDING + 2.5, y), NoteExtraction::Muted);
        }
        let is_open = notes.iter().filter(|(x, y)| *x == 0 && *y == s).count() == 1;
        if is_open {
            draw_note_extraction(&painter, fill, color, rect.min + Vec2::new(LEFT_PADDING + 2.5, y), NoteExtraction::Note((0, s)));
        }

        // draw string
        let x_padding = LEFT_PADDING + STRING_EXTRACTION_SPACE;
        painter.line_segment([
                                 rect.min + Vec2::new(x_padding, y),
                                 rect.min + Vec2::new(WIDTH - RIGHT_PADDING, y)
                             ], Stroke::new(STRING_THICKNESS, color));

        // frets
        for f in 1..(fret_amount + 1) {
            let x = f as f32 * fret_distance - fret_distance;
            let fret_number = f + min_fret - 1;
            // draw fret
            painter.line_segment([
                rect.min + Vec2::new(x_padding + x, TOP_PADDING),
                rect.min + Vec2::new(x_padding + x, HEIGHT - BOTTOM_PADDING)
            ], Stroke::new(FRET_THICKNESS, color));

            // draw note if exist
            // filter open strings
            match notes.iter()
                .filter(|(fret, _)| *fret != 0)
                .find(|(fret, string)| *fret == fret_number && *string == s) {
                None => (),
                Some(v) => {
                    let circle_center = rect.min + Vec2::new(x_padding + x + (fret_distance / 2.0), y);
                    draw_note_extraction(&painter, fill, color, circle_center, NoteExtraction::Note(*v));
                }
            }

            // draw fret number
            if s == STRING_NUMBER {
                painter.text(
                    rect.min + Vec2::new(x_padding + x + (fret_distance / 2.0), HEIGHT - 7.5),
                    Align2::CENTER_CENTER,
                    &fret_string_from_number(fret_number),
                    FontId::default(),
                    color
                );
            }
        }
    }

    // draw chord menu
    response.clone().context_menu(|ui| {
        if notes.len() > 0 {
            if ui.button("minus fret").clicked() {
                let notes_on_first_fret = notes.iter().find(|(x, _)| *x == 1);
                if notes_on_first_fret.is_none() {
                    for note in notes.iter_mut() {
                        // don't move open notes around
                        if note.0 > 0 {
                            *note = (note.0 - 1, note.1)
                        }
                    }
                }
            }

            if ui.button("plus fret").clicked() {
                for note in notes.iter_mut() {
                    // don't move open notes around
                    if note.0 > 0 {
                        *note = (note.0 + 1, note.1)
                    }
                }
            };
        }

        // TODO: fix removing, removes multiple chords
        if ui.button("remove").clicked() {
            is_deleted = true;
        }

        if ui.button("close menu").clicked() {
            ui.close_menu();
        }
    });

    // editing chord
    if response.clicked() {
        ui.close_menu();

        match response.interact_pointer_pos() {
            None => (),
            Some(mouse_pos) => {
                let length_x = mouse_pos.x - rect.min.x;
                let length_y = mouse_pos.y - rect.min.y;

                let string = (length_y as i32).div(distance_between_strings as i32) + 1;
                let fret = if length_x > 30.0 {
                    (length_x as i32).div(fret_distance as i32) + min_fret
                } else {
                    0
                };

                let note = notes.iter().position(|(x, y)| *x == fret && *y == string);
                match note {
                    None => notes.push((fret, string)),
                    Some(pos) => {
                        notes.remove(pos);
                    }
                }
            }
        }
    }

    ChordResponse {
        response,
        is_deleted,
    }
}

fn draw_note_extraction(painter: &Painter, fill: Color32, color: Color32, pos: Pos2, note: NoteExtraction) {
    match note {
        NoteExtraction::Note(note) => {
            painter.add(CircleShape {
                center: pos,
                radius: 10.0,
                fill,
                stroke: Stroke::new(FRET_THICKNESS, color),
            });
            painter.text(
                pos,
                Align2::CENTER_CENTER,
                get_note_by_string_and_fret(note),
                FontId::new(12.0, FontId::default().family),
                color
            );
        }
        NoteExtraction::Muted => {
            painter.line_segment([
                pos + Vec2::new(-5.0, -5.0),
                pos + Vec2::new(5.0, 5.0)
            ], Stroke::new(1.0, color));
            painter.line_segment([
                pos + Vec2::new(5.0, -5.0),
                pos + Vec2::new(-5.0, 5.0)
            ], Stroke::new(1.0, color));
        }
    }
}

fn get_note_by_string_and_fret(note: NotePos) -> String {
    let open_string_note = match note.1 {
        1 => Note::E,
        2 => Note::B,
        3 => Note::G,
        4 => Note::D,
        5 => Note::A,
        6 => Note::E,
        _ => panic!("Guitars with more than 6 strings are not allowed")
    };
    let notes: [Note; 7] = NOTES.clone();
    let position_of_open_note = notes.iter().position(|n| *n == open_string_note).unwrap();
    let notes_with_right_order: Vec<Note> = if position_of_open_note == 0 {
        notes.clone().to_vec()
    } else {
        let (first, second) = notes.split_at(position_of_open_note);
        [second, first].concat()
    };
    let fret_board: Vec<String> = notes_with_right_order.iter().map(|n| {
        return match n {
            Note::E | Note::B => vec![n.to_string()],
            n => vec![n.to_string(), n.to_string() + "#"]
        }
    }).collect::<Vec<Vec<String>>>().concat();

    // TODO: optimize so that we can just use fretboard and move it around to get notes ahead on it
    let big_fret_board = [fret_board.clone(), fret_board.clone(), fret_board.clone()].concat();

    let idx = note.0 as usize;
    big_fret_board[idx].clone()
}

// TODO: dynamic roman number fret derivation
fn fret_string_from_number(i: i32) -> String {
    match i {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        6 => "VI",
        7 => "VII",
        8 => "VIII",
        9 => "IX",
        10 => "X",
        11 => "XI",
        12 => "XII",
        13 => "XIII",
        14 => "XIV",
        15 => "XV",
        16 => "XVI",
        17 => "XVII",
        18 => "XVIII",
        19 => "XIX",
        20 => "XX",
        _ => unimplemented!("Implement more frets you piece of shit")
    }.to_string()
}