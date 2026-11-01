//! Top fiziği — Bounce Tales mekaniklerinin özgün Rust tasarımı.
//!
//! Analizden çıkan temel davranışlar:
//!   * Top, arazi eğrilerine (vektör segmentler) teğet hareket eder;
//!     yerçekiminin teğet bileşeni ivmeyi üretir → "momentum" hissi.
//!   * Temas anında hız, normal + teğet bileşenlere ayrıştırılır:
//!     normal bileşen itilir (sert düşüşte küçük geri sekme), teğet bileşen
//!     yuvarlanmaya dönüşür (ω = v_t / r — kaymaksızın yuvarlanma).
//!   * Zıplama, yüzey normali yönünde ittirilir; basılı yön tuşu atışı büker.
//!   * Havada sınırlı kontrol, düşüşte tam yerçekimi.

use macroquad::prelude::*;

use crate::level::{Level, Seg};

pub const RADIUS: f32 = 11.0;
pub const GRAVITY: f32 = 1500.0;
pub const GROUND_ACCEL: f32 = 900.0;
pub const AIR_ACCEL: f32 = 380.0;
pub const MAX_SPEED: f32 = 340.0;
pub const JUMP_SPEED: f32 = 430.0;
pub const ROLL_FRICTION: f32 = 0.35;   // 1/s
pub const BOUNCE_THRESHOLD: f32 = 260.0;
pub const RESTITUTION: f32 = 0.22;
const ANG_SMOOTH: f32 = 12.0;

pub struct Ball {
    pub pos: Vec2,
    pub vel: Vec2,
    /// toplam dönme açısı (radyan) — çizimde desen döner
    pub angle: f32,
    pub grounded: bool,
    pub ground_normal: Vec2,
    /// son kare temas edilen segmentteki teğet hız (yuvarlanma yönü için)
    last_tangential: f32,
    jump_buffered: f32,
    coyote: f32,
}

impl Ball {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::ZERO,
            angle: 0.0,
            grounded: false,
            ground_normal: Vec2::new(0.0, -1.0),
            last_tangential: 0.0,
            jump_buffered: 0.0,
            coyote: 0.0,
        }
    }

    pub fn try_jump(&mut self) {
        self.jump_buffered = 0.12; // 120 ms tampon
    }

    /// `left`/`right`: basılı yön tuşları. `dt` saniye.
    pub fn update(&mut self, dt: f32, _any_dir: bool, left: bool, right: bool, level: &Level) {
        self.jump_buffered = (self.jump_buffered - dt).max(0.0);
        self.coyote = (self.coyote - dt).max(0.0);
        let was_grounded = self.grounded;
        if was_grounded {
            self.coyote = 0.09; // 90 ms coyote time
        }

        // 1) yerçekimi
        self.vel.y += GRAVITY * dt;

        // 2) kontrol ivmesi
        let dir = (right as i32 - left as i32) as f32;
        if dir != 0.0 {
            if self.grounded {
                // yüzey teğeti boyunca ivme
                let t = tangent(self.ground_normal);
                let a = t * dir * GROUND_ACCEL;
                self.vel += a * dt;
            } else {
                self.vel.x += dir * AIR_ACCEL * dt;
            }
        } else if self.grounded {
            // yuvarlanma sürtünmesi (teğet bileşeni yavaşça söndür)
            let t = tangent(self.ground_normal);
            let vt = self.vel.dot(t);
            let damped = vt * (-ROLL_FRICTION * dt).exp();
            self.vel += t * (damped - vt);
        }

        // 3) hız sınırı
        let sp = self.vel.length();
        if sp > MAX_SPEED {
            self.vel *= MAX_SPEED / sp;
        }

        // 4) konum entegrasyonu
        self.pos += self.vel * dt;

        // 5) çarpışma çözümü (birkaç yineleme: köşe durumları için)
        self.grounded = false;
        for _ in 0..3 {
            if self.resolve_collisions(level) {
                break;
            }
        }

        // 6) zıplama (tampon + coyote)
        if self.jump_buffered > 0.0 && (self.grounded || self.coyote > 0.0) {
            self.jump_buffered = 0.0;
            self.coyote = 0.0;
            let mut n = self.ground_normal;
            if !self.grounded {
                n = Vec2::new(0.0, -1.0);
            }
            // atış yönünü hafifçe bük (basılı yön varsa)
            let bend = dir * 0.30;
            let mut j = Vec2::new(n.x + bend, n.y);
            j = j.normalize_or_zero() * JUMP_SPEED;
            // mevcut hızı koru + normal atışı ekle
            self.vel += j;
            self.grounded = false;
        }

        // 7) yuvarlanma açısı: ω = v_t / r
        let t = if self.grounded {
            tangent(self.ground_normal)
        } else {
            Vec2::new(1.0, 0.0)
        };
        let vt = if self.grounded {
            self.vel.dot(t)
        } else {
            self.vel.x
        };
        self.last_tangential += (vt - self.last_tangential) * (ANG_SMOOTH * dt).min(1.0);
        self.angle += (self.last_tangential / RADIUS) * dt;

        // 8) düşme sınırları / ölüm çizgisi
        if self.pos.y > level.death_y {
            self.pos = level.spawn;
            self.vel = Vec2::ZERO;
        }
    }

    /// Segmentlerle teması çözer; temas olduysa true döner.
    fn resolve_collisions(&mut self, level: &Level) -> bool {
        let mut touched = false;
        let segs: Vec<&Seg> = level
            .segments_near(self.pos, RADIUS * 2.0)
            .collect();
        for seg in segs {
            // segment üzerindeki en yakın nokta
            let ab = seg.b - seg.a;
            let len2 = ab.length_squared();
            let t = if len2 <= f32::EPSILON {
                0.0
            } else {
                ((self.pos - seg.a).dot(ab) / len2).clamp(0.0, 1.0)
            };
            let closest = seg.a + ab * t;
            let off = self.pos - closest;
            let dist = off.length();
            if dist >= RADIUS || dist <= f32::EPSILON {
                continue;
            }
            // yüzey normali: topa bakan taraf
            let mut n = off / dist;
            if n.dot(Vec2::new(0.0, -1.0)) < -0.9 && seg.both_sides {
                n = -n; // alttan gelen temas
            }
            // konumu dışarı it
            self.pos = closest + n * RADIUS;
            // hız ayrıştırma
            let vn = self.vel.dot(n);
            if vn < 0.0 {
                let t = Vec2::new(-n.y, n.x);
                let mut vt = self.vel.dot(t);
                // sert çarpma → küçük sekme
                if vn < -BOUNCE_THRESHOLD {
                    vt *= 0.92;
                    self.vel = t * vt + n * (-vn * RESTITUTION);
                } else {
                    self.vel = t * vt;
                }
            }
            if n.y < -0.35 {
                self.grounded = true;
                self.ground_normal = n;
            }
            touched = true;
        }
        touched
    }
}

