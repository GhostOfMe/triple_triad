use std::rc::Rc;

use ggez::graphics::{DrawParam, InstanceArray};

use crate::consts::{self, CARD_SIZE, WINDOW_DIMENSIONS};
use crate::core::Suit;
use crate::core::{CardAtlas, Element};
use crate::graphics::sprite::Atlas as SpriteAtlas;

use mint::Point2;
use tween::{Linear, Tweener};

// [top, right, bottom. left]
const POW_OFFSET: [[f32; 2]; 4] = [[14., 5.], [22., 27.], [14., 49.], [5., 27.]];

const DEAL_TWEEN_DURATION: f32 = 0.3;
const FOCUS_TWEEN_DURATION: f32 = 0.1;
const FLIP_TWEEN_DURATION: f32 = 0.1;
const MOVE_TWEEN_DURATION: f32 = 0.5;
const MOVE_TWEEN_RISE: f32 = -400.0;
const CARD_ELEM_SIZE: f32 = 50.0;
const CARD_RANK_SIZE: f32 = 24.0;

#[derive(Debug, Clone)]
struct DealAnimation {
    active: bool,
    tweener: Tweener<f32, f32, Linear>,
}

impl DealAnimation {
    pub fn init() -> Self {
        Self {
            active: false,
            tweener: Tweener::new(0.0, 0.0, 0.0, Linear),
        }
    }

    pub fn activate(&mut self, pos_y: f32) {
        self.active = true;
        self.tweener = Tweener::new(
            WINDOW_DIMENSIONS[1] + 150.0,
            pos_y,
            DEAL_TWEEN_DURATION,
            Linear,
        );
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        if !self.active {
            unreachable!()
        }

        if self.tweener.is_finished() {
            self.active = false;
        }

        self.tweener.move_by(dt)
    }
}

#[derive(Debug, Clone)]
pub struct MoveAnimation {
    pub active: bool,
    rise: Tweener<f32, f32, Linear>,
    fall: Tweener<f32, f32, Linear>,
    slide: Tweener<f32, f32, Linear>,
}

impl MoveAnimation {
    pub fn init() -> Self {
        Self {
            active: false,
            rise: Tweener::new(0.0, 0.0, 0.0, Linear),
            fall: Tweener::new(0.0, 0.0, 0.0, Linear),
            slide: Tweener::new(0.0, 0.0, 0.0, Linear),
        }
    }

    pub fn activate(&mut self, start_pos: Point2<f32>, target_pos: Point2<f32>) {
        let half_duration = MOVE_TWEEN_DURATION / 2.0;
        self.active = true;
        self.rise = Tweener::new_at(start_pos.y, MOVE_TWEEN_RISE, half_duration, Linear, 0.0);
        self.fall = Tweener::new_at(
            MOVE_TWEEN_RISE,
            target_pos.y,
            half_duration,
            Linear,
            half_duration * -1.0,
        );
        self.slide = Tweener::new_at(start_pos.x, target_pos.x, MOVE_TWEEN_DURATION, Linear, 0.0);
    }

