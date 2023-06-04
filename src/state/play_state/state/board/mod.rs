use ggez::graphics::{Canvas, Color, DrawParam, Image, InstanceArray, PxScale, Rect, Text, TextFragment};
use ggez::Context;

use mint::Point2;
use rand::{thread_rng, Rng};
use std::rc::Rc;

use crate::consts;
use crate::core::{CardAtlas, DuelOutcome, Element, Rules, Suit};
use crate::utils::{self, Event, Rule as SpecialRule, Sfx};

use crate::graphics::{
    sprite::Atlas as SpriteAtlas, CardEntity, ElementEntity, ElementalEffect, TextBox,
};

//use super::super::GameSummary;

use crate::state::play_state::opponent::{AiEvent, Opponent};

mod hand;
pub use hand::Hand;

#[derive(Debug, PartialEq, Eq)]
pub enum TurnPhase {
    Pick,
    Place,
}
#[derive(Debug, Eq, PartialEq)]
enum State {
    Start,
    RedPlayerTurn(TurnPhase),
    BluePlayerTurn(TurnPhase),
    NextTurn(Suit),
    WaitingMove,
    WaitingPick,
    ComboCheck,
    Check,
    //CalculateWinner,
    Finish,
}

#[derive(Clone, Debug)]
pub struct CardFlipped {
    pub cell_id: usize,
    pub combo: Combo,
}
struct PlayingField {
    cards: [Option<CardEntity>; 9],
    elem: [Option<ElementEntity>; 9],
    hitboxes: [Rect; 9],
}

impl PlayingField {
    pub fn clear(&mut self) {
        self.cards = [None, None, None, None, None, None, None, None, None];
        self.elem = [None, None, None, None, None, None, None, None, None];
    }
}

#[derive(Clone, Debug)]
pub enum Combo {
    None,
    Plus,
    Same,
    Combo,
}

struct ComboMessage {
    active: bool,
    timer: f32,
    duration: f32,
    combo: Combo,
}

impl ComboMessage {
    pub const fn new() -> Self {
        Self {
            active: false,
            timer: 0.0,
            duration: consts::COMBO_BANNER_DURATION,
            combo: Combo::None,
        }
    }

    pub fn start(&mut self, combo: Combo) {
        self.combo = combo;
        self.timer = 0.0;
        self.active = true;
    }

    pub fn update(&mut self, dt: f32) {
        if self.timer >= self.duration {
            self.active = false;
        }
        if !self.active {
            return;
        }
        self.timer += dt;
    }

    pub fn draw(&self, array: &mut InstanceArray, sprite_sheet: &SpriteAtlas) {
        if !self.active {
            return;
        }

        let sprite_id = match self.combo {
            Combo::Plus => consts::COMBO_PLUS_BANNER_SPRITE_ID,
            Combo::Same => consts::COMBO_SAME_BANNER_SPRITE_ID,
            Combo::Combo => consts::COMBO_BANNER_SPRITE_ID,
            Combo::None => unreachable!(),
        };

        let sprite = sprite_sheet.create_sprite(sprite_id);
        let rect = sprite.rect;
        let scale_factor = 140.0 / sprite.height;

        array.push(
            DrawParam::default()
                .src(rect)
                .scale([scale_factor, scale_factor])
                .dest([
                    (consts::WINDOW_DIMENSIONS[0] - sprite.width) / 2.0,
                    (consts::WINDOW_DIMENSIONS[1] - sprite.height) / 2.0,
                ]),
        );
    }
}
struct Tooltip {
    active: bool,
    timer: f32,
    label: Option<String>,
    bg_rect: TextBox,
    card_prev: Option<usize>,
    card_curr: Option<usize>,
    card_atlas: Rc<CardAtlas>,
}
impl Tooltip {
    pub fn new(ctx: &mut Context, card_artlas: &Rc<CardAtlas>) -> Self {
        let card_atlas = Rc::clone(card_artlas);
        let bg_width = 200.0;
        let bg_height = 65.0;
        let pos: Point2<f32> = [
            (consts::WINDOW_DIMENSIONS[0] - bg_width) / 2.0,
            consts::WINDOW_DIMENSIONS[1] - bg_height - 10.0,
        ]
        .into();

        let bg_rect = TextBox::new(ctx, pos, [200.0, 65.0], (64, 64, 64));

        Self {
            active: false,
            timer: consts::TOOLTIP_DELAY,
            label: None,
            bg_rect,
            card_prev: None,
            card_curr: None,
            card_atlas,
        }
    }

