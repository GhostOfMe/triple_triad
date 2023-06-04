use ggez::{
    graphics::{Canvas, Color, DrawParam, Image},
    Context,
};

use tween::{Linear, Tweener};

use crate::{consts::WINDOW_DIMENSIONS, utils::Event};

pub const DURATION: f32 = 2.0;

struct Tween {
    pub active: bool,
    pub tweener: Tweener<f32, f32, Linear>,
}

pub struct Fade {
    color: Color,
    color_rect: Image,
    fade_in: Tween,
    fade_out: Tween,
}

impl Fade {
    pub fn new(ctx: &mut Context) -> Self {
        let color = Color::new(0.0, 0.0, 0.0, 1.0);

        let color_rect = Image::from_solid(ctx, 1, color);
        let fade_in = Tween {
            active: false,
            tweener: Tweener::new(1.0, 0.0, DURATION, Linear),
        };
        let fade_out = Tween {
            active: false,
            tweener: Tweener::new(0.0, 1.0, DURATION, Linear),
        };

        Self {
            color,
            color_rect,
            fade_in,
            fade_out,
        }
    }
    pub fn fade_in(&mut self) {
        self.fade_in.active = true;
        self.fade_in.tweener.current_time = 0.0;
    }
    pub fn fade_out(&mut self) {
        self.fade_out.active = true;
        self.fade_out.tweener.current_time = 0.0;
    }

    pub fn fade_in_update(&mut self, dt: f32) -> Option<Event> {
        if !self.fade_in.active {
            self.fade_in()
        }
        self.color.a = self.fade_in.tweener.move_by(dt);
        self.fade_in.active = !self.fade_in.tweener.is_finished();
        if !self.fade_in.active {
            return Some(Event::Finished);
        }
        None
    }
    pub fn fade_out_update(&mut self, dt: f32) -> Option<Event> {
        if !self.fade_out.active {
            self.fade_out()
        }
        self.color.a = self.fade_out.tweener.move_by(dt);
        self.fade_out.active = !self.fade_out.tweener.is_finished();
        if !self.fade_out.active {
            return Some(Event::Finished);
        }
        None
    }

    // pub fn update(&mut self, dt: f32) -> Option<Event> {
    //     match (self.fade_in.active, self.fade_out.active) {
    //         (true, true) => unreachable!(),
    //         (false, false) => {}
    //         (true, _) => {
    //             self.color.a = self.fade_in.tweener.move_by(dt);
    //             self.fade_in.active = !self.fade_in.tweener.is_finished();
    //             if !self.fade_in.active {
    //                 return Some(Event::Finished);
    //             }
    //         }
    //         (_, true) => {
    //             self.color.a = self.fade_out.tweener.move_by(dt);
    //             self.fade_out.active = !self.fade_out.tweener.is_finished();
    //             if !self.fade_out.active {
    //                 return Some(Event::Finished);
    //             }
    //         }
    //     }

    //     None
    // }

    pub fn draw(&self, canvas: &mut Canvas) {
        canvas.draw(
            &self.color_rect,
            DrawParam::default()
                .scale(WINDOW_DIMENSIONS)
                .color(self.color),
        )
    }
}
