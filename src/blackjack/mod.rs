use std::fmt;

use crate::{model::Model, wasm4::*, PlayerState};
use fastrand::Rng;


fn buzz() {
    tone(140, 6, 40, 0);
}

struct Button {
    text: &'static str,
    disabled: bool
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum CardValue {
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
pub enum CardSuit {
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

#[derive(Clone, Debug)]
pub struct Card {
    pub value: CardValue,
    pub suit: CardSuit,
}

impl Card {
    fn draw_sprite(&self, x: i32, y: i32, face_up: bool) {
        let card_sprite = [5, 85, 64, 106, 170, 69, 170, 169, 86, 170, 165, 90, 170, 149, 106, 170, 85, 170, 169, 86, 170, 165, 90, 170, 149, 106, 170, 85, 170, 169, 86, 170, 165, 90, 170, 149, 106, 170, 81, 170, 169, 1, 85, 80];
        if face_up {
            unsafe {
                *DRAW_COLORS = 0x0130;
            }
            blit(&card_sprite, x, y, 11, 16, BLIT_2BPP);
            match self.suit {
                CardSuit::Club | CardSuit::Spade => {
                    unsafe {
                        *DRAW_COLORS |= 0x3000;
                    }
                }
                CardSuit::Diamond | CardSuit::Heart => {
                    unsafe {
                        *DRAW_COLORS |= 0x2000;
                    }
                }
            }
            let value_sprite = match self.value {
                CardValue::Ace => [60, 195, 195, 255, 195],
                CardValue::Two => [60, 195, 3, 12, 63],
                CardValue::Three => [255, 3, 63, 3, 255],
                CardValue::Four => [3, 15, 51, 255, 3],
                CardValue::Five => [255, 192, 252, 3, 252],
                CardValue::Six => [63, 192, 252, 195, 252],
                CardValue::Seven => [255, 15, 12, 48, 48],
                CardValue::Eight => [60, 195, 60, 195, 60],
                CardValue::Nine => [60, 195, 63, 3, 3],
                CardValue::Ten => [255, 12, 12, 12, 12],
                CardValue::Jack => [255, 12, 12, 204, 48],
                CardValue::Queen => [48, 204, 204, 204, 51],
                CardValue::King => [195, 204, 240, 204, 195]
            };
            blit(&value_sprite, x + 3, y + 2, 4, 5, BLIT_2BPP);
            // draw icon underneath
        } else {
            unsafe {
                *DRAW_COLORS = 0x0430;
            }
            blit(&card_sprite, x, y, 11, 16, BLIT_2BPP);
        }
    }

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
pub struct Hand {
    pub cards: Vec<Card>
}

impl Hand {
    fn new() -> Self {
        Self {
            cards: Vec::with_capacity(4),
        }
    }
}

impl Hand {
    fn dealer_showing_ace(&self) -> bool {
        self.cards[1].value == CardValue::Ace 
    }

    fn points(&self) -> Vec<u8> {
        use CardValue::*;
        let mut pts = vec![];
        let mut sum = 0;
        for card in self.cards.iter() {
            match card.value {
                Ace => {
                    sum += 1;
                }
                Ten | Jack | Queen | King => {
                    sum += 10;
                }
                other => {
                    sum += other as u8;
                }
            }
        }
        pts.push(sum);
        for card in self.cards.iter() {
            if Ace == card.value {
                for prev_pt in pts.clone() {
                    pts.push(prev_pt + 10)
                }
            }
        }
        pts
    }

    fn dealer_must_hit(&self) -> bool {
        for pt in self.points() {
            if 17 <= pt && pt <= 21 {
                return false
            }
        }
        true
    }

    fn is_bust(&self) -> bool {
        self.points().into_iter().all(|pt| pt > 21)
    }

    fn is_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.points().into_iter().any(|pt| pt == 21)
    }

    fn showdown_result(&self, dealer_hand: Option<&Self>) -> HandResult {
        if self.is_blackjack() {
            HandResult::BlackJack
        } else if self.is_bust() {
            HandResult::Lose
        } else {
            let player_points = self.points().into_iter().filter(|pt| *pt <= 21).max();
            let dealer_hand = dealer_hand.unwrap();
            let dealer_points = dealer_hand.points().into_iter().filter(|pt| *pt <= 21).max();
            match (player_points, dealer_points) {
                (Some(pp), Some(dp)) => {
                    if pp == dp {
                        HandResult::Push
                    } else if pp < dp {
                        HandResult::Lose
                    } else {
                        HandResult::Win
                    }
                }
                (Some(pp), None) => {
                    if pp != 21 {
                        HandResult::Win
                    } else {
                        HandResult::BlackJack
                    }
                }
                (None, Some(_))
                | (None, None) => {
                    HandResult::Lose
                }
            }
        }
    }

    fn can_split(&self) -> bool {
        self.cards.len() == 2 && self.cards[0].value.equal_to(&self.cards[1].value)
    }

    fn can_double_down(&self) -> bool {
        self.cards.len() == 2 && self.points().into_iter().any(|pt| pt == 10 || pt == 11)
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
    fn new(dealer_hand: Hand, player_hand: Hand) -> Self {
        Self {
            hit_button: Button { text: "Hit", disabled: false },
            stand_button: Button { text: "Stand", disabled: false },
            split_button: Button { text: "Split", disabled: true },
            // surrender_button: Button { text: "Surrender", disabled: true },
            double_down_button: Button { text: "Double Down", disabled: true },
            button_index: 0,
            dealer_hand: dealer_hand,
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
    BlackJack,
}

struct EndState {
    dealer_hand: Hand,
    player_hands: Vec<(Hand, HandResult)>,
    bought_insurance: bool,
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
    frame_count: u64,
}

struct InsuranceState {
    dealer_hand: Hand,
    player_hand: Hand,
}

impl InsuranceState {
    fn new(dealer_hand: Hand, player_hand: Hand) -> Self {
        Self {
            dealer_hand,
            player_hand,
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

const BET_INCREMENT: u32 = 10;
const MINIMUM_BET: u32 = 10;




fn display_cards(dealer_hand: &Hand, player_hands: &[&Hand], active_player_hand_index: usize, showdown: bool) {
    for (index, card) in dealer_hand.cards.iter().enumerate() {
        let x = (60 + index * 14) as _;
        let y = 67;
        let face_up = !(!showdown && index == 0);
        card.draw_sprite(x, y, face_up)
    }
    let num_hands = player_hands.len();
    for (hand_index, hand) in player_hands.iter().enumerate() {
        let space_size = 160 / num_hands;
        let x = space_size * (hand_index + 1) - space_size * 2 / 3;
        if hand_index == active_player_hand_index {
            unsafe {
                *DRAW_COLORS = 0x0040;
            }
            blit(&[4, 1, 5, 84, 84, 4, 0], x as _, 90, 5, 5, BLIT_2BPP);
        }
        for (card_index, card) in hand.cards.iter().enumerate() {
            let x = x + card_index * 14; // if this is > 160 - sprite width go to next row
            let y = 97;
            card.draw_sprite(x as _, y, true);
        }
    }
}

impl Model<PlayerState> for BlackJack {
    fn update(&mut self, inputs: [crate::model::Inputs; 4]) -> Option<PlayerState> {
        let player_one_inputs = inputs[0];
        match self {
            Self { state: BlackJackState::Betting, .. } => {
                if player_one_inputs.tap_z {
                    return Some(PlayerState { bank: self.player_bank })
                }
                if self.player_bank < MINIMUM_BET {
                    if player_one_inputs.tap_x {
                        buzz();
                    }
                } else {
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
                            buzz();
                        } else {
                            self.player_bank -= self.player_bet;
                            self.total_bet = self.player_bet;
                            self.state = BlackJackState::Dealing(DealingState::new());
                        }
                    }
                }
            }
            Self { state: BlackJackState::Playing(state), player_bet, .. } => {
                if state.player_hand_index >= state.player_hands.len() {
                    let mut showdown_needed = false;
                    for hand in state.player_hands.iter() {
                        if hand.is_bust() || hand.is_blackjack() {
                            continue;
                        }
                        showdown_needed = true;
                        break
                    }
                    if showdown_needed {
                        self.state = BlackJackState::DealerResolving(DealerResolvingState {
                            player_hands: state.player_hands.clone(),
                            dealer_hand: state.dealer_hand.clone(),
                            frame_count: 0
                        });
                    } else {
                        let mut player_hands = Vec::new();
                        for hand in state.player_hands.iter() {
                            player_hands.push((
                                hand.clone(),
                                hand.showdown_result(None)
                            ))
                        }
                        self.state = BlackJackState::End(EndState {
                            dealer_hand: state.dealer_hand.clone(),
                            player_hands,
                            bought_insurance: false
                        })
                    }
                } else {
                    let hand = &mut state.player_hands[state.player_hand_index];
                    if hand.is_bust() || hand.is_blackjack() {
                        state.player_hand_index += 1;
                    }
                    if hand.can_split() && self.player_bank >= *player_bet {
                        state.split_button.disabled = false;
                    } else {
                        state.split_button.disabled = true;
                    }
                    if hand.can_double_down() && self.player_bank >= *player_bet {
                        state.double_down_button.disabled = false;
                    } else {
                        state.double_down_button.disabled = true;
                    }
                    if player_one_inputs.tap_x {
                        match state.button_index {
                            0 if !state.hit_button.disabled => {  // Hit
                                hand.cards.push(draw_card(&mut self.horn, &self.rng));
                            }
                            1 if !state.stand_button.disabled => {  // Stand
                                state.player_hand_index += 1
                            }
                            2 if !state.split_button.disabled => {  // Split
                                // take from hand 1
                                let new_hand_card = hand.cards.pop().unwrap();
                                hand.cards.push(draw_card(&mut self.horn, &self.rng));

                                // give to hand 2
                                let mut new_hand = Hand::new();
                                new_hand.cards.push(new_hand_card);
                                new_hand.cards.push(draw_card(&mut self.horn, &self.rng));
                                state.player_hands.push(new_hand);

                                self.total_bet += *player_bet;
                            }
                            3 if !state.double_down_button.disabled => {  // Double Down
                                hand.cards.push(draw_card(&mut self.horn, &self.rng));
                                self.total_bet += self.player_bet;
                                state.player_hand_index += 1;
                            },
                            _ => {
                                buzz();
                            }
                        }
                    } else {
                        // left right buttons
                        if player_one_inputs.tap_right && state.button_index % 2 == 0 {
                            state.button_index += 1;
                        }
                        if player_one_inputs.tap_left && state.button_index % 2 == 1 {
                            state.button_index -= 1;
                        }
                        if player_one_inputs.tap_down && state.button_index / 2 == 0 {
                            state.button_index += 2;
                        }
                        if player_one_inputs.tap_up && state.button_index / 2 == 1 {
                            state.button_index -= 2;
                        }
                    }
                }
            }
            Self { state: BlackJackState::End(state), .. } => {
                if self.player_bet != 0 {
                    if state.dealer_hand.is_blackjack() {
                        if state.bought_insurance {
                            self.player_bank += self.player_bet * 3 / 2
                        }
                    } else {
                        for (_, res) in state.player_hands.iter() {
                            self.player_bank += match res {
                                HandResult::BlackJack => self.player_bet * 6 / 5 + self.player_bet,
                                HandResult::Lose => 0,
                                HandResult::Push => self.player_bet,
                                HandResult::Win => self.player_bet * 2,
                            }
                        }
                    }
                    self.total_bet = 0;
                    self.player_bet = 0;
                }
                if player_one_inputs.tap_x {
                    self.state = BlackJackState::Betting
                }
                if player_one_inputs.tap_z {
                    return Some(PlayerState { bank: self.player_bank })
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
                    if state.dealer_hand.dealer_showing_ace() {
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
                } else if state.frame > 50 {
                    unreachable!()
                }
            }
            Self { state: BlackJackState::DealerResolving(state), .. } => {
                state.frame_count += 1;
                if state.dealer_hand.dealer_must_hit() && !state.dealer_hand.is_bust() {
                    if state.frame_count % 30 == 0 {
                        state.dealer_hand.cards.push(draw_card(&mut self.horn, &self.rng));
                    }
                } else {
                    let mut player_hands = vec![];
                    for hand in state.player_hands.iter() {
                        let res = hand.showdown_result(Some(&state.dealer_hand));
                        player_hands.push((
                            hand.clone(),
                            res
                        ));
                    }
                    self.state = BlackJackState::End(EndState {
                        dealer_hand: state.dealer_hand.clone(),
                        player_hands,
                        bought_insurance: false
                    });
                }
            }
            Self { state: BlackJackState::Insurance(state), .. } => {
                // buttons for changing bet amount
                if player_one_inputs.tap_x || player_one_inputs.tap_z {
                    let bought_insurance = if player_one_inputs.tap_x {
                        self.player_bank -= self.player_bet / 2;
                        true
                    } else {
                        false
                    };
                    if state.dealer_hand.is_blackjack() {
                        self.state = BlackJackState::End(EndState {
                            dealer_hand: state.dealer_hand.clone(),
                            player_hands: vec![(
                                state.player_hand.clone(),
                                if state.player_hand.is_blackjack() {
                                    HandResult::BlackJack
                                } else {
                                    HandResult::Lose
                                }
                            )],
                            bought_insurance
                        });
                    } else {
                        self.state = BlackJackState::Playing(PlayingState::new(
                            state.dealer_hand.clone(),
                            state.player_hand.clone()
                        ));
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
        text(format!("Chips: ${}", self.player_bank), 10, 5);
        
        // draw bank
        text(format!("Bet Amount: ${}", self.player_bet), 10, 13);
        
        // draw cards in horn
        text(format!("Cards in Shoe: {}", self.horn.len()), 10, 21);

        // draw total bet
        text(format!("Total Bet: ${}", self.total_bet), 10, 29);

        // draw bet amount
        match self {
            Self { state: BlackJackState::Betting, .. } => {
                unsafe { *DRAW_COLORS = 0x31; }
                let t = b"\x86\x87: change bet";
                unsafe {
                    extern_text(t.as_ptr(), t.len(), 0, 142);
                }

                let t = b"\x80: make bet \x81: exit";
                unsafe {
                    extern_text(t.as_ptr(), t.len(), 0, 151);
                }
            }
            Self { state: BlackJackState::Dealing(DealingState { dealer_hand, player_hand, ..}), .. } => {
                display_cards(
                    dealer_hand,
                    &[player_hand],
                    0,
                    false,
                );
            }
            Self { state: BlackJackState::Insurance(InsuranceState { player_hand, dealer_hand }), .. } => {
                display_cards(
                    dealer_hand,
                    &[player_hand],
                    0,
                    false
                );
                unsafe { *DRAW_COLORS = 0x31; }
                text(format!("Insurance Bet: ${}", self.player_bet / 2), 10, 37);
                let t = b"Insurance bet?";
                unsafe {
                    extern_text(t.as_ptr(), t.len(), 0, 142);
                }

                let t = b" \x80: yes  \x81: no";
                unsafe {
                    extern_text(t.as_ptr(), t.len(), 0, 151);
                }
            }
            Self { state: BlackJackState::Playing(state), .. } => {
                display_cards(
                    &state.dealer_hand,
                    &state.player_hands.iter().collect::<Vec<_>>(),
                    state.player_hand_index,
                    false
                );
                // 0: hit, 1: stand, 2: split, 3: double_down
                for (index, button) in [&state.hit_button, &state.stand_button, &state.split_button, &state.double_down_button].iter().enumerate() {
                    if index == state.button_index {
                        unsafe {
                            *DRAW_COLORS = 0x0043
                        }
                    } else {
                        unsafe {
                            *DRAW_COLORS = 0x0003
                        }
                    }
                    // extern_text(t.as_ptr(), t.len(), 0, 142);
                    // extern_text(t.as_ptr(), t.len(), 0, 151);
                    text(button.text, (2 + (index % 2) * 60) as _, (142 + 9 * (index / 2)) as _);
                }
            }
            Self {
                state: BlackJackState::DealerResolving(DealerResolvingState {
                    dealer_hand,
                    player_hands,
                    ..
                }),
                ..
            } => {
                display_cards(
                    dealer_hand,
                    &player_hands.iter().collect::<Vec<_>>(),
                    0,
                    true
                );
            }
            Self { state: BlackJackState::End(EndState { dealer_hand, player_hands , .. }), .. } => {
                display_cards(
                    dealer_hand,
                    &player_hands.iter().map(|(x, _)| x).collect::<Vec<_>>(),
                    0,
                    true
                );

                unsafe { *DRAW_COLORS = 0x31; }
                let t = b"Use \x80 to play again.";
                unsafe {
                    extern_text(t.as_ptr(), t.len(), 0, 142);
                }
                let t = b"Use \x81 to exit.";
                unsafe {
                    extern_text(t.as_ptr(), t.len(), 0, 151);
                }
            }
        }
        unsafe {
            *DRAW_COLORS = 0x0430
        }
    }

    fn share_state(&mut self, state: PlayerState) {
        self.player_bank = state.bank;
    }
}