    pub fn update(&mut self, dt: f32, n: Option<usize>, card_id: Option<usize>) {
        self.active = false;
        self.card_prev = self.card_curr;
        self.card_curr = n;

        self.timer = if self.card_prev == self.card_curr && self.card_curr.is_some() {
            0.0f32.max(self.timer - dt)
        } else {
            consts::TOOLTIP_DELAY
        };
        if self.timer == 0.0 {
            self.active = true;
            self.label = Some(self.card_atlas.cards[card_id.unwrap()].name.clone());
        }
    }

    pub fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) {
        if !self.active {
            return;
        }

        let label = Text::new(TextFragment {
            text: self.label.clone().unwrap(),
            color: None,
            font: Some("pixel font".into()),
            scale: Some(PxScale::from(consts::FONT_SIZE)),
        });

        let label_width = label.measure(ctx).expect("Unable to measute text").x;
        let label_pos: Point2<f32> = [
            (consts::WINDOW_DIMENSIONS[0] - label_width) / 2.0,
            consts::WINDOW_DIMENSIONS[1] - 45.0 - 10.0,
        ]
        .into();
        self.bg_rect.draw(canvas);
        
        let shadow = Text::new(TextFragment {
            text: self.label.clone().unwrap(),
            color: Some(Color::from_rgb(50, 50, 50)),
            font: Some("pixel font".into()),
            scale: Some(PxScale::from(consts::FONT_SIZE)),
        });

        canvas.draw(&shadow, [label_pos.x + 2.0, label_pos.y + 2.0]);
        canvas.draw(&label, DrawParam::default().dest(label_pos));
    }
}

pub struct Board {
    playing_field: PlayingField,
    pub red_hand: Hand,
    pub blue_hand: Hand,
    pub rules: Rules,
    pub opponent: Opponent,
    combo_message: ComboMessage,
    tooltip: Tooltip,
    state_stack: Vec<State>,
    pub card_atlas: Rc<CardAtlas>,
    pub sprite_sheet: Rc<SpriteAtlas>,
    bg_image: Rc<Image>,
}

impl Board {
    pub fn empty(
        ctx: &mut Context,
        card_atlas: &Rc<CardAtlas>,
        sprite_sheet: &Rc<SpriteAtlas>,
        bg_image: &Rc<Image>,
    ) -> Self {
        #[rustfmt::skip]
        let cards = [
            None, None, None,
            None, None, None,
            None, None, None
        ];

        #[rustfmt::skip]
        let elem = [
            None, None, None,
            None, None, None,
            None, None, None
        ];
        let board_offset = [
            (consts::WINDOW_DIMENSIONS[0] - consts::CARD_SIZE[0] * 3.0) / 2.0,
            (consts::WINDOW_DIMENSIONS[1] - consts::CARD_SIZE[1] * 3.0) / 2.0,
        ];
        let hitboxes = core::array::from_fn(|i| {
            let i_small = i16::try_from(i).expect("Value is too big! {i}");
            let x = f32::from(i_small % 3).mul_add(consts::CARD_SIZE[0], board_offset[0]);
            let y = f32::from(i_small / 3).mul_add(consts::CARD_SIZE[1], board_offset[1]);
            let h = consts::CARD_SIZE[1];
            let w = consts::CARD_SIZE[0];
            Rect { x, y, w, h }
        });
        let state_stack = vec![
            State::Finish,
            State::RedPlayerTurn(TurnPhase::Pick),
            State::Start,
        ];

        let playing_field = PlayingField {
            cards,
            elem,
            hitboxes,
        };
        Self {
            playing_field,
            red_hand: Hand::empty(Suit::Red, card_atlas, sprite_sheet),
            blue_hand: Hand::empty(Suit::Blue, card_atlas, sprite_sheet),
            rules: Rules::default(),
            opponent: Opponent::new(),
            state_stack,
            combo_message: ComboMessage::new(),
            tooltip: Tooltip::new(ctx, card_atlas),
            card_atlas: Rc::clone(card_atlas),
            sprite_sheet: Rc::clone(sprite_sheet),
            bg_image: Rc::clone(bg_image),
        }
    }

    pub fn init(&mut self) {
        self.playing_field.clear();

        self.rules = Rules::default();

        self.opponent.clear();
        self.state_stack = vec![
            State::Finish,
            State::RedPlayerTurn(TurnPhase::Pick),
            State::Start,
        ];
        self.red_hand = Hand::empty(Suit::Red, &self.card_atlas, &self.sprite_sheet);
        self.blue_hand = Hand::empty(Suit::Blue, &self.card_atlas, &self.sprite_sheet);
    }
    pub fn first_turn(&mut self, p: Suit) {
        self.state_stack.clear();
        self.state_stack.push(State::Finish);
        match p {
            Suit::Red => self.state_stack.push(State::RedPlayerTurn(TurnPhase::Pick)),
            Suit::Blue => self
                .state_stack
                .push(State::BluePlayerTurn(TurnPhase::Pick)),
        }
        self.state_stack.push(State::Start);
    }
    pub fn clear(&mut self) {
        self.playing_field.clear();
        if self.rules.elemental {
            self.populate_elem();
        }

        self.state_stack = vec![
            State::Finish,
            State::RedPlayerTurn(TurnPhase::Pick),
            State::Start,
        ];

        self.blue_hand = Hand::empty(Suit::Blue, &self.card_atlas, &self.sprite_sheet);
        self.red_hand = Hand::empty(Suit::Blue, &self.card_atlas, &self.sprite_sheet);
    }

