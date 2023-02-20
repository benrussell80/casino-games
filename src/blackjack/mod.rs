use std::fmt;

use crate::{model::Model, wasm4::*, PlayerState};
use fastrand::Rng;


struct Button {
    text: &'static str,
    disabled: bool
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
enum CardValue {
    Ace = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King
}

impl fmt::Display for CardValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Ace => "A",
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
            Self::Five => "5",
            Self::Six => "6",
            Self::Seven => "7",
            Self::Eight => "8",
            Self::Nine => "9",
            Self::Ten => "T",
            Self::Jack => "J",
            Self::Queen => "Q",
            Self::King => "K",
        })
    }
}

impl CardValue {
    fn equal_to(&self, other: &Self) -> bool {
        use CardValue::*;
        match (self, other) {
            (l, r) if l == r => true,
            (
                Ten | Jack | Queen | King,
                Ten | Jack | Queen | King,
            ) => true,
            _ => false
        }
    }
}

impl CardValue {
    fn values() -> [Self; 13] {
        use CardValue::*;
        [Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King]
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum CardSuit {
    Club,
    Diamond,
    Heart,
    Spade,
}

impl CardSuit {
    fn suits() -> [Self; 4] {
        use CardSuit::*;
        [Club, Diamond, Heart, Spade]
    }
}

#[derive(Clone)]
struct Card {
    value: CardValue,
    suit: CardSuit,
}

impl Card {
    fn new_shuffled_horn(rng: &Rng) -> Vec<Self> {
        let mut horn = Vec::with_capacity(7 * 52);
        for _ in 0..7 {
            for suit in CardSuit::suits() {
                for value in CardValue::values() {
                    horn.push(Card { value, suit })
                }
            }
        }
        rng.shuffle(&mut horn);
        horn
    }
}

#[derive(Clone)]
struct Hand {
    cards: Vec<Card>
}

impl Hand {
    fn new() -> Self {
        Self {
            cards: Vec::with_capacity(4),
        }
    }
}

impl Hand {
    fn max_points(&self) -> u8 {
        let (pts, is_soft) = self.points();
        if is_soft {
            if pts <= 11 {
                pts + 10
            } else {
                pts
            }
        } else {
            pts
        }
    }

    fn points(&self) -> (u8, bool) {  // value, is_soft
        use CardValue::*;
        let mut pts = 0;
        let mut has_ace = false;
        for card in self.cards.iter() {
            match card.value {
                Ace => {
                    has_ace = true;
                    pts += 1;
                }
                Ten | Jack | Queen | King => {
                    pts += 10;
                }
                other => {
                    pts += other as u8;
                }
            }
        }
        (pts, has_ace)
    }

    fn dealer_must_hit(&self) -> bool {
        let (pts, is_soft) = self.points();
        pts < 17 || (pts == 17 && is_soft)
    }

    fn is_bust(&self) -> bool {
        let (pts, _) = self.points();
        pts > 21
    }

    fn is_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.max_points() == 21
    }

    fn showdown_result(&self, dealer_hand: &Self) -> HandResult {
        let player_points = self.max_points();
        let dealer_points = dealer_hand.max_points();
        if dealer_points == player_points {
            HandResult::Push
        } else if dealer_points > player_points || self.is_bust() {
            HandResult::Lose
        } else {
            HandResult::Win
        }
    }

    fn can_split(&self) -> bool {
        self.cards.len() == 2 && self.cards[0].value.equal_to(&self.cards[1].value)
    }

    fn can_double_down(&self) -> bool {
        let (pts, is_soft) = self.points();
        self.cards.len() == 2 && (pts == 10 || pts == 11 || (pts == 1 && is_soft))
    }
}

struct PlayingState {
    hit_button: Button,
    stand_button: Button,
    split_button: Button,
    double_down_button: Button,
    button_index: usize,  // 0: hit, 1: stand, 2: split, 3: double_down
    dealer_hand: Hand,
    player_hands: Vec<Hand>,
    player_hand_index: usize,
}

impl PlayingState {
    fn current_button(&self) -> &Button {
        match self.button_index {
            0 => &self.hit_button,
            1 => &self.stand_button,
            2 => &self.split_button,
            3 => &self.double_down_button,
            _ => unreachable!(),
        }
    }

    fn incr_button_index(&mut self) {
        self.button_index = (self.button_index + 1) % 4;
    }

    fn decr_button_index(&mut self) {
        if self.button_index == 0 {
            self.button_index = 3
        } else {
            self.button_index -= 1
        }
    }

