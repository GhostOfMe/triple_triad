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
        _color: impl Into<Color>,
    ) -> Self {
        //let image = Image::from_solid(ctx, 1, color.into());
        #[rustfmt::skip]
        let pixels =  [
            64, 64, 64, 255,
            68, 68, 68 ,255,
            72, 72, 72, 255,
            76, 76, 76, 255,
            80, 80, 80, 255,
            84, 84, 84, 255,
            88, 88, 88, 255,
            92, 92, 92, 255,
            96, 96, 96, 255,
            96, 96, 96, 255,
            96, 96, 96, 255,
            96, 96, 96, 255
        ];
        let image = Image::from_pixels(ctx, &pixels, wgpu::TextureFormat::Rgba8UnormSrgb, 12, 1); 
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
            &self.border_dark,
            DrawParam::default()
                .scale([self.rect.size().x + 8.0, self.rect.size().y + 4.0])
                .dest([self.pos[0] - 4.0, self.pos[1]]),
        );
        canvas.draw(
            &self.border_light,
            DrawParam::default()
                .scale([self.rect.size().x + 4.0, self.rect.size().y + 4.0])
                .dest([self.pos[0] - 4.0, self.pos[1] - 4.0]),
        );
        let scale = [self.rect.size().x / self.image.width() as f32, self.rect.size().y];
        canvas.draw(
            &self.image,
            DrawParam::default().scale(scale).dest(self.pos),
        );
    }
}