    pub fn deal_sunnden_death(&mut self) {
        for maybe_card in &mut self.playing_field.cards {
            match maybe_card.as_ref().unwrap().controller {
                Suit::Red => self.red_hand.add_card_entity(maybe_card.take().unwrap()),
                Suit::Blue => self.blue_hand.add_card_entity(maybe_card.take().unwrap()),
            }
        }

        self.playing_field.clear();

        if self.rules.elemental {
            self.populate_elem();
        }

        self.red_hand.reset_foucus();
        self.blue_hand.reset_foucus();
        self.red_hand.clear_selected();
        self.blue_hand.clear_selected();

        self.state_stack.push(State::Start);
        self.state_stack.push(State::RedPlayerTurn(TurnPhase::Pick));
    }

    fn open_opponent_cards(&mut self) {
        for card in self.red_hand.cards.iter_mut().flatten() {
            card.flip_open();
        }
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        array: &mut InstanceArray,
        elem_array: &mut InstanceArray,
    ) {
        if self.state_stack.is_empty() {
            return;
        }
        let bg_scale = [
            consts::WINDOW_DIMENSIONS[0]
                / f32::from(u16::try_from(self.bg_image.width()).expect("Value ins too big!")),
            consts::WINDOW_DIMENSIONS[1]
                / f32::from(u16::try_from(self.bg_image.height()).expect("Value ins too big!")),
        ];
        canvas.draw(
            Rc::as_ref(&self.bg_image),
            DrawParam::default().scale(bg_scale),
        );
        elem_array.clear();
        array.clear();
        self.populate_instance_array(array);
        self.draw_hands(array);
        self.combo_message.draw(array, &self.sprite_sheet);

        for e in self
            .playing_field
            .elem
            .iter()
            .flatten()
            .filter(|e| e.active)
        {
            e.draw(elem_array);
        }
        // Score is dwawn only after game is started.
        if !matches!(
            self.state_stack.last().unwrap(),
            State::Start | State::WaitingPick
        ) {
            self.draw_score(array);
        }
        canvas.draw(array, [0.0, 0.0]);
        canvas.draw(elem_array, [0.0, 0.0]);

        self.tooltip.draw(ctx, canvas);
        // self.draw_state_stack(canvas);
        // self.draw_ai_state_stack(canvas);
    }

    fn populate_instance_array(&self, array: &mut InstanceArray) {
        for card in self.playing_field.cards.iter().flatten() {
            card.add_to_instance_array(array);
        }
    }

    fn draw_hands(&self, array: &mut InstanceArray) {
        for card in self.blue_hand.cards.iter().flatten() {
            card.add_to_instance_array(array);
        }
        for card in self.red_hand.cards.iter().flatten() {
            card.add_to_instance_array(array);
        }
    }

    // fn draw_rects(&self, ctx: &mut Context, canvas: &mut Canvas) {
    //     let blue_rect = Image::from_solid(ctx, 64, Color::from_rgba(255, 0, 255, 20));

    //     for (_, hitbox_rect) in self.empty_cells_iter() {
    //         canvas.draw(
    //             &blue_rect,
    //             DrawParam::default()
    //                 .scale(CARD_SCALE)
    //                 .dest([hitbox_rect.x, hitbox_rect.y]),
    //         );
    //     }
    // }

    // fn draw_state_stack(&self, canvas: &mut Canvas) {
    //     let text_raw = self
    //         .state_stack
    //         .iter()
    //         .rev()
    //         .map(|state| format!("{state:?}\n"))
    //         .collect::<String>();

    //     let text = Text::new(text_raw);
    //     let text_pos = [300., 520.];
    //     canvas.draw(&text, DrawParam::default().dest(text_pos));
    // }

    // fn draw_ai_state_stack(&self, canvas: &mut Canvas) {
    //     let text_raw = self
    //         .opponent
    //         .ai
    //         .actions
    //         .iter()
    //         .rev()
    //         .map(|state| format!("{state:?}\n"))
    //         .collect::<String>();

