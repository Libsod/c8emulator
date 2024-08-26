mod core;

use crate::core::emu::{Emu, SCREEN_HEIGHT, SCREEN_WIDTH};

use std::env;
use std::fs::File;
use std::io::Read;

use macroquad::prelude::*;

const SCALE: u32 = 15;
const KEYS: [KeyCode; 16] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Q,
    KeyCode::W,
    KeyCode::E,
    KeyCode::R,
    KeyCode::A,
    KeyCode::S,
    KeyCode::D,
    KeyCode::F,
    KeyCode::Z,
    KeyCode::X,
    KeyCode::C,
    KeyCode::V,
];
const TICKS_PER_FRAME: usize = 10;

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();
    let mut chip8 = Emu::new();

    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    'gameloop: loop {
        if is_key_down(KeyCode::Escape) {
            break 'gameloop;
        }

        for key in KEYS {
            if is_key_pressed(key) {
                if let Some(k) = key2btn(key) {
                    chip8.keypress(k, true);
                }
            }

            if is_key_released(key) {
                if let Some(k) = key2btn(key) {
                    chip8.keypress(k, false);
                }
            }
        }

        for _i in 0..TICKS_PER_FRAME {
            chip8.tick();
        }
        chip8.tick_timers();

        handle_input(&mut chip8);
        draw_screen(&chip8).await;

        next_frame().await;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Chip-8 Emulator".to_owned(),
        window_width: SCREEN_WIDTH as i32 * SCALE as i32,
        window_height: SCREEN_HEIGHT as i32 * SCALE as i32,
        ..Default::default()
    }
}

fn handle_input(chip8: &mut Emu) {
    for key in KEYS {
        let pressed = is_key_down(key);
        if let Some(k) = key2btn(key) {
            chip8.keypress(k, pressed);
        }
    }
}

async fn draw_screen(emu: &Emu) {
    clear_background(BLACK);

    let screen_buf = emu.get_display();
    for (i, &pixel) in screen_buf.iter().enumerate() {
        if pixel {
            let x = (i % SCREEN_WIDTH) as f32 * SCALE as f32;
            let y = (i / SCREEN_WIDTH) as f32 * SCALE as f32;

            draw_rectangle(x, y, SCALE as f32, SCALE as f32, WHITE);
        }
    }
}

/*
    Keyboard                    Chip-8
    +---+---+---+---+           +---+---+---+---+
    | 1 | 2 | 3 | 4 |           | 1 | 2 | 3 | C |
    +---+---+---+---+           +---+---+---+---+
    | Q | W | E | R |           | 4 | 5 | 6 | D |
    +---+---+---+---+     =>    +---+---+---+---+
    | A | S | D | F |           | 7 | 8 | 9 | E |
    +---+---+---+---+           +---+---+---+---+
    | Z | X | C | V |           | A | 0 | B | F |
    +---+---+---+---+           +---+---+---+---+
*/

fn key2btn(key: KeyCode) -> Option<usize> {
    match key {
        KeyCode::Key1 => Some(0x1),
        KeyCode::Key2 => Some(0x2),
        KeyCode::Key3 => Some(0x3),
        KeyCode::Key4 => Some(0xC),
        KeyCode::Q => Some(0x4),
        KeyCode::W => Some(0x5),
        KeyCode::E => Some(0x6),
        KeyCode::R => Some(0xD),
        KeyCode::A => Some(0x7),
        KeyCode::S => Some(0x8),
        KeyCode::D => Some(0x9),
        KeyCode::F => Some(0xE),
        KeyCode::Z => Some(0xA),
        KeyCode::X => Some(0x0),
        KeyCode::C => Some(0xB),
        KeyCode::V => Some(0xF),
        _ => None,
    }
}
