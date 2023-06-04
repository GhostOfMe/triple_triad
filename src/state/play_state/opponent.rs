use rand::seq::SliceRandom;

use crate::core::Rules;
use crate::graphics::{CardEntity, ElementEntity};
use crate::state::play_state::ai::Opponent as Ai;
use crate::state::play_state::Hand;

pub enum AiEvent {
    Focus,
    Put(usize),
}

pub struct Opponent {
    card_enabled: [bool; 10],
    ai: Ai,
}

impl Opponent {
    pub fn new() -> Self {
        Self {
            card_enabled: [true; 10],
            ai: Ai::new(),
        }
    }
    pub fn toogle_cards(&mut self, n: usize) {
        if self.cards().iter().filter(|a| **a).count() <= 1 && self.card_enabled[n] {
            return;
        }

        self.card_enabled[n] = !self.card_enabled[n];
    }
    pub fn difficulty(&self) -> usize {
        self.ai.diffuculty
    }
    pub fn cards(&self) -> [bool; 10] {
        self.card_enabled
    }

    pub fn set_difficulty(&mut self, value: usize) {
        self.ai.diffuculty(value)
    }

    pub fn clear(&mut self) {
        self.ai = Ai::new();
        self.card_enabled = [true; 10];
    }

    pub fn think(
        &mut self,
        dt: f32,
        red_hand: &mut Hand,
        blue_hand: &mut Hand,
        field: &[Option<CardEntity>; 9],
        elem: &[Option<ElementEntity>; 9],
        rules: &Rules,
    ) -> Option<AiEvent> {
        self.ai.think(dt, red_hand, blue_hand, field, elem, rules)
    }

    pub fn new_hand(&self) -> [usize; 5] {
        let mut cards = Vec::with_capacity(110);

        for (i, lvl) in self.card_enabled.iter().enumerate() {
            if *lvl {
                let lower_bound = i * 11;
                let upper_bound = (i + 1) * 11;
                for j in lower_bound..upper_bound {
                    cards.push(j);
                }
            }
        }
        let ids: [usize; 5] =
            std::array::from_fn(|_| *cards.choose(&mut rand::thread_rng()).unwrap());
        ids
    }
}
