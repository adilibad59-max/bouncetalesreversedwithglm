//! Çizim katmanı — tamamen prosedürel grafik (harici varlık gerektirmez).
//!
//! Sanal çözünürlük 240x320 (orijinal J2ME canvas boyutu); tüm dünya
//! koordinatları bu uzayda tutulur ve ekrana ölçekle taşınır.

use macroquad::prelude::*;

use crate::ball::{Ball, RADIUS};
use crate::entities::Entities;
use crate::level::Level;
use crate::{Game, GameState};
pub const VW: f32 = 240.0;
pub const VH: f32 = 320.0;

pub fn virtual_to_screen() -> (f32, f32, f32) {
    let sw = screen_width();
    let sh = screen_height();
    let scale = (sw / VW).min(sh / VH);
    let ox = (sw - VW * scale) * 0.5;
    let oy = (sh - VH * scale) * 0.5;
    (scale, ox, oy)
}

pub fn push_virtual_camera() {
    let (scale, ox, oy) = virtual_to_screen();
    let vw = (VW * scale) as i32;
    let vh = (VH * scale) as i32;
    // macroquad viewport y-ekseni alttan ölçülür
    let vy = (screen_height() as i32) - (oy as i32) - vh;
    let cam = Camera2D {
        zoom: Vec2::new(2.0 / VW, 2.0 / VH),
        target: Vec2::new(VW * 0.5, VH * 0.5),
        viewport: Some((ox as i32, vy, vw, vh)),
        ..Default::default()
    };
    set_camera(&cam);
}