    //     let text = Text::new(text_raw);
    //     let text_pos = [100., 520.];
    //     canvas.draw(&text, DrawParam::default().dest(text_pos));
    // }

    fn calculate_score(&self) -> (usize, usize) {
        self.playing_field
            .cards
            .iter()
            .chain(&self.red_hand.cards)
            .chain(&self.blue_hand.cards)
            .flatten()
            .fold((0, 0), |(red, blue), card| match card.controller {
                Suit::Red => (red + 1, blue),
                Suit::Blue => (red, blue + 1),
            })
    }

    fn draw_score(&self, array: &mut InstanceArray) {
        let (red_score, blue_score) = self.calculate_score();

        for (score, pos) in [red_score, blue_score]
            .iter()
            .zip(&[consts::RED_SCORE_POS, consts::BLUE_SCORE_POS])
        {
            let digit_id = match score {
                0 => consts::BIG_DIGIT_0_SPRITE_ID,
                1 => consts::BIG_DIGIT_1_SPRITE_ID,
                2 => consts::BIG_DIGIT_2_SPRITE_ID,
                3 => consts::BIG_DIGIT_3_SPRITE_ID,
                4 => consts::BIG_DIGIT_4_SPRITE_ID,
                5 => consts::BIG_DIGIT_5_SPRITE_ID,
                6 => consts::BIG_DIGIT_6_SPRITE_ID,
                7 => consts::BIG_DIGIT_7_SPRITE_ID,
                8 => consts::BIG_DIGIT_8_SPRITE_ID,
                9 => consts::BIG_DIGIT_9_SPRITE_ID,
                _ => panic!("Wrong score value {score}"),
            };
            let score_rect = self.sprite_sheet.create_sprite(digit_id).rect;

            array.push(
                DrawParam::default()
                    .src(score_rect)
                    .dest(*pos)
                    .scale([2.0, 2.0]),
            );
        }
    }

    pub fn turn_marker_status(&self) -> [bool; 2] {
        [
            matches!(self.state_stack.last().unwrap(), State::RedPlayerTurn(_)),
            matches!(self.state_stack.last().unwrap(), State::BluePlayerTurn(_)),
        ]
    }

    fn calculate_result(&mut self) -> (usize, usize) {
        let (red_score, blue_score) = self
            .playing_field
            .cards
            .iter()
            .chain(&self.red_hand.cards)
            .chain(&self.blue_hand.cards)
            .flatten()
            .fold((0, 0), |(red, blue), card| match card.controller {
                Suit::Red => (red + 1, blue),
                Suit::Blue => (red, blue + 1),
            });

        (red_score, blue_score)
    }

    fn move_animation_finished(&self) -> bool {
        self.playing_field
            .cards
            .iter()
            .flatten()
            .all(|card| !card.move_animation.active)
    }

    fn deal_animation_finished(&self) -> bool {
        self.blue_hand
            .cards
            .iter()
            .flatten()
            .all(CardEntity::deal_animation_finished)
    }

    fn flip_animation_finished(&self) -> bool {
        self.playing_field
            .cards
            .iter()
            .chain(self.red_hand.cards.iter())
            .chain(self.blue_hand.cards.iter())
            .all(|maybe_card| {
                if let Some(card) = maybe_card {
                    if card.is_flipping() {
                        return false;
                    }
                }
                true
            })
    }

    // fn mark_checked_all(&mut self) {
    //     for card in self.cards.iter_mut().flatten() {
    //         card.mark_checked();
    //     }
    // }

    fn cards_to_check(&mut self) -> Vec<usize> {
        self.playing_field
            .cards
            .iter_mut()
            .enumerate()
            .filter_map(|(i, maybe_card)| {
                if let Some(card) = maybe_card {
                    if card.is_unchecked() {
                        return Some(i);
                    }
                }
                None
            })
            .collect()
    }

    pub fn check_cards(&mut self) -> Vec<CardFlipped> {
        let mut cards_flipped: Vec<CardFlipped> = Vec::new();
        for id in &self.cards_to_check() {
            cards_flipped.extend(self.check_card(*id).iter().flatten().cloned());
        }
        cards_flipped.dedup_by(|a, b| a.cell_id == b.cell_id);
        cards_flipped
    }

