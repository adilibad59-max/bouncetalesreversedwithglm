//! Toplanabilir küreler, dikenler ve bitiş halkası.

use macroquad::prelude::*;

use crate::ball::RADIUS;
use crate::level::Level;

#[derive(Clone, Copy, PartialEq)]
pub enum Outcome {
    None,
    Died,
    Goal,
}

#[derive(Default)]
pub struct Entities {
    pub orbs: Vec<(Vec2, bool)>,   // konum, alındı mı
    pub spikes: Vec<Vec2>,
    pub goal: Vec2,
    pub goal_spin: f32,
    pub bob: f32,
}

impl Entities {
    pub fn from_level(level: &Level) -> Self {
        Self {
            orbs: level.orbs.iter().map(|p| (*p, false)).collect(),
            spikes: level.spikes.clone(),
            goal: level.goal,
            goal_spin: 0.0,
            bob: 0.0,
        }
    }

    pub fn collected(&self) -> usize {
        self.orbs.iter().filter(|(_, got)| *got).count()
    }

    pub fn total(&self) -> usize {
        self.orbs.len()
    }

    pub fn update(&mut self, dt: f32, ball_pos: Vec2) -> Outcome {
        self.goal_spin += dt * 2.4;
        self.bob += dt * 3.0;

        // küre toplama
        for (p, got) in self.orbs.iter_mut() {
            if !*got && p.distance(ball_pos) < RADIUS + 6.0 {
                *got = true;
            }
        }
        // diken teması
        for s in &self.spikes {
            if s.distance(ball_pos) < RADIUS + 5.0 {
                return Outcome::Died;
            }
        }
        // bitiş
        if self.goal.distance(ball_pos) < RADIUS + 12.0 {
            return Outcome::Goal;
        }
        Outcome::None
    }
}
