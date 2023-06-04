use ggez::graphics;
use mint::{Point2, Vector2};
use serde::Deserialize;
use serde_with::serde_as;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Meta {
    size: AtlasSize,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct AtlasSize {
    w: i16,
    h: i16,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Sprite {
    pub rect: graphics::Rect,
    pub scale: Vector2<f32>,
    pub width: f32,
    pub height: f32,
}

#[allow(dead_code)]
impl Sprite {
    pub const fn new(rect: graphics::Rect, width: f32, height: f32) -> Self {
        Self {
            rect,
            scale: Vector2 { x: 1.0, y: 1.0 },
            width,
            height,
        }
    }

    /// Adds a draw command to the sprite batch.
    pub fn add_draw_param(&mut self, pos: Point2<f32>) -> graphics::DrawParam {
        self.draw_params(pos)
    }

    pub fn draw_params(&self, pos: Point2<f32>) -> graphics::DrawParam {
        graphics::DrawParam::new()
            .src(self.rect)
            .scale(self.scale)
            .dest(pos)
    }

    /// Returns the bounding box for this sprite.
    pub fn get_bound_box(&self) -> graphics::Rect {
        let mut r = graphics::Rect::new(0.0, 0.0, self.width, self.height);
        r.scale(self.scale.x, self.scale.y);
        r
    }
}

#[derive(Deserialize, Debug, Clone)]
struct JsonRect {
    x: i16,
    y: i16,
    w: i16,
    h: i16,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct SpriteData {
    id: usize,
    pub frame: JsonRect,
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Atlas {
    #[serde_as(as = "Vec<(_)>")]
    sprites: Vec<SpriteData>,
    meta: Meta,
}
impl Atlas {
    pub fn parse_atlas_json(filename: &str) -> Self {
        use std::fs::File;
        use std::io::BufReader;
        let path = std::path::PathBuf::from(filename);
        println!("Loading {path:?}");
        let file = File::open(path).expect("Couldn't find the texture_atlas file");
        let buf_reader = BufReader::new(file);
        serde_json::from_reader(buf_reader).expect("Couldn't create texture atlas")
    }

    /// Returns a sprite from the Atlas.
    pub fn create_sprite(&self, id: usize) -> Sprite {
        let width = f32::from(self.meta.size.w);
        let height = f32::from(self.meta.size.h);
        let atlas_rect = graphics::Rect::new(0.0, 0.0, width, height);

        let sprite_data = &self.sprites[id];

        Sprite::new(
            graphics::Rect::fraction(
                f32::from(sprite_data.frame.x),
                f32::from(sprite_data.frame.y),
                f32::from(sprite_data.frame.w),
                f32::from(sprite_data.frame.h),
                &atlas_rect,
            ),
            f32::from(sprite_data.frame.w),
            f32::from(sprite_data.frame.h),
        )
    }
}

// self.sprites
//     .iter()
//     .find(|d| d.id == sprite_name)
//     .map_or_else(
//         || {
//             println!("Sprite name: {sprite_name}");
//             unimplemented!("Not handling failure to find sprite");
//         },
//         |sprite_data| {
//             Sprite::new(
//                 graphics::Rect::fraction(
//                     sprite_data.frame.x as f32,
//                     sprite_data.frame.y as f32,
//                     sprite_data.frame.w as f32,
//                     sprite_data.frame.h as f32,
//                     &atlas_rect,
//                 ),
//                 sprite_data.frame.w as f32,
//                 sprite_data.frame.h as f32,
//             )
//         },
//     )