pub fn pop_virtual_camera() {
    set_default_camera();
}

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Self
    }

    // ----------------------------------------------------------------- menü
    pub fn draw_menu(&self, g: &Game) {
        // gökyüzü
        let sky_top = Color::new(0.45, 0.75, 0.95, 1.0);
        let sky_bot = Color::new(0.75, 0.92, 1.0, 1.0);
        draw_gradient_rect(0.0, 0.0, VW, VH, sky_top, sky_bot);

        // dekoratif tepeler
        for (i, (par, col)) in [
            (0.35, Color::new(0.55, 0.80, 0.55, 1.0)),
            (0.6, Color::new(0.40, 0.68, 0.42, 1.0)),
        ]
        .iter()
        .enumerate()
        {
            let off = (g.time0()) * par * 18.0;
            for k in 0..4 {
                let cx = -off % 180.0 + k as f32 * 180.0 - 40.0;
                draw_hill(cx + i as f32 * 60.0, 300.0, 130.0, 70.0, *col);
            }
        }

        // zıplayan top animasyonu
        let t = g.time0() * 2.2;
        let by = 150.0 - (t.sin().max(0.0)) * 60.0;
        self.draw_ball_shape(Vec2::new(VW * 0.5, by), t * 1.5);

        // başlık
        draw_text_centered("BOUNCE", VW * 0.5, 60.0, 34, WHITE);
        draw_text_centered("TALES  RS", VW * 0.5, 88.0, 26, Color::new(1.0, 0.95, 0.4, 1.0));
        draw_text_centered("Rust + macroquad", VW * 0.5, 106.0, 11, Color::new(0.2, 0.2, 0.2, 1.0));

        draw_text_centered("Dokun / SPACE: basla", VW * 0.5, 240.0, 14, Color::new(0.15, 0.25, 0.15, 1.0));
        draw_text_centered("Sol/Sag: hareket  Ust: zipla", VW * 0.5, 258.0, 11, Color::new(0.15, 0.25, 0.15, 1.0));
        draw_text_centered("R: yeniden basla", VW * 0.5, 274.0, 11, Color::new(0.15, 0.25, 0.15, 1.0));
    }

    // ----------------------------------------------------------------- dünya
    pub fn draw_world(&self, g: &Game) {
        let cam = g.camera;
        let lv = &g.level;

        // gökyüzü + paralaks
        let sky_top = Color::new(0.45, 0.75, 0.95, 1.0);
        let sky_bot = Color::new(0.78, 0.93, 1.0, 1.0);
        draw_gradient_rect(0.0, 0.0, VW, VH, sky_top, sky_bot);

        // uzak bulutlar
        let cl = (cam.x * 0.25) % (VW + 160.0);
        for k in 0..3 {
            let x = k as f32 * 130.0 - cl + 40.0;
            if x > -80.0 && x < VW + 40.0 {
                draw_cloud(x, 50.0 + (k % 2) as f32 * 28.0, 1.0);
            }
        }
        // paralaks tepeler
        let hoff = (cam.x * 0.4) % 240.0;
        for k in 0..13 {
            let x = k as f32 * 240.0 - hoff;
            draw_hill(x, 330.0, 150.0, 95.0, Color::new(0.60, 0.82, 0.60, 1.0));
            draw_hill(x + 120.0, 330.0, 110.0, 75.0, Color::new(0.50, 0.75, 0.52, 1.0));
        }

        // --- arazi: zincir altını doldur ---
        let vis: Vec<Vec2> = lv
            .chain
            .iter()
            .map(|p| Vec2::new(p.x - cam.x, p.y - cam.y))
            .filter(|p| p.x > -60.0 && p.x < VW + 60.0)
            .collect();
        if vis.len() >= 2 {
            // gövde
            let mut poly: Vec<Vec2> = vis.clone();
            poly.push(Vec2::new(vis.last().unwrap().x, VH + 40.0));
            poly.push(Vec2::new(vis[0].x, VH + 40.0));
            draw_poly_fill(&poly, Color::new(0.32, 0.55, 0.30, 1.0));
            // üst çim şeridi
            for w in vis.windows(2) {
                draw_line(w[0].x, w[0].y, w[1].x, w[1].y, 5.0, Color::new(0.42, 0.72, 0.38, 1.0));
                draw_line(w[0].x, w[0].y + 3.0, w[1].x, w[1].y + 3.0, 2.0, Color::new(0.55, 0.85, 0.45, 1.0));
            }
        }

        // varlıklar
        self.draw_entities(&g.ents, cam);

        // top
        self.draw_ball_shape(Vec2::new(g.ball.pos.x - cam.x, g.ball.pos.y - cam.y), g.ball.angle);

        // HUD
        self.draw_hud(g);

        // dokunmatik yön bölgeleri (hafif ipuçları)
        if is_touch_device_hint() {
            let a = 0.10 + 0.05 * (g.time0() * 2.0).sin();
            draw_triangle_hint(40.0, VH - 38.0, true, Color::new(1.0, 1.0, 1.0, a));
            draw_triangle_hint(VW - 40.0, VH - 38.0, false, Color::new(1.0, 1.0, 1.0, a));
        }
    }

    fn draw_entities(&self, ents: &Entities, cam: Vec2) {
        // küreler
        for (p, got) in &ents.orbs {
            if *got {
                continue;
            }
            let x = p.x - cam.x;
            let y = p.y - cam.y + (ents.bob * 0.8).sin() * 2.5;
            if x < -20.0 || x > VW + 20.0 {
                continue;
            }
            draw_circle(x, y, 6.0, Color::new(1.0, 0.85, 0.25, 1.0));
            draw_circle(x - 1.5, y - 1.5, 2.0, Color::new(1.0, 1.0, 0.75, 1.0));
            draw_circle_lines(x, y, 6.0, 1.0, Color::new(0.75, 0.55, 0.05, 1.0));
        }
        // dikenler
        for s in &ents.spikes {
            let x = s.x - cam.x;
            let y = s.y - cam.y;
            if x < -20.0 || x > VW + 20.0 {
                continue;
            }
            for k in -1..=1i32 {
                let bx = x + k as f32 * 9.0;
                draw_triangle(
                    Vec2::new(bx - 6.0, y + 8.0),
                    Vec2::new(bx + 6.0, y + 8.0),
                    Vec2::new(bx, y - 10.0),
                    Color::new(0.85, 0.20, 0.20, 1.0),
                );
            }
        }
        // bitiş halkası
        let gx = ents.goal.x - cam.x;
        let gy = ents.goal.y - cam.y;
        let r = 13.0 + (ents.goal_spin * 1.4).sin() * 1.5;
        for i in 0..8 {
            let a0 = ents.goal_spin + i as f32 * std::f32::consts::TAU / 8.0;
            let a1 = a0 + std::f32::consts::TAU / 10.0;
            draw_arc(gx, gy, r, a0, a1, 3.0,
                     Color::new(0.95, 0.75, 0.95, 1.0));
        }
        draw_circle(gx, gy, 3.0, Color::new(1.0, 0.55, 0.9, 1.0));
    }

    // ------------------------------------------------------------------- top
    /// Deseni dönen top: kırmızı gövde + açık şeritler + iki nokta.
    /// Dönme açısı `angle` (radyan) desenin tamamını döndürür → yuvarlanma görünür.
    pub fn draw_ball_shape(&self, pos: Vec2, angle: f32) {
        // gövde
        draw_circle(pos.x, pos.y, RADIUS, Color::new(0.85, 0.20, 0.16, 1.0));
        draw_circle(pos.x - RADIUS * 0.3, pos.y - RADIUS * 0.35, RADIUS * 0.45,
                    Color::new(0.95, 0.40, 0.32, 1.0));
        draw_circle_lines(pos.x, pos.y, RADIUS, 1.5, Color::new(0.55, 0.10, 0.08, 1.0));

        // dönen desen: 3 kavisli şerit + 2 beyaz nokta (yön göstergesi)
        for i in 0..3 {
            let a = angle + i as f32 * std::f32::consts::TAU / 3.0;
            let p = Vec2::new(a.cos(), a.sin()) * (RADIUS * 0.55);
            draw_circle(pos.x + p.x, pos.y + p.y, 2.6, Color::new(1.0, 0.90, 0.75, 1.0));
        }
        // merkezde küçük parlak nokta
        draw_circle(pos.x, pos.y, 3.2, Color::new(1.0, 0.97, 0.85, 1.0));
    }

    // ------------------------------------------------------------------- HUD
    fn draw_hud(&self, g: &Game) {
        // üst bar
        draw_rectangle(0.0, 0.0, VW, 16.0, Color::new(0.0, 0.0, 0.0, 0.35));
        let name: String = g.level.name.chars().take(18).collect();
        draw_text(&name, 4.0, 12.0, 11.0, WHITE);
        let orbs = format!("Kure {}/{}", g.ents.collected(), g.ents.total());
        draw_text(&orbs, VW * 0.5 - 20.0, 12.0, 11.0, Color::new(1.0, 0.9, 0.3, 1.0));
        let t = format!("{:.1}s", g.time);
        draw_text(&t, VW - 40.0, 12.0, 11.0, WHITE);
    }

    pub fn draw_level_done(&self, g: &Game) {
        let a = (1.0 - g.fade.min(1.0)).max(0.0);
        draw_rectangle(0.0, VH * 0.28, VW, 120.0, Color::new(0.0, 0.0, 0.0, 0.55));
        draw_text_centered("SEVIYE TAMAM!", VW * 0.5, VH * 0.28 + 40.0, 22,
                           Color::new(1.0, 0.95, 0.5, 1.0));
        let s = format!("Sure: {:.1} s   Kure: {}/{}", g.time, g.ents.collected(), g.ents.total());
        draw_text_centered(&s, VW * 0.5, VH * 0.28 + 70.0, 13, WHITE);
        draw_text_centered("Devam icin dokun", VW * 0.5, VH * 0.28 + 100.0, 12,
                           Color::new(1.0, 1.0, 1.0, a.max(0.6)));
    }

    pub fn draw_game_done(&self, g: &Game) {
        let sky_top = Color::new(0.20, 0.45, 0.75, 1.0);
        let sky_bot = Color::new(0.55, 0.80, 1.0, 1.0);
        draw_gradient_rect(0.0, 0.0, VW, VH, sky_top, sky_bot);
        self.draw_ball_shape(Vec2::new(VW * 0.5, 120.0 + (g.time0() * 2.0).sin() * 8.0), g.time0() * 3.0);
        draw_text_centered("TEBRIKLER!", VW * 0.5, 180.0, 28, Color::new(1.0, 0.95, 0.4, 1.0));
        draw_text_centered("Butun seviyeler bitti.", VW * 0.5, 205.0, 13, WHITE);
        let s = format!("Toplam olum: {}", g.deaths);
        draw_text_centered(&s, VW * 0.5, 225.0, 12, WHITE);
        draw_text_centered("R: bastan oyna", VW * 0.5, 260.0, 12, WHITE);
    }
}

