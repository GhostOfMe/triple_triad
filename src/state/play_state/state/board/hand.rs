use ggez::graphics::Rect;

use crate::graphics::{CardEntity, sprite::Atlas as SpriteAtlas};

use std::rc::Rc;
use crate::core::{CardAtlas, Suit};
use crate::consts;

#[derive(Debug)]
pub struct Hand {
    pub cards: [Option<CardEntity>; 5],
    focus: Option<usize>,
    pub selected: Option<usize>,
    pub side: Suit,
    card_atlas: Rc<CardAtlas>,
    sprite_sheet: Rc<SpriteAtlas>,
}

impl Hand {
    pub fn empty(side: Suit, card_atlas: &Rc<CardAtlas>, sprite_sheet: &Rc<SpriteAtlas>) -> Self {
        let cards: [Option<CardEntity>; 5] = [None, None, None, None, None];
        Self {
            cards,
            focus: None,
            selected: None,
            side,
            card_atlas: Rc::clone(card_atlas),
            sprite_sheet: Rc::clone(sprite_sheet),
        }
    }

    pub fn from_ids(
        side: Suit,
        open: bool,
        ids: &[usize],
        card_atlas: &Rc<CardAtlas>,
        sprite_sheet: &Rc<SpriteAtlas>,
    ) -> Self {
        let mut cards: [Option<CardEntity>; 5] = [None, None, None, None, None];

        for (n, id) in ids.iter().enumerate() {
            let offset = match &side {
                Suit::Red => consts::LEFT_HAND_OFFSET,
                Suit::Blue => consts::RIGHT_HAND_OFFSET,
            };
            let n_small = i16::try_from(n).expect("Card index in out of i16 range n");
            let pos_x = offset[0];
            let pos_y = consts::HAND_STEP.mul_add(f32::from(n_small), offset[1]);
            let mut card_entity = CardEntity::new(
                *id,
                [pos_x, consts::WINDOW_DIMENSIONS[1] + 150.0].into(),
                side,
                side,
                !open,
                card_atlas,
                sprite_sheet,
            );
            card_entity.start_deal_animation(pos_y);
            cards[n] = Some(card_entity);
        }

        Self {
            cards,
            focus: Option::<usize>::None,
            selected: Option::<usize>::None,
            side,
            card_atlas: Rc::clone(card_atlas),
            sprite_sheet: Rc::clone(sprite_sheet),
        }
    }

    // pub fn random_from_collection(
    //     side: Suit,
    //     collection: &mut [(usize, u8, bool)],
    //     card_atlas: &Rc<CardAtlas>,
    //     sprite_sheet: &Rc<SpriteAtlas>,
    // ) -> Self {
    //     let mut cards: [Option<CardEntity>; 5] = [None, None, None, None, None];
    //     let mut ids: [usize; 5] = [0; 5];

    //     for id in ids.iter_mut() {
    //         let collection_cleared: Vec<(usize, u8, bool)> = collection
    //             .iter()
    //             .filter(|(_, n, _)| *n > 0)
    //             .copied()
    //             .collect();
    //         let new_id = collection_cleared.choose(&mut rand::thread_rng()).unwrap();
    //         *id = new_id.0;

    //         let collection_item = collection.iter_mut().find(|(i, _, _)| i == id).unwrap();
    //         collection_item.1 -= 1;
    //     }

    //     for (i, id) in ids.iter().enumerate() {
    //         let offset = match &side {
    //             Suit::Red => consts::LEFT_HAND_OFFSET,
    //             Suit::Blue => consts::RIGHT_HAND_OFFSET,
    //         };
    //         let i_small = i16::try_from(i).expect("Card index in out of i16 range n");
    //         let pos_x = offset[0];
    //         let pos_y = consts::HAND_STEP.mul_add(f32::from(i_small), offset[1]);
    //         let mut card_entity = CardEntity::new(
    //             *id,
    //             [pos_x, consts::WINDOW_DIMENSIONS[1] + 150.0].into(),
    //             side,
    //             side,
    //             false,
    //             card_atlas,
    //             sprite_sheet,
    //         );
    //         card_entity.start_deal_animation(pos_y);
    //         cards[i] = Some(card_entity);
    //         let collection_item = collection.iter_mut().find(|(i, _, _)| i == id).unwrap();
    //         collection_item.1 -= 1;
    //     }

