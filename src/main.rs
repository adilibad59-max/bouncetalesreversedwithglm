//! Bounce Tales RS — yuvarlanan top platform oyunu (Rust + macroquad)
//!
//! Orijinal J2ME oyunundan tersine mühendislikle çıkarılan *mekaniklere*
//! dayalı, sıfırdan yazılmış özgün bir implementasyon:
//!   - Eğri (vektör) tabanlı arazi üzerinde yuvarlanan top fiziği
//!   - 16.16 sabit nokta yerine f32; dönüş açısı yuvarlanmayla senkron
//!   - 240x320 mantıksal çözünürlük (orijinal J2ME canvas boyutu), ölçekli çizim
//!   - Durum makinesi: Menü → Oyun → Seviye Tamam → Bitiş
//!   - Dokunmatik (Android) + klavye (masaüstü) girdi
//!
//! Bu modül: uygulama girişi, oyun döngüsü ve durum makinesi.

mod ball;
mod entities;
mod input;
mod level;
mod render;

use macroquad::prelude::*;

use ball::Ball;
use entities::Entities;
use input::Controls;
use level::Level;
use render::{virtual_to_screen, Renderer, VW, VH};

#[derive(Clone, Copy, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    LevelDone,
    GameDone,
}

pub struct Game {
    pub state: GameState,
    pub level_idx: usize,
    pub level: Level,
    pub ball: Ball,
    pub ents: Entities,
    pub controls: Controls,
    pub camera: Vec2,
    pub time: f32,
    pub fade: f32,       // geçiş animasyonu 0..1
    pub fade_out: bool,  // true: karartma, false: açılma
    pub deaths: u32,
}

impl Game {
    pub fn new() -> Self {
        // BOUNCE_AUTOSTART=1: otomatik test modu (menüyü atla)
        let autostart = std::env::var("BOUNCE_AUTOSTART").ok()
            .map(|v| v == "1").unwrap_or(false);
        let mut g = Self {
            state: GameState::Menu,
            level_idx: 0,
            level: Level::demo(0),
            ball: Ball::new(Vec2::ZERO),
            ents: Entities::default(),
            controls: Controls::default(),
            camera: Vec2::ZERO,
            time: 0.0,
            fade: 1.0,
            fade_out: false,
            deaths: 0,
        };
        if autostart {
            g.load_level(0);
            g.state = GameState::Playing;
        }
        g
    }

    pub fn load_level(&mut self, idx: usize) {
        self.level_idx = idx;
        self.level = Level::demo(idx);
        self.ball = Ball::new(self.level.spawn);
        self.ents = Entities::from_level(&self.level);
        self.camera = self.level.spawn - Vec2::new(VW * 0.4, VH * 0.55);
        self.clamp_camera();
        self.time = 0.0;
        self.fade_out = false;
        self.fade = 1.0;
    }

    fn clamp_camera(&mut self) {
        self.camera.x = self.camera.x.clamp(0.0, (self.level.width - VW).max(0.0));
        self.camera.y = self.camera.y.clamp(0.0, (self.level.height - VH).max(0.0));
    }

    /// Oyun mantığı güncellemesi (sabit üst sınır clamp'li dt).
    pub fn update(&mut self, dt: f32) {
        // geçiş karartması
        if self.fade_out {
            self.fade += dt * 2.5;
        } else {
            self.fade = (self.fade - dt * 2.5).max(0.0);
        }

        match self.state {
            GameState::Menu => {
                self.controls.update();
                if self.controls.jump_just_pressed || self.controls.any_tap {
                    self.load_level(0);
                    self.state = GameState::Playing;
                }
            }
            GameState::Playing => {
                self.controls.update();
                if self.controls.restart_just_pressed {
                    self.load_level(self.level_idx);
                }

                self.time += dt;
                // top fiziği
                self.ball.update(
                    dt,
                    self.controls.left_held || self.controls.right_held,
                    self.controls.left_held,
                    self.controls.right_held,
                    &self.level,
                );

                // top girdisi: zıplama
                if self.controls.jump_just_pressed {
                    self.ball.try_jump();
                }

                // varlık etkileşimleri
                let outcome = self.ents.update(dt, self.ball.pos);
                match outcome {
                    entities::Outcome::None => {}
                    entities::Outcome::Died => {
                        self.deaths += 1;
                        self.ball = Ball::new(self.level.spawn);
                    }
                    entities::Outcome::Goal => {
                        self.state = GameState::LevelDone;
                        self.fade_out = false;
                        self.fade = 1.0;
                    }
                }

                // kamera takibi (yumuşak)
                let target = self.ball.pos - Vec2::new(VW * 0.42, VH * 0.58);
                self.camera += (target - self.camera) * (1.0 - (-8.0 * dt).exp());
                self.clamp_camera();
            }
            GameState::LevelDone => {
                self.controls.update();
                if self.controls.jump_just_pressed || self.controls.any_tap {
                    if self.level_idx + 1 < Level::demo_count() {
                        self.load_level(self.level_idx + 1);
                        self.state = GameState::Playing;
                    } else {
                        self.state = GameState::GameDone;
                        self.fade_out = false;
                        self.fade = 1.0;
                    }
                }
            }
            GameState::GameDone => {
                self.controls.update();
                if self.controls.restart_just_pressed {
                    self.load_level(0);
                    self.state = GameState::Playing;
                }
            }
        }
    }
}

fn conf() -> Conf {
    Conf {
        window_title: "Bounce Tales RS".into(),
        window_width: 480,
        window_height: 640,
        window_resizable: false,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut game = Game::new();
    let renderer = Renderer::new();
    let mut last = get_time();

    loop {
        // dt: orijinaldeki gibi üst sınıra clamp (J2ME 500 ms cap → 100 ms)
        let now = get_time();
        let dt = ((now - last) as f32).clamp(0.0, 0.1);
        last = now;

        game.update(dt);

        // --- çizim (sanal 240x320 uzaya ölçekli) ---
        clear_background(BLACK);
        let (_scale, _ox, _oy) = virtual_to_screen();
        render::push_virtual_camera();

        match game.state {
            GameState::Menu => renderer.draw_menu(&game),
            GameState::Playing => renderer.draw_world(&game),
            GameState::LevelDone => {
                renderer.draw_world(&game);
                renderer.draw_level_done(&game);
            }
            GameState::GameDone => renderer.draw_game_done(&game),
        }

        render::pop_virtual_camera();

        // karartma perdesi
        if game.fade > 0.0 {
            let a = game.fade.min(1.0);
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                           Color::new(0.0, 0.0, 0.0, a));
        }

        next_frame().await;
    }
}
