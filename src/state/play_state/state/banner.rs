use ggez::event::MouseButton;
use ggez::graphics::{Canvas, DrawParam, InstanceArray};
use ggez::Context;

use std::rc::Rc;

use crate::consts;
use crate::core::DuelOutcome;
use crate::graphics::sprite::Atlas as SpriteAtlas;
use crate::utils::{Event, Sfx};

pub struct Banner {
    _timer: f32,
    started: bool,
    pub outcome: DuelOutcome,
    sprite_sheet: Rc<SpriteAtlas>,
}

impl Banner {
    pub fn init(&mut self) {
        self._timer = 0.0;
        self.started = false;
        self.outcome = DuelOutcome::Draw;
    }

    pub fn new(sprite_sheet: &Rc<SpriteAtlas>) -> Self {
        Self {
            _timer: 0.0,
            started: false,
            outcome: DuelOutcome::Draw,
            sprite_sheet: Rc::clone(sprite_sheet),
        }
    }

    pub fn update(&mut self, ctx: &mut Context) -> Option<Event> {
        if !self.started {
            self.started = true;
            if matches!(self.outcome, DuelOutcome::Win) {
                return Some(Event::PlaySound(Sfx::Fanfare));
            }
        }
        if ctx.mouse.button_just_pressed(MouseButton::Left) {
            return Some(Event::Finished);
        }
        None
    }

    pub fn draw(&self, canvas: &mut Canvas, array: &mut InstanceArray) {
        array.clear();
        let sprite_id = match self.outcome {
            DuelOutcome::Win => consts::FINISH_MESSAGE_WON_SPRITE_ID,
            DuelOutcome::Lose => consts::FINISH_MESSAGE_LOSE_SPRITE_ID,
            DuelOutcome::Draw => consts::FINISH_MESSAGE_DRAW_SPRITE_ID,
        };
        let sprite = self.sprite_sheet.create_sprite(sprite_id);
        let rect = sprite.rect;
        array.push(
            DrawParam::default()
                .src(rect)
                .scale([consts::SCALE_FACTOR, consts::SCALE_FACTOR])
                .dest([
                    (consts::WINDOW_DIMENSIONS[0] - sprite.width * consts::SCALE_FACTOR) / 2.0,
                    (consts::WINDOW_DIMENSIONS[1] - sprite.height * consts::SCALE_FACTOR) / 2.0,
                ]),
        );
        canvas.draw(array, [0.0, 0.0]);
    }
}