    fn new(dealer_hand: Hand, player_hand: Hand) -> Self {
        Self {
            hit_button: Button { text: "Hit", disabled: false },
            stand_button: Button { text: "Stand", disabled: false },
            split_button: Button { text: "Split", disabled: true },
            // surrender_button: Button { text: "Surrender", disabled: true },
            double_down_button: Button { text: "Double Down", disabled: true },
            button_index: 0,
            dealer_hand: Hand { cards: vec![] },
            player_hands: vec![
                player_hand
            ],
            player_hand_index: 0,
        }
    }
}

enum HandResult {
    Lose,
    Win,
    Push,
}

struct EndState {
    // x to play again or z to return to main menu
    player_hands: Vec<(Hand, HandResult)>,
}

struct DealingState {
    frame: u8,
    dealer_hand: Hand,
    player_hand: Hand,
}

impl DealingState {
    fn new() -> Self {
        Self {
            frame: 0,
            dealer_hand: Hand::new(),
            player_hand: Hand::new(),
        }
    }
}

struct DealerResolvingState {
    player_hands: Vec<Hand>,
    dealer_hand: Hand,
}

struct InsuranceState {
    dealer_hand: Hand,
    player_hand: Hand,
    player_insurance_bet: u32
}

impl InsuranceState {
    fn new(dealer_hand: Hand, player_hand: Hand) -> Self {
        Self {
            dealer_hand,
            player_hand,
            player_insurance_bet: 0,
        }
    }
}

enum BlackJackState {
    Betting,
    Dealing(DealingState),
    Insurance(InsuranceState),
    Playing(PlayingState),
    DealerResolving(DealerResolvingState),
    End(EndState),
}

fn draw_card(horn: &mut Vec<Card>, rng: &Rng) -> Card {
    if let Some(card) = horn.pop() {
        card
    } else {
        *horn = Card::new_shuffled_horn(rng);
        horn.pop().unwrap()
    }
}

pub struct BlackJack {
    horn: Vec<Card>,
    player_bet: u32,
    total_bet: u32,
    player_bank: u32,
    state: BlackJackState,
    rng: Rng,
}

impl BlackJack {
    pub fn new(random_seed: u64) -> Box<dyn Model<PlayerState>> {
        let rng = Rng::with_seed(random_seed);
        Box::new(Self {
            horn: Card::new_shuffled_horn(&rng),
            player_bank: 0,
            player_bet: 0,
            total_bet: 0,
            state: BlackJackState::Betting,
            rng
        })
    }
}

const BET_INCREMENT: u32 = 5;
const MINIMUM_BET: u32 = 5;




fn display_cards(dealer_hand: &Hand, player_hands: &[&Hand], showdown: bool) {
    for (index, card) in dealer_hand.cards.iter().enumerate() {
        if showdown && index == 0 {

        } else {
            
        }
        unsafe {
            *DRAW_COLORS = 0x01;
        }
        let x = (60 + index * 14) as _;
        rect(x, 65, 12, 16);
        unsafe {
            *DRAW_COLORS = if card.suit == CardSuit::Club || card.suit == CardSuit::Spade {
                0x03
            } else {
                0x02
            }
        }
        text(format!("{}", card.value), x + 2, 67);
    }
}

impl Model<PlayerState> for BlackJack {
    fn update(&mut self, inputs: [crate::model::Inputs; 4]) -> Option<PlayerState> {
        let player_one_inputs = inputs[0];
        // if player_one_inputs.press_z {
        //     return Some(rc_refcell(StartScreen::new(get_games(), self.player_bank)))
        // }
        match self {
            Self { state: BlackJackState::Betting, .. } => {
                // buttons for changing bet amount
                if player_one_inputs.tap_up {
                    self.player_bet = self.player_bet.saturating_add(BET_INCREMENT)
                } else if player_one_inputs.tap_down {
                    self.player_bet = self.player_bet.saturating_sub(BET_INCREMENT);
                }
                self.player_bet = self.player_bet.max(MINIMUM_BET);
                self.player_bet = self.player_bet.min(self.player_bank);

                // buttons for making bet
                if player_one_inputs.tap_x {
                    if self.player_bet > self.player_bank {
                        // make sound
                    } else {
                        self.player_bank -= self.player_bet;
                        self.total_bet = self.player_bet;
                        self.state = BlackJackState::Dealing(DealingState::new());
                    }
                }
            }
            Self { state: BlackJackState::Playing(state), player_bet, total_bet, .. } => {
                if state.player_hand_index >= state.player_hands.len() {
                    self.state = BlackJackState::DealerResolving(DealerResolvingState {
                        player_hands: state.player_hands.clone(),
                        dealer_hand: state.dealer_hand.clone()
                    });
                } else {
                    let mut hand = &state.player_hands[state.player_hand_index];
                    if hand.is_bust() {
                        *total_bet -= *player_bet;
                        state.player_hand_index += 1;
                    }
                    if hand.can_split() {
                        state.split_button.disabled = false;
                    }
                    if hand.can_double_down() {
                        state.double_down_button.disabled = false;
                    }
                    // left right buttons
                    if player_one_inputs.tap_left {
                        state.decr_button_index()
                    }
                    if player_one_inputs.tap_right {
                        state.incr_button_index()
                    }
                }
                // if player can split then enable split button
                // if player can double down then enable double down button
                // if player can surrender then enable surrender button
                // if player can hit then enable hit button
                // if state.player_hands.
            }
            Self { state: BlackJackState::End(state), .. } => {
                if player_one_inputs.press_x {
                    self.state = BlackJackState::Betting
                }
            }
            Self { state: BlackJackState::Dealing(state), rng, horn, .. } => {
                state.frame += 1;
                if state.frame == 10 {
                    state.dealer_hand.cards.push(draw_card(horn, rng));
                } else if state.frame == 20 {
                    state.player_hand.cards.push(draw_card(horn, rng));
                } else if state.frame == 30 {
                    state.dealer_hand.cards.push(draw_card(horn, rng));
                } else if state.frame == 40 {
                    state.player_hand.cards.push(draw_card(horn, rng));
                } else if state.frame == 50 {
                    if state.dealer_hand.cards[0].value == CardValue::Ace {
                        self.state = BlackJackState::Insurance(InsuranceState::new(
                            state.dealer_hand.clone(),
                            state.player_hand.clone(),
                        ));
                    } else {
                        self.state = BlackJackState::Playing(PlayingState::new(
                            state.dealer_hand.clone(),
                            state.player_hand.clone(),
                        ));
                    }
                }
            }
            Self { state: BlackJackState::DealerResolving(state), .. } => {

            }
            Self { state: BlackJackState::Insurance(state), .. } => {
                // buttons for changing bet amount
                if player_one_inputs.tap_up {
                    state.player_insurance_bet = state.player_insurance_bet.saturating_add(BET_INCREMENT)
                } else if player_one_inputs.tap_down {
                    state.player_insurance_bet = state.player_insurance_bet.saturating_sub(BET_INCREMENT);
                }
                state.player_insurance_bet = state.player_insurance_bet.max(MINIMUM_BET);
                state.player_insurance_bet = state.player_insurance_bet.min(self.player_bank);

                // buttons for making bet
                if player_one_inputs.tap_x {
                    if state.player_insurance_bet > self.player_bank {
                        // make sound
                    } else {
                        self.player_bank -= state.player_insurance_bet;
                        self.state = BlackJackState::Playing(PlayingState::new(state.dealer_hand.clone(), state.player_hand.clone()));
                    }
                }
            }
        }
        None
    }