    pub fn check_card(&mut self, cell_id: usize) -> [Option<CardFlipped>; 4] {
        let mut cards_flipped = [None, None, None, None];
        let card = self.playing_field.cards[cell_id]
            .as_ref()
            .expect("Wrong card id: {cell_id}!");

        // Combo check
        //

        let card_suit = card.controller;

        let border_mask = utils::border_mask(cell_id);
        let card_ranks = card.rank_slice();
        let card_ranks_elemental = card.rank_slice_elemental();

        let ranks_other: [Option<u8>; 4] = core::array::from_fn(|i| {
            if let Some(idx) = border_mask[i].as_ref() {
                if let Some(card) = self.playing_field.cards[*idx].as_ref() {
                    if card.controller == card_suit {
                        return None;
                    }
                    return match i {
                        0 => Some(card.rank_bottom()),
                        1 => Some(card.rank_left()),
                        2 => Some(card.rank_top()),
                        3 => Some(card.rank_right()),
                        _ => unreachable!(),
                    };
                }
            }
            None
        });

        let ranks_other_wall: [Option<u8>; 4] = core::array::from_fn(|i| {
            if let Some(idx) = border_mask[i].as_ref() {
                if let Some(card) = self.playing_field.cards[*idx].as_ref() {
                    if card.controller == card_suit {
                        return None;
                    }
                    return match i {
                        0 => Some(card.rank_bottom()),
                        1 => Some(card.rank_left()),
                        2 => Some(card.rank_top()),
                        3 => Some(card.rank_right()),
                        _ => unreachable!(),
                    };
                } else {
                    return None;
                }
            }
            Some(10)
        });

        let ranks_other_elemental: [Option<u8>; 4] = core::array::from_fn(|i| {
            if let Some(idx) = border_mask[i].as_ref() {
                if let Some(card) = self.playing_field.cards[*idx].as_ref() {
                    if card.controller == card_suit {
                        return None;
                    }
                    return match i {
                        0 => Some(card.rank_bottom_with_elemental()),
                        1 => Some(card.rank_left_with_elemental()),
                        2 => Some(card.rank_top_with_elemental()),
                        3 => Some(card.rank_right_with_elemental()),
                        _ => unreachable!(),
                    };
                }
            }
            None
        });

        // Same Rule

        let cards_flipped_same = if self.rules.same {
            if self.rules.same_wall {
                let wall_tmp = utils::check_same(card_ranks, ranks_other_wall);
                core::array::from_fn(|i| wall_tmp[i] && border_mask[i].is_some())
            } else {
                utils::check_same(card_ranks, ranks_other)
            }
        } else {
            [false; 4]
        };

        // Plus check
        //

        let cards_flipped_plus = if self.rules.plus {
            utils::check_plus(card_ranks, ranks_other)
        } else {
            [false; 4]
        };

        //Regular Checks

        let cards_flipped_normal = utils::check_normal(card_ranks_elemental, ranks_other_elemental);

        for (i, (normal, (same, plus))) in cards_flipped_normal
            .iter()
            .zip(cards_flipped_same.iter().zip(cards_flipped_plus))
            .enumerate()
            .filter(|(_, (n, (s, p)))| **n || **s || *p)
        {
            let vertical = i % 2 != 0;
            let combo = *same || plus;
            let card = self.playing_field.cards[border_mask[i].unwrap()]
                .as_mut()
                .unwrap();

            if vertical {
                card.flip_vertical(combo);
            } else {
                card.flip_horizontal(combo);
            }

            cards_flipped[i] = Some(CardFlipped {
                cell_id: border_mask[i].unwrap(),
                combo: Combo::None,
            });

            if *same {
                cards_flipped[i].as_mut().unwrap().combo = Combo::Same;
                continue;
            }
            if plus {
                cards_flipped[i].as_mut().unwrap().combo = Combo::Plus;
                continue;
            }
            if *normal {
                cards_flipped[i] = Some(CardFlipped {
                    cell_id: border_mask[i].unwrap(),
                    combo: Combo::None,
                });
            }
        }

        let card = self.playing_field.cards[cell_id]
            .as_mut()
            .expect("Wrong card id: {cell_id}!");
        card.mark_checked();

        cards_flipped
    }

    fn empty_cells_iter(&self) -> impl Iterator<Item = (usize, &Rect)> {
        self.playing_field
            .hitboxes
            .iter()
            .enumerate()
            .filter(|(i, _)| self.playing_field.cards[*i].is_none())
    }

    pub fn put_card(&mut self, cell_id: usize, mut card: CardEntity) {
        let new_pos = self.playing_field.hitboxes[cell_id].point();
        card.reset_focus_tweens();
        card.mark_unchecked();
        card.start_move_tween(new_pos);

        let card_elem = self.card_atlas.cards[card.id].element;
        let cell_elem = self.playing_field.elem[cell_id].as_ref().map(|e| e.element);
        match (card_elem, cell_elem) {
            (_, None) => card.elemental_effect = ElementalEffect::None,
            (Some(card_elem), Some(cell_elem)) if card_elem == cell_elem => {
                card.elemental_effect = ElementalEffect::Bonus;
            }
            (Some(card_elem), Some(cell_elem)) if card_elem != cell_elem => {
                card.elemental_effect = ElementalEffect::Malus;
            }
            (None, Some(_)) => {
                card.elemental_effect = ElementalEffect::Malus;
            }
            (_, _) => unreachable!(),
        }
        self.playing_field.cards[cell_id] = Some(card);
    }

