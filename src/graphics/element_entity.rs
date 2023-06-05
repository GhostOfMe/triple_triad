use ggez::graphics::{InstanceArray, Rect, Color, DrawParam};

use mint::Point2;

use crate::core::Element;
use crate::consts::CARD_SIZE;

// sprites
const THUNDER: f32 = 0.0;
const WIND: f32 = 1.0;
const POISON: f32 = 2.0;
const WATER: f32 = 3.0;
const FIRE: f32 = 4.0;
const HOLY: f32 = 5.0;
const EARTH: f32 = 6.0;
const ICE: f32 = 7.0;

const ELEM_SIZE: f32 = 48.0;

#[derive(Debug, Clone)]
pub struct ElementEntity {
    pub element: Element,
    pub pos: Point2<f32>,
    pub active: bool,
    frames: f32,
    current: f32,
    timer: f32,
    frame_duration: f32,
}
impl ElementEntity {
    pub const fn new(element: Element, pos: Point2<f32>) -> Self {
        let frames = match element {
            Element::Fire => 4.0,
            Element::Ice => 2.0,
            Element::Poison => 3.0,
            Element::Wind => 2.0,
            Element::Thunder => 4.0,
            Element::Earth => 2.0,
            Element::Water => 4.0,
            Element::Holy => 2.0,
        };

        Self {
            element,
            pos,
            active: false,
            frames,
            current: 0.0,
            timer: 0.0,
            frame_duration: 0.1,
        }
    }

    fn next_frame(&mut self) {
        if self.current == self.frames - 1.0 {
            self.current = 0.0;
        } else {
            self.current += 1.0;
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.active {
            return;
        }

        self.timer -= dt;

        if self.timer <= 0.0 {
            self.next_frame();
            self.timer = self.frame_duration;
        }
    }

    pub fn draw(&self, array: &mut InstanceArray) {
        let y = match self.element {
            Element::Fire => FIRE,
            Element::Ice => ICE,
            Element::Poison => POISON,
            Element::Wind => WIND,
            Element::Thunder => THUNDER,
            Element::Earth => EARTH,
            Element::Water => WATER,
            Element::Holy => HOLY,
        } * 20.0
            / 160.0;
        let x = self.current * 20.0 / 80.0;
        let w = 20.0 / 80.0;
        let h = 20.0 / 160.0;
        let rect = Rect { x, y, w, h };

        let pos = [
            self.pos.x + CARD_SIZE[0] / 2.0 - ELEM_SIZE / 2.0,
            self.pos.y + CARD_SIZE[1] / 2.0 - ELEM_SIZE / 2.0,
        ];
        let scale: [f32; 2] = [ELEM_SIZE / 20.0, ELEM_SIZE / 20.0];
        let color = Color::from_rgba(255, 255, 255, 200);
        array.push(
            DrawParam::default()
                .src(rect)
                .dest(pos)
                .scale(scale)
                .color(color),
        );
    }
}
