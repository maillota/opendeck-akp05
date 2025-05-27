use mirajazz::{error::MirajazzError, types::DeviceInput};

use crate::mappings::{ENCODER_COUNT, KEY_COUNT};

pub fn process_input(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    log::debug!("Processing input: {}, {}", input, state);

    match input {
        (0..=10) => read_button_press(input, state),                   // main buttons 1-5 top row, 6-10 bottom row
        //64 | 65 | 66 | 67 => read_button_press(input, state),  // touch screen - single press, no up down
        55 | 53 | 51 | 56 => read_encoder_press(input, state), // dial presses - left to right
        (160..=161) | (80..=81) |
          (144..=145) | (112..=113) => read_encoder_value(input),
        _ => Err(MirajazzError::BadData),
    }
}

fn read_button_states(states: &[u8]) -> Vec<bool> {
    let mut bools = vec![];

    for i in 0..KEY_COUNT {
        bools.push(states[i + 1] != 0);
    }

    bools
}

fn read_button_press(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let mut button_states = vec![0x01];
    button_states.extend(vec![0u8; KEY_COUNT + 1]);

    if input == 0 {
        return Ok(DeviceInput::ButtonStateChange(read_button_states(
            &button_states,
        )));
    }

    let pressed_index: usize = match input {
        // main ten buttons
        1 => 0xb,
        2 => 0xc,
        3 => 0xd,
        4 => 0xe,
        5 => 0xf,
        6 => 0x6,
        7 => 0x7,
        8 => 0x8,
        9 => 0x8,
        10 => 0xa,
        // four "buttons" on the touch screen
        //64 => 1,
        //65 => 2,
        //66 => 3,
        //67 => 4,
        _ => return Err(MirajazzError::BadData),
    };

    button_states[pressed_index] = state;

    Ok(DeviceInput::ButtonStateChange(read_button_states(
        &button_states,
    )))
}

fn read_encoder_value(input: u8) -> Result<DeviceInput, MirajazzError> {
    let mut encoder_values = vec![0i8; ENCODER_COUNT];

    let (encoder, value): (usize, i8) = match input {
        // Left encoder
        0xa0 => (0, -1),
        0xa1 => (0, 1),
        // Second encoder
        0x50 => (1, -1),
        0x51 => (1, 1),
        // Third encoder
        0x90 => (2, -1),
        0x91 => (2, 1),
        // Right encoder
        0x70 => (3, -1),
        0x71 => (3, 1),
        _ => return Err(MirajazzError::BadData),
    };

    encoder_values[encoder] = value;
    Ok(DeviceInput::EncoderTwist(encoder_values))
}

fn read_encoder_press(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let mut encoder_states = vec![false; ENCODER_COUNT];

    let encoder: usize = match input {
        0x37 => 0, // Left encoder
        0x35 => 1, // Second encoder
        0x33 => 2, // Third encoder
        0x36 => 3, // Right encoder
        _ => return Err(MirajazzError::BadData),
    };

    encoder_states[encoder] = state != 0;
    Ok(DeviceInput::EncoderStateChange(encoder_states))
}
