use app::SoundManager;
use ggez::{
    event,
    audio,
    graphics::{Canvas, Color, FontData, Image, InstanceArray, Rect, Sampler},
    Context, GameResult,
};

use std::rc::Rc;

use crate::app::App;
use crate::core::CardAtlas;
use crate::graphics::sprite::Atlas as SpriteAtlas;

mod app;
mod consts;
mod core;
mod graphics;
mod state;
mod utils;

const CARD_SPRITESHEET_FILE: &str = "/card_sprites.png";
const ELEM_SPRITESHEET_FILE: &str = "/elem_animations.png";
const CARD_SPRITESHEET_DATA: &str = "./assets/card_sprites.json";
const CARD_ATLAS_JSON: &str = "./assets/cards.json";
const FF8_FONT: &str = "/seed-computer.ttf";
const BOARD: &str = "/board.png";

const SOUND_BGM: &str = "/sfx/src_assets_sounds_music.mp3";
const SOUND_FLIP: &str = "/sfx/src_assets_sounds_flip_card.mp3";
const SOUND_MOVE: &str = "/sfx/src_assets_sounds_move_card.mp3";
const SOUND_SELECT: &str = "/sfx/src_assets_sounds_select.mp3";
const SOUND_CANCEL: &str = "/sfx/src_assets_sounds_cancel.mp3";
const SOUND_VICORY: &str = "/sfx/src_assets_sounds_victory.mp3";

//const TRADE: &str = "/trade.png";

struct MainState {
    app: App,
    _card_atlas: Rc<CardAtlas>,
    _card_sprite_sheet: Rc<SpriteAtlas>,
}

impl MainState {
    fn new(
        ctx: &mut Context,
        card_atlas: CardAtlas,
        card_sprite_sheet: SpriteAtlas,
        board_bg: Image,
        card_instance_array: InstanceArray,
        elem_instance_array: InstanceArray,
        sound_manager: SoundManager,
    ) -> GameResult<Self> {
        let card_atlas = Rc::new(card_atlas);
        let card_sprite_sheet = Rc::new(card_sprite_sheet);
        let board_bg = Rc::new(board_bg);
        let app = App::new(
            ctx,
            &card_atlas,
            &card_sprite_sheet,
            &board_bg,
            card_instance_array,
            elem_instance_array,
            sound_manager
        );
        Ok(Self {
            _card_atlas: Rc::clone(&card_atlas),
            _card_sprite_sheet: Rc::clone(&card_sprite_sheet),
            app,
        })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.app.update(ctx)?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));
        canvas.set_sampler(Sampler::nearest_clamp());
        canvas.set_screen_coordinates(Rect {
            x: 0.0,
            y: 0.0,
            w: 800.0,
            h: 600.0,
        });

        self.app.draw(ctx, &mut canvas);
        canvas.finish(ctx)?;
        self.app.wgpu_shapes.draw(ctx);

        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = std::path::PathBuf::from("./assets");
    let cb = ggez::ContextBuilder::new("Triple Triad", "ggez")
        .window_mode(ggez::conf::WindowMode::default().transparent(true))
        .add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;
    let bgm = audio::Source::new(&ctx, SOUND_BGM)?;
    let flip_card = audio::Source::new(&ctx, SOUND_FLIP)?;
    let move_card = audio::Source::new(&ctx, SOUND_MOVE)?;
    let select = audio::Source::new(&ctx, SOUND_SELECT)?;
    let cancel = audio::Source::new(&ctx, SOUND_CANCEL)?;
    let victory = audio::Source::new(&ctx, SOUND_VICORY)?;
    
    let sound_manager = SoundManager{
        bgm,
        flip_card,
        move_card,
        select,
        cancel,
        victory
    };

    let card_instance_array = create_sprite_instance_array(&mut ctx, CARD_SPRITESHEET_FILE);
    let elem_instance_array = create_sprite_instance_array(&mut ctx, ELEM_SPRITESHEET_FILE);
    let card_atlas = CardAtlas::parse_atlas_json(CARD_ATLAS_JSON);
    let card_sprite_sheet = SpriteAtlas::parse_atlas_json(CARD_SPRITESHEET_DATA);
    let board_bg = Image::from_path(&ctx, BOARD)?;

    let state = MainState::new(
        &mut ctx,
        card_atlas,
        card_sprite_sheet,
        board_bg,
        card_instance_array,
        elem_instance_array,
        sound_manager
    )?;

    ctx.gfx
        .add_font("pixel font", FontData::from_path(&ctx, FF8_FONT)?);
    event::run(ctx, event_loop, state)
}

fn create_sprite_instance_array(ctx: &mut Context, filename: &str) -> InstanceArray {
    let image = Image::from_path(ctx, filename).expect("Error while opening a file {filename}");

    InstanceArray::new(ctx, image)
}
