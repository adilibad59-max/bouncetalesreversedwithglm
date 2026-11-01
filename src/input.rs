//! Girdi katmanı — klavye (masaüstü) + dokunmatik (Android) birleşik kontrol.
//!
//! Android şeması (J2ME portlarında yaygın üç-bölge düzeni):
//!   * sol alt bölge  → sola git
//!   * sağ alt bölge  → sağa git
//!   * ekranın üst yarısına dokunma → zıpla
//! Çoklu dokunma desteklidir: yön basılıyken zıplanabilir.

use macroquad::prelude::*;

use crate::render::{VH, VW};

#[derive(Default)]
pub struct Controls {
    pub left_held: bool,
    pub right_held: bool,
    pub jump_just_pressed: bool,
    pub restart_just_pressed: bool,
    pub any_tap: bool,
    prev_jump: bool,
    prev_restart: bool,
    prev_taps: Vec<u64>,
}

impl Controls {
    pub fn update(&mut self) {
        // ---- klavye ----
        let kb_left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
        let kb_right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
        let kb_jump = is_key_down(KeyCode::Space)
            || is_key_down(KeyCode::Up)
            || is_key_down(KeyCode::W);
        let kb_restart = is_key_down(KeyCode::R);

        // ---- dokunmatik: sanal koordinatta bölge testi ----
        let (scale, ox, oy) = crate::render::virtual_to_screen();
        let inv = if scale != 0.0 { 1.0 / scale } else { 0.0 };
        let mut t_left = false;
        let mut t_right = false;
        let mut t_jump = false;
        let mut tap_now = false;
        let mut cur_ids: Vec<u64> = Vec::new();

        for t in touches() {
            cur_ids.push(t.id);
            let vx = (t.position.x - ox) * inv;
            let vy = (t.position.y - oy) * inv;
            let began = t.phase == TouchPhase::Started;
            match t.phase {
                TouchPhase::Started | TouchPhase::Moved | TouchPhase::Stationary => {
                    if vy > VH * 0.55 {
                        if vx < VW * 0.5 {
                            t_left = true;
                        } else {
                            t_right = true;
                        }
                    } else {
                        t_jump = true;
                        if began {
                            tap_now = true;
                        }
                    }
                }
                _ => {}
            }
        }

        self.left_held = kb_left || t_left;
        self.right_held = kb_right || t_right;
        let jump = kb_jump || t_jump;
        let restart = kb_restart;

        self.jump_just_pressed = jump && !self.prev_jump;
        self.restart_just_pressed = restart && !self.prev_restart;
        self.any_tap = tap_now || cur_ids.len() > self.prev_taps.len();
        self.prev_jump = jump;
        self.prev_restart = restart;
        self.prev_taps = cur_ids;
    }
}
