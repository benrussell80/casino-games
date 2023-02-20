use std::{rc::Rc, cell::RefCell};

use crate::{wasm4::*, GAMEPAD1_PREV, GAMEPAD2_PREV, GAMEPAD3_PREV, GAMEPAD4_PREV};

#[derive(Copy, Clone, Debug)]
pub struct Inputs {
    pub press_x: bool,
    pub press_z: bool,
    pub press_left: bool,
    pub press_right: bool,
    pub press_up: bool,
    pub press_down: bool,
    
    pub tap_x: bool,
    pub tap_z: bool,
    pub tap_left: bool,
    pub tap_right: bool,
    pub tap_up: bool,
    pub tap_down: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum User {
    One,
    Two,
    Three,
    Four,
}

impl User {
    pub fn gamepad(&self) -> u8 {
        unsafe {
            match self {
                Self::One => *GAMEPAD1,
                Self::Two => *GAMEPAD2,
                Self::Three => *GAMEPAD3,
                Self::Four => *GAMEPAD4,
            }
        }
    }

    pub fn gamepad_prev(&self) -> u8 {
        unsafe {
            match self {
                Self::One => GAMEPAD1_PREV,
                Self::Two => GAMEPAD2_PREV,
                Self::Three => GAMEPAD3_PREV,
                Self::Four => GAMEPAD4_PREV,
            }
        }
    }

    pub fn get_inputs(&self) -> Inputs {
        let gamepad = self.gamepad();
        let prev = self.gamepad_prev();
        let pressed_this_frame = gamepad & (gamepad ^ prev);
        Inputs {
            press_x: gamepad & BUTTON_1 != 0,
            press_z: gamepad & BUTTON_2 != 0,
            press_left: gamepad & BUTTON_LEFT != 0,
            press_right: gamepad & BUTTON_RIGHT != 0,
            press_up: gamepad & BUTTON_UP != 0,
            press_down: gamepad & BUTTON_DOWN != 0,

            tap_x: pressed_this_frame & BUTTON_1 != 0,
            tap_z: pressed_this_frame & BUTTON_2 != 0,
            tap_left: pressed_this_frame & BUTTON_LEFT != 0,
            tap_right: pressed_this_frame & BUTTON_RIGHT != 0,
            tap_up: pressed_this_frame & BUTTON_UP != 0,
            tap_down: pressed_this_frame & BUTTON_DOWN != 0,
        }
    }
}


pub trait Model<State> {
    fn update(&mut self, inputs: [Inputs; 4]) -> Option<State>;
    fn draw(&self);
    fn share_state(&mut self, state: State);
}