    // pub fn card_to_array(&self) -> [&CardEntity; 10] {
    //     let v: Vec<&CardEntity> = self
    //         .blue_hand
    //         .cards
    //         .iter()
    //         .chain(self.red_hand.cards.iter())
    //         .chain(self.playing_field.cards.iter())
    //         .flatten()
    //         .collect();
    //     v.try_into().expect("unable to convert")
    // }

    // pub fn trade_info(&mut self) -> TradeInfo {
    //     let cards = self.card_to_array();
    //     let red: [TradeItem; 5] = cards
    //         .iter()
    //         .filter(|c| c.owner == Suit::Red)
    //         .map(|c| TradeItem {
    //             id: c.id,
    //             controller: c.controller,
    //         })
    //         .collect::<Vec<TradeItem>>()
    //         .try_into()
    //         .expect("Unable to convert!");

    //     let blue: [TradeItem; 5] = cards
    //         .iter()
    //         .filter(|c| c.owner == Suit::Blue)
    //         .map(|c| TradeItem {
    //             id: c.id,
    //             controller: c.controller,
    //         })
    //         .collect::<Vec<TradeItem>>()
    //         .try_into()
    //         .expect("Unable to convert!");

    //     let (red_score, blue_score) = self.calculate_score();

    //     let winner = self.outcome();

    //     let diff = (red_score.max(blue_score) - red_score.min(blue_score)) / 2;

    //     TradeInfo {
    //         red,
    //         blue,
    //         winner,
    //         rule: self.trade_rule,
    //         diff,
    //     }
    // }

    // pub const fn outcome(&self) -> DuelOutcome {
    //     self.final_message.outcome
    // }

    pub fn update(&mut self, ctx: &mut Context) -> Option<Event> {
        // if self.state_stack.is_empty() {
        //     return Some(Event::Quit);
        // }

        let mouse_pos = ctx.mouse.position();
        let dt = ctx.time.delta().as_secs_f32();

        let is_left_pressed = ctx
            .mouse
            .button_just_pressed(ggez::event::MouseButton::Left);

        let is_right_pressed = ctx
            .mouse
            .button_just_pressed(ggez::event::MouseButton::Right);

        for card in self.blue_hand.cards.iter_mut().flatten() {
            card.update(dt);
        }

        for card in self.red_hand.cards.iter_mut().flatten() {
            card.update(dt);
        }

        for card in self.playing_field.cards.iter_mut().flatten() {
            card.update(dt);
        }

        for e in self.playing_field.elem.iter_mut().flatten() {
            e.update(dt);
        }
        let mut card_hover = None;
        let mut card_id_hover = None;

        for (i, c) in self
            .playing_field
            .cards
            .iter()
            .chain(self.red_hand.cards.iter())
            .chain(self.blue_hand.cards.iter())
            .enumerate()
        {
            let card_rect = c.as_ref().map_or(
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
                        w: consts::CARD_SIZE[0],
                        h: consts::CARD_SIZE[1],
                    }
                },
            );

