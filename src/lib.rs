#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
mod model;
use wasm4::*;
use model::{Model, User};
mod blackjack;
use blackjack::{BlackJack};


static mut GAMEPAD1_PREV: u8 = 0;
static mut GAMEPAD2_PREV: u8 = 0;
static mut GAMEPAD3_PREV: u8 = 0;
static mut GAMEPAD4_PREV: u8 = 0;

fn start_frame() {

}

fn end_frame() {
    unsafe {
        GAMEPAD1_PREV = *GAMEPAD1;
        GAMEPAD2_PREV = *GAMEPAD2;
        GAMEPAD3_PREV = *GAMEPAD3;
        GAMEPAD4_PREV = *GAMEPAD4;
    }
}

#[no_mangle]
fn start() {
    unsafe {
        *PALETTE = [
            0xffffff,  // white
            0xc60e0e,  // red
            0x000000,  // black
            0xffef00,  // yellow
        ];
        GAME.init()
    }
}

#[derive(Copy, Clone)]
pub struct PlayerState {
    bank: u32,
}

struct MainGame {
    frame_count: u64,
    games: Option<[(&'static str, fn(u64) -> Box<dyn Model<PlayerState>>); 1]>,
    num_games: usize,
    current_index: usize,
    current_game: Option<Box<dyn Model<PlayerState>>>,
    player_state: PlayerState,
}

impl MainGame {
    pub fn init(&mut self) {
        if self.games.is_none() {
            self.games = Some([
                ("Blackjack", BlackJack::new)
            ]);
            self.num_games = 1;
            self.player_state = PlayerState { bank: 100 };
        }
    }
}

impl Model<PlayerState> for MainGame {
    fn draw(&self) {
        match self {
            Self { current_game: Some(g), .. } => {
                g.draw()
            }
            Self { current_game: None, .. } => {
                for (index, (name, _)) in self.games.unwrap().iter().enumerate() {
                    if index == self.current_index {
                        unsafe {
                            *DRAW_COLORS = (*DRAW_COLORS & 0b1111111100000000) | 0x32
                        }
                    } else {
                        unsafe {
                            *DRAW_COLORS = (*DRAW_COLORS & 0b1111111100000000) | 0x02
                        }
                    }
                    text(&name, 20, (20 + 10 * index) as _);
                }
            }
        }
    }

    fn update(&mut self, inputs: [crate::model::Inputs; 4]) -> Option<PlayerState> {
        self.frame_count += 1;
        match self {
            Self { current_game: Some(g), .. } => {
                if let Some(state) = g.update(inputs) {
                    self.current_game = None;
                    self.share_state(state);
                }
            }
            Self {
                current_game,
                player_state,
                games: Some(games),
                num_games,
                current_index,
                ..
            } => {
                let first_player_inputs = inputs[0];
                if first_player_inputs.tap_down {
                    *current_index = (*current_index + 1) % *num_games;
                }
                if first_player_inputs.tap_up {
                    if *current_index == 0 {
                        *current_index = *num_games - 1;
                    } else {
                        *current_index = (*current_index - 1) % *num_games;
                    }
                }
                if first_player_inputs.tap_x {
                    let (_, func) = &games[*current_index];
                    let mut game = (*func)(
                        self.frame_count + unsafe { *MOUSE_X + *MOUSE_Y } as u64
                    );
                    game.share_state(*player_state);
                    *current_game = Some(game);
                }
            },
            _ => unreachable!()
        }
        None
    }

    fn share_state(&mut self, state: PlayerState) {
        self.player_state = state;
    }
}


static mut GAME: MainGame = MainGame {
    frame_count: 0,
    games: None,
    num_games: 0,
    current_index: 0,
    current_game: None,
    player_state: PlayerState { bank: 0 }
};

#[no_mangle]
unsafe fn update() {
    start_frame();
    let inputs = [
        User::One.get_inputs(),
        User::Two.get_inputs(),
        User::Three.get_inputs(),
        User::Four.get_inputs(),
    ];

    GAME.update(inputs);
    GAME.draw();

    end_frame();
}