// ---------------------------------------------------------------- yardımcılar
fn draw_text_centered(txt: &str, cx: f32, y: f32, size: u16, col: Color) {
    let dims = measure_text(txt, None, size, 1.0);
    draw_text(txt, cx - dims.width * 0.5, y, size as f32, col);
}

fn draw_gradient_rect(x: f32, y: f32, w: f32, h: f32, top: Color, bot: Color) {
    let n = 24;
    let step = h / n as f32;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let c = Color::new(
            top.r + (bot.r - top.r) * t,
            top.g + (bot.g - top.g) * t,
            top.b + (bot.b - top.b) * t,
            1.0,
        );
        draw_rectangle(x, y + i as f32 * step, w, step + 1.0, c);
    }
}

fn draw_hill(cx: f32, base: f32, w: f32, h: f32, col: Color) {
    let steps = 16;
    let mut pts = Vec::with_capacity(steps + 2);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = cx - w * 0.5 + w * t;
        let y = base - (t * std::f32::consts::PI).sin() * h;
        pts.push(Vec2::new(x, y));
    }
    pts.push(Vec2::new(cx + w * 0.5, base + 60.0));
    pts.push(Vec2::new(cx - w * 0.5, base + 60.0));
    draw_poly_fill(&pts, col);
}

fn draw_cloud(x: f32, y: f32, s: f32) {
    let c = Color::new(1.0, 1.0, 1.0, 0.85);
    draw_circle(x, y, 10.0 * s, c);
    draw_circle(x + 12.0 * s, y + 2.0 * s, 8.0 * s, c);
    draw_circle(x - 12.0 * s, y + 3.0 * s, 7.0 * s, c);
    draw_rectangle(x - 16.0 * s, y + 3.0 * s, 34.0 * s, 8.0 * s, c);
}

