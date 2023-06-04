use ggez::event::MouseButton;
use ggez::graphics::{
    Canvas, Color, DrawParam, Image, InstanceArray, PxScale, Rect, Text, TextFragment,
};
use ggez::Context;

use mint::Point2;

use std::rc::Rc;

use crate::consts::{FONT_SIZE, RIGHT_ARROW_SPRITE_ID};
use crate::core::{self, CardAtlas};
use crate::graphics::{sprite::Atlas, CardEntity, TextBox};

const NAV_BTNS_Y: f32 = 292.0;
const PREV_BTN_X: f32 = 68.0;
const NEXT_BTN_X: f32 = 510.0;
const NAV_BTN_SIZE: f32 = 32.0;

const CARDS_PER_PAGE: usize = 11;

// const CARD_SELECT_NUM_X: f32 = 405.0;

// const MENU_BG_POS: [f32; 2] = [286.0, 116.0];
// const MENU_BG_DIMENSIONS: [f32; 2] = [266.0, 312.0];
// const PLAY_BUTTON_POS: [f32; 2] = [370.0, 336.0];
// const QUIT_BUTTON_POS: [f32; 2] = [370.0, 377.0];

const CARD_SELECT_VIEW_POS: [f32; 2] = [130.0, 92.0];
const CARD_SELECT_DIMENSIONS: [f32; 2] = [350.0, 400.0];
const CARD_SELECT_TEXT_POS: [f32; 2] = [175.0, 105.0];
const CARD_SELECT_LINE_GAP: f32 = 10.0;
pub struct MenuCardItem {
    pub label: String,
    pub id: usize,
    pub rect: Rect,
}

impl MenuCardItem {
    pub fn draw(&self, canvas: &mut Canvas) {
        let label = Text::new(TextFragment {
            text: self.label.clone(),
            color: None,
            font: Some("pixel font".into()),
            scale: Some(PxScale::from(FONT_SIZE)),
        });
        let shadow = Text::new(TextFragment {
            text: self.label.clone(),
            color: Some(Color::from_rgb(50, 50, 50)),
            font: Some("pixel font".into()),
            scale: Some(PxScale::from(FONT_SIZE)),
        });
        canvas.draw(&shadow, [self.rect.x + 2.0, self.rect.y + 2.0]);
        canvas.draw(&label, self.rect.point());
    }
}

pub enum PageNavEvent {
    Next,
    Prev,
}

pub struct PageNavButton {
    pub sprite_id: usize,
    pub scale: Point2<f32>,
    pub rect: Rect,
    pub active: bool,
    pub flipped: bool,
    pub callback: Box<dyn Fn() -> PageNavEvent>,
    sprite_sheet: Rc<Atlas>,
    pub bg_rect: Image,
}

impl PageNavButton {
    pub fn new_right(ctx: &mut Context, sprite_sheet: &Rc<Atlas>) -> Self {
        let bg_rect = Image::from_solid(ctx, 1, Color::from_rgb(0, 255, 0));
        Self {
            sprite_id: crate::consts::RIGHT_ARROW_SPRITE_ID,
            scale: [2.0, 2.0].into(),
            rect: Rect {
                x: NEXT_BTN_X,
                y: NAV_BTNS_Y,
                w: NAV_BTN_SIZE,
                h: NAV_BTN_SIZE,
            },
            active: true,
            flipped: false,
            callback: Box::new(|| PageNavEvent::Next),
            bg_rect,
            sprite_sheet: Rc::clone(sprite_sheet),
        }
    }

    pub fn new_left(ctx: &mut Context, sprite_sheet: &Rc<Atlas>) -> Self {
        let mut btn = Self::new_right(ctx, sprite_sheet);
        btn.callback = Box::new(|| PageNavEvent::Prev);
        btn.flipped = true;
        btn.rect = Rect {
            x: PREV_BTN_X,
            y: NAV_BTNS_Y,
            w: NAV_BTN_SIZE,
            h: NAV_BTN_SIZE,
        };

        btn
    }

    pub fn draw(&self, _ctx: &mut Context, canvas: &mut Canvas, array: &mut InstanceArray) {
        if !self.active {
            return;
        }

        //let pos = self.rect.point();
        let dimensions = self.rect.size();

        // canvas.draw(
        //     &self.bg_rect,
        //     DrawParam::default().dest(pos).scale(dimensions),
        // );

        array.clear();
        let rect = self.sprite_sheet.create_sprite(RIGHT_ARROW_SPRITE_ID).rect;
        let flip = if self.flipped { -1.0 } else { 1.0 };
        let flip_x_offset = if self.flipped { 64.0 } else { 0.0 };
        let dest = [
            self.rect.point().x - 16.0 + flip_x_offset,
            self.rect.point().y - 16.0,
        ];

        array.push(
            DrawParam::default()
                .src(rect)
                .offset([0.5, 0.5])
                .dest(dest)
                .scale([dimensions.x / 8.0 * flip, dimensions.y / 8.0]),
        );
        canvas.draw(array, [0.0, 0.0]);
    }
}
pub struct PickMenu {
    pub active: bool,
    page: usize,
    pages: usize,
    count: u8,
    cards: Vec<(usize, String)>,
    next_btn: PageNavButton,
    prev_btn: PageNavButton,
    items: Vec<Option<MenuCardItem>>,
    bg_rect: TextBox,
    card_atlas: Rc<CardAtlas>,
}