    pub fn update(&mut self, dt: f32) -> Point2<f32> {
        assert!(self.active);

        let rise_is_valid = self.rise.is_valid();
        let fall_is_valid = self.fall.is_valid();

        let rise_y = self.rise.move_by(dt);
        let fall_y = self.fall.move_by(dt);

        let y = match (rise_is_valid, fall_is_valid) {
            (true, false) => rise_y,
            (false, true) => fall_y,
            (_, _) => unreachable!(),
        };

        let x = self.slide.move_by(dt);

        self.active = self.rise.is_valid() || self.fall.is_valid() || self.slide.is_valid();
        Point2 { x, y }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FlipStage {
    Waiting,
    First,
    Second,
    Third,
    Fourth,
    Finished,
}

#[derive(Debug, Clone)]
enum FlipDirection {
    Vertical,
    Horizintal,
    Open,
}

#[derive(Debug, Clone)]
pub struct FlipAnimation {
    pub active: bool,
    pub combo: bool,
    pub stage: FlipStage,
    direction: FlipDirection,
    grow_h: Tweener<f32, f32, Linear>,
    shrink_h: Tweener<f32, f32, Linear>,
    adjust_shrink: Tweener<f32, f32, Linear>,
    adjust_grow: Tweener<f32, f32, Linear>,
    grow_w: Tweener<f32, f32, Linear>,
    shrink_w: Tweener<f32, f32, Linear>,
}

impl FlipAnimation {
    pub fn new() -> Self {
        Self {
            active: false,
            //            finished: false,
            combo: false,
            stage: FlipStage::Waiting,
            direction: FlipDirection::Vertical,
            grow_h: Tweener::new(0.0, 1.0, FLIP_TWEEN_DURATION, Linear),
            shrink_h: Tweener::new(1.0, 0.0, FLIP_TWEEN_DURATION, Linear),
            adjust_shrink: Tweener::new(0.0, 0.0, FLIP_TWEEN_DURATION, Linear),
            adjust_grow: Tweener::new(0.0, 0.0, FLIP_TWEEN_DURATION, Linear),
            grow_w: Tweener::new(0.0, 1.0, FLIP_TWEEN_DURATION, Linear),
            shrink_w: Tweener::new(1.0, 0.0, FLIP_TWEEN_DURATION, Linear),
        }
    }

    pub fn y_adjust_tweens(
        &mut self,
        t1: Tweener<f32, f32, Linear>,
        t2: Tweener<f32, f32, Linear>,
    ) {
        self.adjust_shrink = t1;
        self.adjust_grow = t2;
    }

    pub fn update_open(&mut self, dt: f32) -> (f32, f32, bool) {
        match self.stage {
            FlipStage::First => {
                if self.shrink_w.is_finished() {
                    self.stage = FlipStage::Second;
                    self.shrink_w.move_to(0.0);
                    self.adjust_shrink.move_to(0.0);
                    return (
                        self.shrink_w.final_value(),
                        self.adjust_shrink.final_value(),
                        true,
                    );
                }
                (
                    self.shrink_w.move_by(dt),
                    self.adjust_shrink.move_by(dt),
                    false,
                )
            }
            FlipStage::Second => {
                if self.grow_w.is_finished() {
                    self.stage = FlipStage::Waiting;
                    self.grow_w.move_to(0.0);
                    self.adjust_grow.move_to(0.0);
                    self.active = false;
                    return (
                        self.grow_w.final_value(),
                        self.adjust_grow.final_value(),
                        false,
                    );
                }
                (self.grow_w.move_by(dt), self.adjust_grow.move_by(dt), false)
            }
            _ => unreachable!(),
        }
    }
    pub fn update_horizontal(&mut self, dt: f32) -> (f32, f32, bool, bool) {
        match self.stage {
            FlipStage::First => {
                if self.shrink_h.is_finished() {
                    self.stage = FlipStage::Second;
                    self.shrink_h.move_to(0.0);
                    self.adjust_shrink.move_to(0.0);
                    return (
                        self.shrink_h.final_value(),
                        self.adjust_shrink.final_value(),
                        false,
                        true,
                    );
                }
                (
                    self.shrink_h.move_by(dt),
                    self.adjust_shrink.move_by(dt),
                    false,
                    false,
                )
            }
            FlipStage::Second => {
                if self.grow_h.is_finished() {
                    self.stage = FlipStage::Third;
                    self.grow_h.move_to(0.0);
                    self.adjust_grow.move_to(0.0);
                    return (
                        self.grow_h.final_value(),
                        self.adjust_shrink.final_value(),
                        false,
                        false,
                    );
                }
                (
                    self.grow_h.move_by(dt),
                    self.adjust_grow.move_by(dt),
                    false,
                    false,
                )
            }
            FlipStage::Third => {
                if self.shrink_h.is_finished() {
                    self.stage = FlipStage::Fourth;
                    self.shrink_h.move_to(0.0);
                    self.adjust_shrink.move_to(0.0);
                    return (
                        self.shrink_h.final_value(),
                        self.adjust_shrink.final_value(),
                        false,
                        true,
                    );
                }
                (
                    self.shrink_h.move_by(dt),
                    self.adjust_shrink.move_by(dt),
                    false,
                    false,
                )
            }
            FlipStage::Fourth => {
                if self.grow_h.is_finished() {
                    self.stage = if self.combo {
                        FlipStage::Finished
                    } else {
                        FlipStage::Waiting
                    };
                    self.combo = false;
                    self.grow_h.move_to(0.0);
                    self.adjust_grow.move_to(0.0);
                    self.active = false;
                    return (
                        self.grow_h.final_value(),
                        self.adjust_grow.final_value(),
                        true,
                        false,
                    );
                }
                (
                    self.grow_h.move_by(dt),
                    self.adjust_grow.move_by(dt),
                    false,
                    false,
                )
            }
            FlipStage::Waiting | FlipStage::Finished => unreachable!(),
        }
    }

    pub fn update_vertical(&mut self, dt: f32) -> (f32, f32, bool, bool) {
        match self.stage {
            FlipStage::First => {
                if self.shrink_w.is_finished() {
                    self.stage = FlipStage::Second;
                    self.shrink_w.move_to(0.0);
                    self.adjust_shrink.move_to(0.0);
                    return (
                        self.shrink_w.final_value(),
                        self.adjust_shrink.final_value(),
                        false,
                        true,
                    );
                }
                (
                    self.shrink_w.move_by(dt),
                    self.adjust_shrink.move_by(dt),
                    false,
                    false,
                )
            }
            FlipStage::Second => {
                if self.grow_w.is_finished() {
                    self.stage = FlipStage::Third;
                    self.grow_w.move_to(0.0);
                    self.adjust_grow.move_to(0.0);
                    return (
                        self.grow_w.final_value(),
                        self.adjust_shrink.final_value(),
                        false,
                        false,
                    );
                }
                (
                    self.grow_w.move_by(dt),
                    self.adjust_grow.move_by(dt),
                    false,
                    false,
                )
            }
            FlipStage::Third => {
                if self.shrink_w.is_finished() {
                    self.stage = FlipStage::Fourth;
                    self.shrink_w.move_to(0.0);
                    self.adjust_shrink.move_to(0.0);
                    return (
                        self.shrink_w.final_value(),
                        self.adjust_shrink.final_value(),
                        false,
                        true,
                    );
                }
                (
                    self.shrink_w.move_by(dt),
                    self.adjust_shrink.move_by(dt),
                    false,
                    false,
                )
            }
            FlipStage::Fourth => {
                if self.grow_w.is_finished() {
                    self.stage = if self.combo {
                        FlipStage::Finished
                    } else {
                        FlipStage::Waiting
                    };
                    self.combo = false;
                    self.grow_w.move_to(0.0);
                    self.adjust_grow.move_to(0.0);
                    self.active = false;
                    return (
                        self.grow_w.final_value(),
                        self.adjust_grow.final_value(),
                        true,
                        false,
                    );
                }
                (
                    self.grow_w.move_by(dt),
                    self.adjust_grow.move_by(dt),
                    false,
                    false,
                )
            }
            FlipStage::Waiting | FlipStage::Finished => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ElementalEffect {
    Bonus,
    Malus,
    None,
}

#[derive(Clone, Debug)]
pub struct CardEntity {
    pub id: usize,
    pub pos: Point2<f32>,
    pub controller: Suit,
    pub owner: Suit,
    pub elemental_effect: ElementalEffect,
    scale: Point2<f32>,
    pub flipped: bool,
    card_atlas: Rc<CardAtlas>,
    sprite_sheet: Rc<SpriteAtlas>,
    focus_tween: Tweener<f32, f32, Linear>,
    unfocus_tween: Tweener<f32, f32, Linear>,
    pub focused: bool,
    pub flip_animation: FlipAnimation,
    pub move_animation: MoveAnimation,
    deal_animation: DealAnimation,
}

impl CardEntity {
    pub fn new(
        id: usize,
        pos: Point2<f32>,
        controller: Suit,
        owner: Suit,
        flipped: bool,
        card_atlas: &Rc<CardAtlas>,
        sprite_sheet: &Rc<SpriteAtlas>,
    ) -> Self {
        let focus_offset = match &controller {
            Suit::Red => -consts::RIGHT_HAND_SELECTED_OFFSET,
            Suit::Blue => consts::RIGHT_HAND_SELECTED_OFFSET,
        };
        let focus_tween_start = match &controller {
            Suit::Red => consts::LEFT_HAND_OFFSET[0],
            Suit::Blue => consts::RIGHT_HAND_OFFSET[0],
        };
        let mut focus_tween = Tweener::new(
            focus_tween_start,
            focus_tween_start + focus_offset,
            FOCUS_TWEEN_DURATION,
            Linear,
        );
        focus_tween.move_to(10.0);

        let mut unfocus_tween = Tweener::new(
            focus_tween_start + focus_offset,
            focus_tween_start,
            FOCUS_TWEEN_DURATION,
            Linear,
        );

        unfocus_tween.move_to(10.0);

        Self {
            id,
            pos,
            controller,
            owner,
            scale: [1.0, 1.0].into(),
            flipped,
            elemental_effect: ElementalEffect::None,
            card_atlas: Rc::clone(card_atlas),
            sprite_sheet: Rc::clone(sprite_sheet),
            focus_tween,
            unfocus_tween,
            focused: false,
            flip_animation: FlipAnimation::new(),
            move_animation: MoveAnimation::init(),
            deal_animation: DealAnimation::init(),
        }
    }

    pub fn mark_unchecked(&mut self) {
        self.flip_animation.active = false;
        self.flip_animation.stage = FlipStage::Finished;
    }
    pub fn mark_checked(&mut self) {
        self.flip_animation.active = false;
        self.flip_animation.stage = FlipStage::Waiting;
    }

    pub fn is_unchecked(&self) -> bool {
        self.flip_animation.stage == FlipStage::Finished
    }

    pub fn is_flipping(&self) -> bool {
        !(self.flip_animation.stage == FlipStage::Waiting
            || self.flip_animation.stage == FlipStage::Finished)
    }

    pub fn reset_focus_tweens(&mut self) {
        self.focus_tween.move_to(10.0);
        self.unfocus_tween.move_to(10.0);
    }

    pub fn add_to_instance_array(&self, array: &mut InstanceArray) {
        let rect_face = if self.flipped {
            self.sprite_sheet
                .create_sprite(consts::CARD_BACK_SPRITE_ID)
                .rect
        } else {
            self.sprite_sheet.create_sprite(self.id).rect
        };
        let card_stats = &self.card_atlas.cards[self.id];
        let rank_slice = card_stats.rank_as_slice();

        let card_pos = self.pos;

        let suit_id = self.suit_id();
        let rect_suit = self.sprite_sheet.create_sprite(suit_id).rect;
        let offset = [0.5, 0.0];
        let draw_param_face = DrawParam::new()
            .src(rect_face)
            .dest(card_pos)
            .offset(offset)
            .scale([
                consts::CARD_SCALE[0] * self.scale.x,
                consts::CARD_SCALE[1] * self.scale.y,
            ]);

        let draw_param_suit = DrawParam::new()
            .src(rect_suit)
            .dest(card_pos)
            .offset(offset)
            .scale([
                consts::CARD_SCALE[0] * self.scale.x,
                consts::CARD_SCALE[1] * self.scale.y,
            ]);

        if !self.flipped {
            array.push(draw_param_suit);
        }
        array.push(draw_param_face);

        if self.flip_animation.active || self.flipped {
            return;
        }

        for (rank, offset) in rank_slice.iter().zip(POW_OFFSET.iter()) {
            let digit_id = match rank {
                0 => consts::DIGIT_0_SPRITE_ID,
                1 => consts::DIGIT_1_SPRITE_ID,
                2 => consts::DIGIT_2_SPRITE_ID,
                3 => consts::DIGIT_3_SPRITE_ID,
                4 => consts::DIGIT_4_SPRITE_ID,
                5 => consts::DIGIT_5_SPRITE_ID,
                6 => consts::DIGIT_6_SPRITE_ID,
                7 => consts::DIGIT_7_SPRITE_ID,
                8 => consts::DIGIT_8_SPRITE_ID,
                9 => consts::DIGIT_9_SPRITE_ID,
                10 => consts::DIGIT_A_SPRITE_ID,
                _ => panic!("Wrong rank value {rank}"),
            };
            let sprite = self.sprite_sheet.create_sprite(digit_id);
            let rect_rank = sprite.rect;
            let pos = [self.pos.x + offset[0], self.pos.y + offset[1]];

            let draw_param_digit = DrawParam::new().src(rect_rank).dest(pos).scale([
                CARD_RANK_SIZE / sprite.width,
                CARD_RANK_SIZE / sprite.height,
            ]);

            array.push(draw_param_digit);

            if let Some(elem) = &card_stats.element {
                let elem_id = match elem {
                    Element::Fire => consts::ELEM_FIRE_SPRITE_ID,
                    Element::Ice => consts::ELEM_ICE_SPRITE_ID,
                    Element::Poison => consts::ELEM_POISON_SPRITE_ID,
                    Element::Wind => consts::ELEM_WIND_SPRITE_ID,
                    Element::Thunder => consts::ELEM_TRHUNDER_SPRITE_ID,
                    Element::Earth => consts::ELEM_EARTH_SPRITE_ID,
                    Element::Water => consts::ELEM_WATER_SPRITE_ID,
                    Element::Holy => consts::ELEM_HOLY_SPRITE_ID,
                };

                let rect_elem = self.sprite_sheet.create_sprite(elem_id).rect;
                let pos = [
                    self.pos.x + CARD_SIZE[0] - CARD_ELEM_SIZE - 8.0,
                    self.pos.y + 8.0,
                ];
                let draw_param_digit = DrawParam::new().src(rect_elem).dest(pos).scale([
                    CARD_ELEM_SIZE / 32.0 * self.scale.x,
                    CARD_ELEM_SIZE / 32.0 * self.scale.y,
                ]);

                array.push(draw_param_digit);
            }

            let maybe_bonus = match self.elemental_effect {
                ElementalEffect::None => None,
                ElementalEffect::Bonus => Some(consts::ELEM_BONUS_SPRITE_ID),
                ElementalEffect::Malus => Some(consts::ELEM_MALUS_SPRITE_ID),
            };

            if let Some(sprite_id) = maybe_bonus {
                let rect_bonus = self.sprite_sheet.create_sprite(sprite_id).rect;
                let pos = [
                    self.pos.x + (CARD_SIZE[0] - CARD_ELEM_SIZE) / 2.0,
                    self.pos.y + (CARD_SIZE[1] - CARD_ELEM_SIZE) / 2.0,
                ];
                let draw_param_digit = DrawParam::new().src(rect_bonus).dest(pos).scale([
                    CARD_ELEM_SIZE / 32.0 * self.scale.x,
                    CARD_ELEM_SIZE / 32.0 * self.scale.y,
                ]);

                array.push(draw_param_digit);
            }
        }
    }

    const fn suit_id(&self) -> usize {
        match (&self.controller, &self.flip_animation.stage) {
            (Suit::Red, FlipStage::Fourth) => consts::CARD_BLUE_SUIT_SPRITE_ID,
            (Suit::Blue, FlipStage::Fourth) => consts::CARD_RED_SUIT_SPRITE_ID,
            (Suit::Red, _) => consts::CARD_RED_SUIT_SPRITE_ID,
            (Suit::Blue, _) => consts::CARD_BLUE_SUIT_SPRITE_ID,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // if !matches!(self.flip_animation.direction, FlipDirection::Open){
        //     self.flipped = matches!(
        //         self.flip_animation.stage,
        //         FlipStage::Second | FlipStage::Third
        //     );
        // }

        if self.focus_tween.is_valid() {
            let value = self.focus_tween.move_by(dt);
            self.pos.x = value;
        }
        if self.unfocus_tween.is_valid() {
            let value = self.unfocus_tween.move_by(dt);
            self.pos.x = value;
        }

        if self.deal_animation.active {
            self.pos.y = self.deal_animation.update(dt);
        }

        if self.move_animation.active {
            self.pos = self.move_animation.update(dt);
        }

        if self.flip_animation.active {
            #[allow(unused_assignments)]
            let (mut flip_owner, mut flip_face) = (false, false);
            match self.flip_animation.direction {
                FlipDirection::Horizintal => {
                    (self.scale.y, self.pos.y, flip_owner, flip_face) =
                        self.flip_animation.update_horizontal(dt);
                }
                FlipDirection::Open => {
                    (self.scale.x, self.pos.x, flip_face) = self.flip_animation.update_open(dt);
                }
                FlipDirection::Vertical => {
                    (self.scale.x, self.pos.x, flip_owner, flip_face) =
                        self.flip_animation.update_vertical(dt);
                }
            }
            if flip_owner {
                self.flip_color();
            }
            if flip_face {
                self.flipped = !self.flipped;
            }
        }
    }

    pub fn start_deal_animation(&mut self, target_y: f32) {
        self.deal_animation.activate(target_y);
    }

    pub const fn deal_animation_finished(&self) -> bool {
        !self.deal_animation.active
    }

    pub fn start_move_tween(&mut self, target_pos: Point2<f32>) {
        self.move_animation.activate(self.pos, target_pos);
    }

    pub fn adjust_focus_tween(&mut self) {
        let focus_offset = match &self.controller {
            Suit::Red => -consts::RIGHT_HAND_SELECTED_OFFSET,
            Suit::Blue => consts::RIGHT_HAND_SELECTED_OFFSET,
        };
        let focus_tween_start = match &self.controller {
            Suit::Red => consts::LEFT_HAND_OFFSET[0],
            Suit::Blue => consts::RIGHT_HAND_OFFSET[0],
        };
        let mut focus_tween = Tweener::new(
            focus_tween_start,
            focus_tween_start + focus_offset,
            FOCUS_TWEEN_DURATION,
            Linear,
        );
        focus_tween.move_to(10.0);

        let mut unfocus_tween = Tweener::new(
            focus_tween_start + focus_offset,
            focus_tween_start,
            FOCUS_TWEEN_DURATION,
            Linear,
        );
        unfocus_tween.move_to(10.0);

        self.focus_tween = focus_tween;
        self.unfocus_tween = unfocus_tween;
    }

    pub fn start_focus_tween(&mut self) {
        if self.focus_tween.is_finished() && self.unfocus_tween.is_finished() && !self.focused {
            self.focused = true;
            self.focus_tween.move_to(0.0);
        }
    }

    pub fn start_unfocus_tween(&mut self) {
        if self.unfocus_tween.is_finished() {
            let start_pos = self.pos.x;
            let end_pos = match self.controller {
                Suit::Red => consts::LEFT_HAND_OFFSET[0],
                Suit::Blue => consts::RIGHT_HAND_OFFSET[0],
            };
            if self.focus_tween.is_valid() {
                let duration = FOCUS_TWEEN_DURATION - self.focus_tween.current_time;
                let new_tweener = Tweener::new(start_pos, end_pos, duration, Linear);
                self.unfocus_tween = new_tweener;
                self.focus_tween.current_time = 10.0;
            } else {
                self.unfocus_tween =
                    Tweener::new(self.pos.x, end_pos, FOCUS_TWEEN_DURATION, Linear);
            }

            self.focused = false;
            self.unfocus_tween.move_to(0.0);
        }
    }

    pub fn flip_open(&mut self) {
        self.flip_animation.direction = FlipDirection::Open;
        self.flip_animation.stage = FlipStage::First;
        self.flip_animation.active = true;
        let new_tweener_shrink = Tweener::new(
            self.pos.x,
            self.pos.x + consts::CARD_SIZE[0] / 2.0,
            FLIP_TWEEN_DURATION,
            Linear,
        );
        let new_tweener_grow = Tweener::new(
            self.pos.x + consts::CARD_SIZE[0] / 2.0,
            self.pos.x,
            FLIP_TWEEN_DURATION,
            Linear,
        );
        self.flip_animation
            .y_adjust_tweens(new_tweener_shrink, new_tweener_grow);
    }

    pub fn flip_horizontal(&mut self, combo: bool) {
        self.flip_animation.direction = FlipDirection::Horizintal;
        self.flip_animation.stage = FlipStage::First;
        self.flip_animation.active = true;
        self.flip_animation.combo = combo;
        let new_tweener_shrink = Tweener::new(
            self.pos.y,
            self.pos.y + consts::CARD_SIZE[1] / 2.0,
            FLIP_TWEEN_DURATION,
            Linear,
        );
        let new_tweener_grow = Tweener::new(
            self.pos.y + consts::CARD_SIZE[1] / 2.0,
            self.pos.y,
            FLIP_TWEEN_DURATION,
            Linear,
        );

        self.flip_animation
            .y_adjust_tweens(new_tweener_shrink, new_tweener_grow);
    }

    pub fn flip_vertical(&mut self, combo: bool) {
        self.flip_animation.direction = FlipDirection::Vertical;
        self.flip_animation.stage = FlipStage::First;
        self.flip_animation.active = true;
        self.flip_animation.combo = combo;
        let new_tweener_shrink = Tweener::new(
            self.pos.x,
            self.pos.x + consts::CARD_SIZE[0] / 2.0,
            FLIP_TWEEN_DURATION,
            Linear,
        );
        let new_tweener_grow = Tweener::new(
            self.pos.x + consts::CARD_SIZE[0] / 2.0,
            self.pos.x,
            FLIP_TWEEN_DURATION,
            Linear,
        );

        self.flip_animation
            .y_adjust_tweens(new_tweener_shrink, new_tweener_grow);
    }

    fn flip_color(&mut self) {
        self.controller = match self.controller {
            Suit::Blue => Suit::Red,
            Suit::Red => Suit::Blue,
        }
    }

    // pub fn rect(&self) -> Rect {
    //     Rect {
    //         x: self.pos.x,
    //         y: self.pos.y,
    //         w: consts::CARD_SIZE[0],
    //         h: consts::CARD_SIZE[1],
    //     }
    // }

    // pub fn rank_sum(&self) -> u8 {
    //     let stats = &self.card_atlas.cards[self.id];
    //     stats.pow_top + stats.pow_right + stats.pow_bottom + stats.pow_left
    // }
    pub fn rank_top(&self) -> u8 {
        self.card_atlas.cards[self.id].pow_top
    }
    pub fn rank_right(&self) -> u8 {
        self.card_atlas.cards[self.id].pow_right
    }
    pub fn rank_bottom(&self) -> u8 {
        self.card_atlas.cards[self.id].pow_bottom
    }
    pub fn rank_left(&self) -> u8 {
        self.card_atlas.cards[self.id].pow_left
    }
    pub fn element(&self) -> Option<Element> {
        self.card_atlas.cards[self.id].element
    }
    pub const fn elemental_effect(&self) -> i16 {
        match self.elemental_effect {
            ElementalEffect::None => 0,
            ElementalEffect::Bonus => 1,
            ElementalEffect::Malus => -1,
        }
    }
    pub fn rank_slice(&self) -> [u8; 4] {
        [
            self.rank_top(),
            self.rank_right(),
            self.rank_bottom(),
            self.rank_left(),
        ]
    }
    pub fn rank_slice_elemental(&self) -> [u8; 4] {
        [
            self.rank_top_with_elemental(),
            self.rank_right_with_elemental(),
            self.rank_bottom_with_elemental(),
            self.rank_left_with_elemental(),
        ]
    }

    pub fn rank_top_with_elemental(&self) -> u8 {
        u8::try_from(i16::from(self.rank_top()) + self.elemental_effect())
            .expect("Conversion error.")
    }
    pub fn rank_right_with_elemental(&self) -> u8 {
        u8::try_from(i16::from(self.rank_right()) + self.elemental_effect())
            .expect("Conversion error.")
    }
    pub fn rank_bottom_with_elemental(&self) -> u8 {
        u8::try_from(i16::from(self.rank_bottom()) + self.elemental_effect())
            .expect("Conversion error.")
    }
    pub fn rank_left_with_elemental(&self) -> u8 {
        u8::try_from(i16::from(self.rank_left()) + self.elemental_effect())
            .expect("Conversion error.")
    }
}
