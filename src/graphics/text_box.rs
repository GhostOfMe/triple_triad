use ggez::graphics::{Rect, Image, Color, DrawParam, Canvas};
use ggez::Context;
use mint::Vector2;

pub struct TextBox {
    pos: [f32; 2],
    rect: Rect,
    border_light: Image,
    border_dark: Image,
    border_outline: Image,
    image: Image,
}

impl TextBox{
    pub fn new(
        ctx: &mut Context,
        pos: impl Into<[f32; 2]>,
        size: impl Into<Vector2<f32>>,
        color: impl Into<Color>,
    ) -> Self {
        let image = Image::from_solid(ctx, 1, color.into());
        let border_light = Image::from_solid(ctx, 1, (110, 110, 110).into());
        let border_dark = Image::from_solid(ctx, 1, (52, 52, 52).into());
        let border_outline = Image::from_solid(ctx, 1, (22, 32, 34).into());
        let size = size.into();
        Self {
            pos: pos.into(),
            rect: Rect {
                x: 0.0,
                y: 0.0,
                w: size.x,
                h: size.y,
            },
            border_light,
            border_dark,
            border_outline,
            image,
        }
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        canvas.draw(
            &self.border_outline,
            DrawParam::default()
                .scale([self.rect.size().x + 10.0, self.rect.size().y + 10.0])
                .dest([self.pos[0] - 5.0, self.pos[1] - 5.0]),
        );
        canvas.draw(
            &self.border_light,
            DrawParam::default()
                .scale([self.rect.size().x + 8.0, self.rect.size().y + 8.0])
                .dest([self.pos[0] - 4.0, self.pos[1] - 4.0]),
        );
        canvas.draw(
            &self.border_dark,
            DrawParam::default()
                .scale([self.rect.size().x + 4.0, self.rect.size().y + 4.0])
                .dest([self.pos[0], self.pos[1]]),
        );

        canvas.draw(
            &self.image,
            DrawParam::default().scale(self.rect.size()).dest(self.pos),
        );
    }
}