    fn draw(&self) {
        let table_height = 60;
        // draw table
        unsafe { *DRAW_COLORS = 0x32; }
        oval(0, 0, 160, (table_height * 2) as _);
        unsafe { *DRAW_COLORS = 0x11; }
        rect(0, 0, 160, table_height as _);
        unsafe { *DRAW_COLORS = 0x44; }
        line(0, table_height, 160, table_height);
        // draw input bar
        unsafe { *DRAW_COLORS = 0x32; }
        rect(0, 140, 160, 20);

        // draw
        text(format!("Chips: ${}", self.player_bank), 10, 10);
        
        // draw bank
        text(format!("Bet Amount: ${}", self.player_bet), 10, 20);
        
        // draw cards in horn
        text(format!("Cards in Shoe: {}", self.horn.len()), 10, 30);

        // draw bet amount
        match self {
            Self { state: BlackJackState::Betting, .. } => {
                unsafe { *DRAW_COLORS = 0x31; }
                let t = b"Use \x86\x87 to change bet.";
                unsafe {
                    extern_text(t.as_ptr(), t.len(), 0, 142);
                }

                let t = b"Use \x80 to make bet.";
                unsafe {
                    extern_text(t.as_ptr(), t.len(), 0, 151);
                }
            },
            Self { state: BlackJackState::Dealing(state), .. } => {
                display_cards(
                    &state.dealer_hand,
                    &[&state.player_hand],
                );
            },
            _ => {}
        }
    }

    fn share_state(&mut self, state: PlayerState) {
        self.player_bank = state.bank;
    }
}