            if card_rect.contains(mouse_pos) {
                card_hover = Some(i);
                card_id_hover = Some(c.as_ref().unwrap().id);
            }
        }

        self.tooltip.update(dt, card_hover, card_id_hover);

        self.combo_message.update(dt);

        match self.state_stack.last().expect("State stack is empty!") {
            State::WaitingPick => {}
            State::Start => self.start(),
            State::BluePlayerTurn(phase) => match phase {
                TurnPhase::Pick => {
                    // if self.empty_cells_iter().next().is_none() {
                    //     self.state_stack.pop();
                    //     self.state_stack.push(State::Check);
                    // }

                    let mut no_focus = true;
                    let mut play_sound = false; 
                    for i in (0..5).rev() {
                        let card_hitbox = self.blue_hand.card_rect(i);
                        if card_hitbox.contains(mouse_pos) { 
                            if self.blue_hand.focus() != Some(i){
                                play_sound = true;
                            } 
                            self.blue_hand.set_focus(i);
                            no_focus = false;
                            if is_left_pressed {
                                self.blue_hand.select(i);
                                self.state_stack.pop();
                                self.state_stack
                                    .push(State::BluePlayerTurn(TurnPhase::Place));
                                return Some(Event::PlaySound(Sfx::Move));
                            }
                            break;
                        };
                    }
                    if no_focus {
                        self.blue_hand.reset_foucus();
                    }
                    if play_sound{
                        return Some(Event::PlaySound(Sfx::Move));
                    }
                    
                }
                TurnPhase::Place => {
                    if !(is_left_pressed || is_right_pressed) {
                        return None;
                    }
                    if is_right_pressed {
                        self.blue_hand.clear_selected();
                        self.state_stack.pop();
                        self.state_stack
                            .push(State::BluePlayerTurn(TurnPhase::Pick));
                    }

                    let mut playing_field_rect = self.playing_field.hitboxes[0];
                    playing_field_rect.h = consts::CARD_SIZE[1] * 3.0;
                    playing_field_rect.w = consts::CARD_SIZE[0] * 3.0;

                    if !playing_field_rect.contains(mouse_pos) {
                        self.blue_hand.clear_selected();
                        self.state_stack.pop();
                        self.state_stack
                            .push(State::BluePlayerTurn(TurnPhase::Pick));
                    }

                    for i in 0..9 {
                        if !self.playing_field.hitboxes[i].contains(mouse_pos) {
                            continue;
                        }
                        if self.playing_field.cards[i].is_some() {
                            continue;
                        }
                        let selected_card_entity = self.blue_hand.take_selected();
                        self.put_card(i, selected_card_entity);
                        self.state_stack.pop();
                        self.state_stack.push(State::NextTurn(Suit::Red));
                        self.state_stack.push(State::Check);
                        self.state_stack.push(State::WaitingMove);

                        return Some(Event::PlaySound(Sfx::Move));
                    }
                }
            },
            State::RedPlayerTurn(_) => {
                // if self.empty_cells_iter().next().is_none() {
                //     self.state_stack.pop();
                //     //                    self.state_stack.push(State::Check);
                // }

                if let Some(e) = self.opponent.think(
                    dt,
                    &mut self.red_hand,
                    &mut self.blue_hand,
                    &self.playing_field.cards,
                    &self.playing_field.elem,
                    &self.rules,
                ) {
                    match e {
                        AiEvent::Put(to) => {
                            let selected_card_entity = self.red_hand.take_selected();
                            self.put_card(to, selected_card_entity);
                            self.state_stack.pop();
                            self.state_stack.push(State::NextTurn(Suit::Blue));
                            self.state_stack.push(State::Check);
                            self.state_stack.push(State::WaitingMove);

                            return Some(Event::PlaySound(Sfx::Move));
                        }
                        AiEvent::Focus => {
                            return Some(Event::PlaySound(Sfx::Move));
                        }
                    }
                }
            }
            State::NextTurn(player) => match player {
                Suit::Blue => {
                    self.state_stack.pop();
                    self.state_stack
                        .push(State::BluePlayerTurn(TurnPhase::Pick));
                }
                Suit::Red => {
                    self.state_stack.pop();
                    self.state_stack.push(State::RedPlayerTurn(TurnPhase::Pick));
                }
            },
            State::Check => {
                // if self.cards_to_check().is_empty() {
                //     return None;
                // }
                let flipped_cards = self.check_cards();
                let combo_same = flipped_cards.iter().any(|c| matches!(c.combo, Combo::Same));
                let combo_plus = flipped_cards.iter().any(|c| matches!(c.combo, Combo::Plus));
                if combo_same {
                    self.combo_message.start(Combo::Same);
                } else if combo_plus {
                    self.combo_message.start(Combo::Plus);
                }
                if combo_plus || combo_same {
                    self.state_stack.push(State::ComboCheck);
                }

                if !flipped_cards.is_empty() {
                    return Some(Event::PlaySound(Sfx::Flip));
                }
                
                if !self.flip_animation_finished() {
                    return None;
                }

                self.state_stack.pop();

                if self.empty_cells_iter().next().is_none() {
                    self.state_stack.pop();
                }

            }
            State::ComboCheck => {
                if !self.flip_animation_finished() {
                    return None;
                }

                self.state_stack.pop();
                if !self.cards_to_check().is_empty() {
                    let flipped_cards = self.check_cards();
                    if !flipped_cards.is_empty() {
                        self.combo_message.start(Combo::Combo);
                        return Some(Event::PlaySound(Sfx::Flip));
                    }
                }
            }

            State::WaitingMove => {
                if self.move_animation_finished() {
                    self.deactivate_occupied_elem();
                    self.state_stack.pop();
                }
            }
            State::Finish => {
                let outcome = match self.calculate_result() {
                    (r, b) if r > b => DuelOutcome::Lose,
                    (r, b) if r < b => DuelOutcome::Win,
                    (r, b) if r == b => DuelOutcome::Draw,
                    (_, _) => unreachable!(),
                };
                self.tooltip.active = false;
                return Some(Event::GameSummary(outcome, self.rules.sudden_death));
            }
        };

        None
    }
    fn start(&mut self) {
        if self.deal_animation_finished() {
            self.activate_elem();
            self.state_stack.pop();
            if self.rules.open {
                self.open_opponent_cards();
                self.state_stack.push(State::Check);
            }
        }
    }

    fn deactivate_occupied_elem(&mut self) {
        for (maybe_elem, maybe_card) in self
            .playing_field
            .elem
            .iter_mut()
            .zip(self.playing_field.cards.iter())
        {
            if let (Some(e), Some(_)) = (maybe_elem, maybe_card) {
                e.active = false;
            }
        }
    }

    fn activate_elem(&mut self) {
        for e in self.playing_field.elem.iter_mut().flatten() {
            e.active = true;
        }
    }

    // pub fn clear_stack(&mut self) {
    //     self.state_stack.clear()
    // }

    pub fn wait_for_pick(&mut self) {
        self.state_stack.push(State::WaitingPick);
    }
    pub fn next_state(&mut self) {
        self.state_stack.pop();
    }

    pub fn toggle_rule(&mut self, r: SpecialRule) {
        match r {
            SpecialRule::Open => self.rules.open = !self.rules.open,
            SpecialRule::Elemental => self.rules.elemental = !self.rules.elemental,
            SpecialRule::Random => self.rules.random = !self.rules.random,
            SpecialRule::Same => self.rules.same = !self.rules.same,
            SpecialRule::Wall => self.rules.same_wall = !self.rules.same_wall,
            SpecialRule::Plus => self.rules.plus = !self.rules.plus,
            SpecialRule::SuddenDeath => self.rules.sudden_death = !self.rules.sudden_death,
        }
    }

    // fn update_menu(&mut self, ctx: &mut Context) {
    //     if let Some(e) = self.menu.update(&self.rules, ctx) {
    //         match e {
    //             MenuEvent::Play => {
    //                 self.clear();
    //                 self.red_hand = Hand::from_ids(
    //                     Suit::Red,
    //                     &self.opponent.hand(),
    //                     &self.card_atlas,
    //                     &self.sprite_sheet,
    //                 );
    //                 self.ai.diffuculty(self.opponent.difficulty);
    //                 if self.rules.random {
    //                     self.blue_hand = Hand::random_from_collection(
    //                         Suit::Blue,
    //                         &mut self.collection,
    //                         &self.card_atlas,
    //                         &self.sprite_sheet,
    //                     );
    //                     self.state_stack.pop();
    //                     self.state_stack.pop();
    //                     self.state_stack.push(DuelState::Start);
    //                     return;
    //                 }

    //                 self.state_stack.pop();
    //             }
    //             MenuEvent::Quit => {
    //                 self.state_stack.clear();
    //             }
    //             MenuEvent::ChangeRule(r) => {
    //                 self.toggle_rule(r);
    //             }
    //             _ => {
    //                 unreachable!();
    //             }
    //         };
    //     }
    // }

    // fn update_card_select(&mut self, ctx: &mut Context) {
    //     let collection_cleared = self
    //         .collection
    //         .iter()
    //         .filter(|(_, n, _)| *n > 0)
    //         .copied()
    //         .collect();

    //     if let Some((id, count)) =
    //         self.card_selector
    //             .update(ctx, &collection_cleared, &self.card_atlas)
    //     {
    //         self.blue_hand.add_card(id, count, true);

    //         let n = self
    //             .collection
    //             .iter()
    //             .position(|(i, _, _)| *i == id)
    //             .expect("No such card: {id}");
    //         self.collection[n].1 -= 1;

    //         let collection_cleared: Vec<(usize, u8, bool)> = self
    //             .collection
    //             .iter()
    //             .copied()
    //             .filter(|(_, n, _)| *n > 0)
    //             .collect();

    //         self.card_selector
    //             .card_menu
    //             .init(&collection_cleared, &self.card_atlas);

    //         if 4 == count {
    //             self.state_stack.pop();
    //             self.state_stack.push(DuelState::Start);
    //         }
    //     }
    // }

    fn populate_elem(&mut self) {
        let first_elem_n = thread_rng().gen_range(0..9);
        for i in 0..9 {
            if !(thread_rng().gen::<f32>() < consts::ELEM_PROB || i == first_elem_n) {
                continue;
            }
            let element: Element = rand::random();
            let pos = self.playing_field.hitboxes[i].point();
            self.playing_field.elem[i] = Some(ElementEntity::new(element, pos));
        }
    }
}