    //     Self {
    //         cards,
    //         focus: Option::<usize>::None,
    //         selected: Option::<usize>::None,
    //         side,
    //         card_atlas: Rc::clone(card_atlas),
    //         sprite_sheet: Rc::clone(sprite_sheet),
    //     }
    // }

    pub fn add_card(&mut self, id: usize, n: u8, open: bool) {
        assert!(n < 5);
        let offset = match &self.side {
            Suit::Red => consts::LEFT_HAND_OFFSET,
            Suit::Blue => consts::RIGHT_HAND_OFFSET,
        };
        let pos_x = offset[0];
        let pos_y = consts::HAND_STEP.mul_add(f32::from(n), offset[1]);
        let mut card_entity = CardEntity::new(
            id,
            [pos_x, consts::WINDOW_DIMENSIONS[1] + 150.0].into(),
            self.side,
            self.side,
            !open,
            &self.card_atlas,
            &self.sprite_sheet,
        );
        card_entity.start_deal_animation(pos_y);
        self.cards[n as usize] = Some(card_entity);
    }

    pub fn add_card_entity(&mut self, mut card: CardEntity) {
        if let Some((n, empty)) = self
            .cards
            .iter_mut()
            .enumerate()
            .find(|(_, maybe_card)| maybe_card.is_none())
        {
            let n_small = u8::try_from(n).expect("Value is too big: {n}");
            let offset = match &self.side {
                Suit::Red => consts::LEFT_HAND_OFFSET,
                Suit::Blue => consts::RIGHT_HAND_OFFSET,
            };
            let pos_x = offset[0];
            let pos_y = consts::HAND_STEP.mul_add(f32::from(n_small), offset[1]);
            card.pos = [pos_x, pos_y].into();
            card.focused = false;
            card.adjust_focus_tween();
            card.reset_focus_tweens();
            self.selected = None;
            *empty = Some(card);

            return;
        }
        panic!("Hand is full!");
    }
    
    pub fn focus(&self) -> Option<usize>{
        self.focus
    }

    pub fn set_focus(&mut self, new: usize) {
        if self.selected.is_some() {
            return;
        }

        if let Some(id) = self.focus {
            if id != new {
                if let Some(card_1) = self.cards[id].as_mut() {
                    card_1.start_unfocus_tween();
                };
                if let Some(card_2) = self.cards[new].as_mut() {
                    card_2.start_focus_tween();
                };
            };
            self.focus = Some(new);
        } else {
            self.focus = Some(new);
            if let Some(card) = self.cards[new].as_mut() {
                card.start_focus_tween();
            }
        }
    }

    pub fn select(&mut self, card_id: usize) {
        self.cards[card_id].as_mut().unwrap().flipped = false; 
        self.selected = Some(card_id);
    }

    pub fn select_focused(&mut self) {
        if let Some(id) = self.focus {
            self.select(id);
            return;
        }
        panic!("Focus is empty")
    }

    pub fn clear_selected(&mut self) {
        if let Some(n) = self.selected {
            if let Some(card) = self.cards[n].as_mut() {
                card.focused = false;
            }
        }

        self.selected = None;
    }

    pub fn take_selected(&mut self) -> CardEntity {
        let id = self.selected.expect("No card is selected");
        self.clear_selected();
        self.cards[id].take().expect("Card id missing")
    }

    pub fn reset_foucus(&mut self) {
        if let Some(focus) = self.focus {
            if let Some(card) = self.cards[focus].as_mut() {
                card.start_unfocus_tween();
            }
        }
        self.focus = None;
    }

    pub fn card_rect(&self, n: usize) -> Rect {
        self.cards[n].as_ref().map_or(
            Rect {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            },
            |card| {
                let card_pos = card.pos;
                Rect {
                    x: card_pos.x,
                    y: card_pos.y,
                    w: consts::RIGHT_HAND_OFFSET[0] + consts::CARD_SIZE[0] - card_pos.x,
                    h: consts::CARD_SIZE[1],
                }
            },
        )
    }
}