impl PickMenu {
    pub fn new(ctx: &mut Context, sprite_sheet: &Rc<Atlas>, card_atlas: &Rc<CardAtlas>) -> Self {
        let next_btn = PageNavButton::new_right(ctx, sprite_sheet);
        let prev_btn = PageNavButton::new_left(ctx, sprite_sheet);
        let items = vec![
            None, None, None, None, None, None, None, None, None, None, None,
        ];
        let bg_rect = TextBox::new(
            ctx,
            CARD_SELECT_VIEW_POS,
            CARD_SELECT_DIMENSIONS,
            (64, 64, 64),
        );
        let mut res = Self {
            active: true,
            page: 0,
            pages: 0,
            count: 0,
            cards: Vec::with_capacity(11),
            next_btn,
            prev_btn,
            items,
            bg_rect,
            card_atlas: Rc::clone(card_atlas),
        };

        res.init();
        res
    }

    pub fn init(&mut self) {
        self.cards.clear();

        for item in &mut self.items {
            *item = None;
        }
        self.cards = (0..110)
            .skip(self.page * CARDS_PER_PAGE)
            .take(CARDS_PER_PAGE)
            .map(|id| (id, self.card_atlas.cards[id].name.clone()))
            .collect();

        for (i, card) in self.cards.iter().enumerate() {
            let i_small = i16::try_from(i).expect("Value is too big! {i}");

            let item = MenuCardItem {
                label: card.1.to_string(),
                id: card.0,
                rect: Rect {
                    x: CARD_SELECT_TEXT_POS[0],
                    y: (FONT_SIZE + CARD_SELECT_LINE_GAP)
                        .mul_add(f32::from(i_small), CARD_SELECT_TEXT_POS[1]),
                    w: 100.0,
                    h: FONT_SIZE,
                },
            };
            self.items[i] = Some(item);
        }
    }

    pub fn update(&mut self, ctx: &mut Context) -> Option<(usize, u8)> {
        if !self.active {
            return None;
        }
        self.pages = 10;

        if !ctx.mouse.button_just_pressed(MouseButton::Left) {
            return None;
        }

        self.next_btn.active = true;
        self.prev_btn.active = true;
        // if collection.len() < 11 {
        //     self.next_btn.active = false;
        //     self.prev_btn.active = false;
        // }

        for item in self.items.iter().flatten() {
            if item.rect.contains(ctx.mouse.position()) {
                self.count += 1;
                return Some((item.id, self.count - 1));
            }
        }

        if self.next_btn.rect.contains(ctx.mouse.position()) {
            self.next_page();
        }
        if self.prev_btn.rect.contains(ctx.mouse.position()) {
            self.prev_page();
        }

        None
    }

    pub fn update_hover(&self, ctx: &mut Context) -> Option<usize> {
        for item in self.items.iter().flatten() {
            if item.rect.contains(ctx.mouse.position()) {
                return Some(item.id);
            }
        }
        None
    }

    // pub fn reset(&mut self) {
    //     self.count = 0;
    //     self.page = 0;
    // }

    fn next_page(&mut self) {
        if self.page == self.pages - 1 {
            self.page = 0;
        } else {
            self.page += 1;
        }
        self.init();
    }

    fn prev_page(&mut self) {
        if self.page == 0 {
            self.page = self.pages - 1;
        } else {
            self.page -= 1;
        }
        self.init();
    }
    pub fn draw(&self, ctx: &mut Context, array: &mut InstanceArray, canvas: &mut Canvas) {
        if !self.active {
            return;
        }

        self.bg_rect.draw(canvas);

        // canvas.draw(
        //     &self.bg_rect,
        //     DrawParam::default()
        //         .scale(CARD_SELECT_DIMENSIONS)
        //         .dest(CARD_SELECT_VIEW_POS),
        // );

        for item in self.items.iter().flatten() {
            item.draw(canvas);
        }
        self.next_btn.draw(ctx, canvas, array);
        self.prev_btn.draw(ctx, canvas, array);
    }
}

pub struct CardSelect {
    pub card_menu: PickMenu,
    card_preview: CardEntity,
    show_preview: bool,
    pub active: bool,
}

impl CardSelect {
    pub fn new(ctx: &mut Context, card_atlas: &Rc<CardAtlas>, sprite_sheet: &Rc<Atlas>) -> Self {
        let card_menu = PickMenu::new(ctx, sprite_sheet, card_atlas);
        let card_preview = CardEntity::new(
            0,
            [500.0, 330.0].into(),
            core::Suit::Blue,
            core::Suit::Blue,
            false,
            card_atlas,
            sprite_sheet,
        );
        Self {
            card_menu,
            card_preview,
            show_preview: false,
            active: true,
        }
    }
    pub fn init(&mut self){
        self.card_menu.count = 0;
    }
    pub fn draw(&self, ctx: &mut Context, canvas: &mut Canvas, array: &mut InstanceArray) {
        self.card_menu.draw(ctx, array, canvas);

        if self.show_preview {
            array.clear();
            self.card_preview.add_to_instance_array(array);
            canvas.draw(array, DrawParam::default());
        }
    }

    pub fn update(&mut self, ctx: &mut Context) -> Option<(usize, u8)> {
        self.show_preview = false;
        if let Some(id) = self.card_menu.update_hover(ctx) {
            self.show_preview = true;
            self.card_preview.id = id;
        }

        self.card_menu.update(ctx)
    }
}