fn draw_arc(cx: f32, cy: f32, r: f32, a0: f32, a1: f32, thick: f32, col: Color) {
    let steps = 6;
    for i in 0..steps {
        let t0 = a0 + (a1 - a0) * i as f32 / steps as f32;
        let t1 = a0 + (a1 - a0) * (i + 1) as f32 / steps as f32;
        draw_line(
            cx + t0.cos() * r, cy + t0.sin() * r,
            cx + t1.cos() * r, cy + t1.sin() * r,
            thick, col,
        );
    }
}

fn draw_triangle_hint(cx: f32, cy: f32, left: bool, col: Color) {
    let d = if left { -1.0 } else { 1.0 };
    draw_triangle(
        Vec2::new(cx + d * 10.0, cy - 12.0),
        Vec2::new(cx + d * 10.0, cy + 12.0),
        Vec2::new(cx - d * 10.0, cy),
        col,
    );
}

/// Basit çokgen doldurma (yelpaze üçgenleme; konveks olmayan zincirler için
/// kaba ama bu oyunun arazi şekilleri için yeterli).
fn draw_poly_fill(pts: &[Vec2], col: Color) {
    if pts.len() < 3 {
        return;
    }
    // kulak kesme yerine: merkez noktasına yelpaze (görsel amaçlı)
    let mut cx = 0.0;
    let mut cy = 0.0;
    for p in pts {
        cx += p.x;
        cy += p.y;
    }
    let c = Vec2::new(cx / pts.len() as f32, cy / pts.len() as f32);
    for w in pts.windows(2) {
        draw_triangle(w[0], w[1], c, col);
    }
    // kapanış kenarı
    let n = pts.len();
    draw_triangle(pts[n - 1], pts[0], c, col);
}

fn is_touch_device_hint() -> bool {
    // Android'de pencere genişliği yükseklikten küçük → dikey mobil ekran
    screen_width() < screen_height()
}

// Game'e menü animasyonu için zaman erişimi (ayrı sayaç istemiyoruz; level
// tamamlanmadıysa 0'dan başlayan zaman kullanılır).
impl Game {
    pub fn time0(&self) -> f32 {
        match self.state {
            GameState::Playing | GameState::LevelDone => self.time,
            _ => get_time() as f32 % 100.0,
        }
    }
                  }