#[inline]
fn tangent(n: Vec2) -> Vec2 {
    Vec2::new(-n.y, n.x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_level() -> Level {
        // y=200'de düz zemin
        Level::demo(0) // yerine gerçek düz zemin aşağıda inşa ediliyor
    }

    fn make_flat() -> Level {
        let chain = vec![
            Vec2::new(-100.0, 200.0),
            Vec2::new(0.0, 200.0),
            Vec2::new(1000.0, 200.0),
        ];
        Level::build_for_test("test", 1000.0, 320.0, Vec2::new(50.0, 100.0), 600.0, chain)
    }

    /// Düşen top zeminde durmalı ve zemine gömülmemeli.
    #[test]
    fn ball_settles_on_flat_ground() {
        let lv = make_flat();
        let mut b = Ball::new(Vec2::new(500.0, 120.0));
        let a = flat_level();
        let _ = a;
        for _ in 0..400 {
            b.update(1.0 / 60.0, false, false, false, &lv);
        }
        assert!(b.grounded, "top yerde durmali");
        assert!((b.pos.y - (200.0 - RADIUS)).abs() < 0.5,
            "top zeminde durmali: y={} beklenen={}", b.pos.y, 200.0 - RADIUS);
        assert!(b.vel.length() < 5.0, "hiz sönümlenmeli: {}", b.vel.length());
    }

    /// Sağa yuvarlanan topun açısı artmalı (saat yönü = ekran koordinatında +).
    #[test]
    fn rolling_right_increases_angle() {
        let lv = make_flat();
        let mut b = Ball::new(Vec2::new(500.0, 120.0));
        let a0 = 0.0f32;
        let _ = a0;
        // önce yerleşsin
        for _ in 0..200 {
            b.update(1.0 / 60.0, true, false, true, &lv);
        }
        let angle_before = b.angle;
        let x_before = b.pos.x;
        for _ in 0..120 {
            b.update(1.0 / 60.0, true, false, true, &lv);
        }
        assert!(b.pos.x > x_before + 30.0, "top ilerlemeli");
        assert!(b.angle > angle_before + 0.5, "top donmeli: Δ={}", b.angle - angle_before);
    }

    /// Zıplama topu yukarı taşımali.
    #[test]
    fn jump_raises_ball() {
        let lv = make_flat();
        let mut b = Ball::new(Vec2::new(500.0, 120.0));
        for _ in 0..200 {
            b.update(1.0 / 60.0, false, false, false, &lv);
        }
        let y0 = b.pos.y;
        b.try_jump();
        let mut peak = y0;
        for _ in 0..60 {
            b.update(1.0 / 60.0, false, false, false, &lv);
            peak = peak.min(b.pos.y);
        }
        assert!(peak < y0 - 40.0, "ziplama yukseltmeli: Δ={}", y0 - peak);
    }
}
