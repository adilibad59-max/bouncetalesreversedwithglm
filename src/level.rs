//! Seviye formatı ve demo seviyeler.
//!
//! Oyunun kendi (özgün) metin tabanlı formatı; orijinal J2ME RLEF ikili
//! formatından bağımsız, insan tarafından okunabilir ve elle düzenlenebilir:
//!
//! ```text
//! name   Yeşil Tepeler        # seviye adı
//! size   2400 320             # dünya boyutu (piksel)
//! spawn  60 80                # başlangıç noktası
//! death  600                  # bu y'nin altına düşen top yeniden başlar
//! ground 0,320 200,240 500,180 1200,300 2400,320   # katı arazi折 hattı (polyline)
//! orb    300 140              # toplanabilir küre (x y)
//! spike  700 260              # tehlike (diken) (x y)
//! goal   2300 250             # bitiş halkası (x y)
//! ```
//!
//! `ground` satırları birleştirilerek sürekli bir arazi zinciri oluşturulur;
//! her ardışık nokta çifti bir çarpışma segmentidir (vektör tabanlı seviyeler,
//! orijinal oyunun tasarımına uygun).

use macroquad::prelude::*;

#[derive(Clone)]
pub struct Seg {
    pub a: Vec2,
    pub b: Vec2,
    pub both_sides: bool,
}

pub struct Level {
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub spawn: Vec2,
    pub death_y: f32,
    pub chain: Vec<Vec2>,
    pub segments: Vec<Seg>,
    pub orbs: Vec<Vec2>,
    pub spikes: Vec<Vec2>,
    pub goal: Vec2,
    /// hızlı yakınlık sorgusu için kaba ızgara (hücre 120 px)
    cell: f32,
    grid: Vec<Vec<usize>>,
    gw: usize,
    gh: usize,
}

impl Level {
    /// Test amaçlı seviye kurucu (zemin + boş varlık listeleri).
    #[cfg(test)]
    pub fn build_for_test(name: &str, width: f32, height: f32, spawn: Vec2,
                          death_y: f32, chain: Vec<Vec2>) -> Self {
        Self::build(name, width, height, spawn, death_y, chain, Vec::new(), Vec::new(),
                    Vec2::new(width - 60.0, 100.0))
    }

    fn build(name: &str, width: f32, height: f32, spawn: Vec2, death_y: f32,
             chain: Vec<Vec2>, orbs: Vec<Vec2>, spikes: Vec<Vec2>, goal: Vec2) -> Self {
        let segments = chain
            .windows(2)
            .map(|w| Seg { a: w[0], b: w[1], both_sides: false })
            .collect();
        let mut lv = Self {
            name: name.into(),
            width,
            height,
            spawn,
            death_y,
            chain,
            segments,
            orbs,
            spikes,
            goal,
            cell: 120.0,
            grid: Vec::new(),
            gw: 0,
            gh: 0,
        };
        lv.build_grid();
        lv
    }

    fn build_grid(&mut self) {
        self.gw = ((self.width / self.cell).ceil() as usize).max(1);
        self.gh = ((self.height / self.cell).ceil() as usize).max(1);
        self.grid = vec![Vec::new(); self.gw * self.gh];
        for (i, s) in self.segments.iter().enumerate() {
            let x0 = ((s.a.x.min(s.b.x) / self.cell).floor() as usize).min(self.gw - 1);
            let x1 = ((s.a.x.max(s.b.x) / self.cell).ceil() as usize).min(self.gw - 1);
            let y0 = ((s.a.y.min(s.b.y) / self.cell).floor() as usize).min(self.gh - 1);
            let y1 = ((s.a.y.max(s.b.y) / self.cell).ceil() as usize).min(self.gh - 1);
            for gy in y0..=y1 {
                for gx in x0..=x1 {
                    self.grid[gy * self.gw + gx].push(i);
                }
            }
        }
    }

    /// `pos` çevresindeki (yarıçap eklenecek) segmentler.
    pub fn segments_near(&self, pos: Vec2, pad: f32) -> impl Iterator<Item = &Seg> {
        let x0 = (((pos.x - pad) / self.cell).floor() as usize).min(self.gw - 1);
        let x1 = (((pos.x + pad) / self.cell).ceil() as usize).min(self.gw - 1);
        let y0 = (((pos.y - pad) / self.cell).floor() as usize).min(self.gh - 1);
        let y1 = (((pos.y + pad) / self.cell).ceil() as usize).min(self.gh - 1);
        let mut seen = [false; 4096];
        let idxs: Vec<usize> = (y0..=y1)
            .flat_map(|gy| (x0..=x1).map(move |gx| gy * self.gw + gx))
            .flat_map(|c| self.grid[c].iter().copied())
            .filter(|i| {
                if *i < 4096 {
                    if seen[*i] { false } else { seen[*i] = true; true }
                } else {
                    true
                }
            })
            .collect();
        idxs.into_iter().map(move |i| &self.segments[i])
    }

    pub fn demo_count() -> usize {
        3
    }

    pub fn demo(idx: usize) -> Self {
        match idx {
            0 => Self::level_green_hills(),
            1 => Self::level_cave(),
            2 => Self::level_sky(),
            _ => Self::level_green_hills(),
        }
    }

    // --- Seviye 1: yuvarlanma ve ilk küreler ---------------------------------
    fn level_green_hills() -> Self {
        let chain = vec![
            Vec2::new(-40.0, 280.0),
            Vec2::new(160.0, 280.0),
            Vec2::new(300.0, 190.0),   // tepe
            Vec2::new(460.0, 285.0),
            Vec2::new(620.0, 285.0),
            Vec2::new(760.0, 200.0),
            Vec2::new(900.0, 285.0),
            Vec2::new(1080.0, 285.0),
            Vec2::new(1220.0, 175.0),  // yüksek tepe → rampa
            Vec2::new(1380.0, 285.0),
            Vec2::new(1600.0, 285.0),
            Vec2::new(1740.0, 210.0),
            Vec2::new(1880.0, 285.0),
            Vec2::new(2100.0, 285.0),
            Vec2::new(2260.0, 240.0),
            Vec2::new(2440.0, 300.0),
        ];
        let orbs = vec![
            Vec2::new(300.0, 140.0),
            Vec2::new(480.0, 235.0),
            Vec2::new(640.0, 240.0),
            Vec2::new(760.0, 150.0),
            Vec2::new(940.0, 240.0),
            Vec2::new(1100.0, 240.0),
            Vec2::new(1230.0, 120.0),
            Vec2::new(1420.0, 240.0),
            Vec2::new(1750.0, 160.0),
            Vec2::new(1900.0, 240.0),
            Vec2::new(2280.0, 190.0),
        ];
        Self::build(
            "1. Yesil Tepeler",
            2400.0, 320.0,
            Vec2::new(50.0, 220.0),
            600.0,
            chain,
            orbs,
            vec![Vec2::new(1010.0, 262.0), Vec2::new(1560.0, 262.0)],
            Vec2::new(2340.0, 180.0),
        )
    }

    // --- Seviye 2: çukurlar ve keskin inişler -------------------------------
    fn level_cave() -> Self {
        let chain = vec![
            Vec2::new(-40.0, 240.0),
            Vec2::new(140.0, 240.0),
            Vec2::new(240.0, 300.0),   // çukur
            Vec2::new(360.0, 210.0),   // yokuş yukarı
            Vec2::new(520.0, 300.0),
            Vec2::new(700.0, 300.0),
            Vec2::new(840.0, 160.0),   // yüksek rampa
            Vec2::new(1040.0, 300.0),  // iniş — hız biriktir
            Vec2::new(1220.0, 300.0),
            Vec2::new(1330.0, 190.0),
            Vec2::new(1470.0, 285.0),
            Vec2::new(1620.0, 285.0),
            Vec2::new(1760.0, 255.0),
            Vec2::new(1900.0, 285.0),
            Vec2::new(2100.0, 285.0),
            Vec2::new(2280.0, 160.0),
            Vec2::new(2440.0, 250.0),
        ];
        let orbs = vec![
            Vec2::new(300.0, 255.0),
            Vec2::new(360.0, 155.0),
            Vec2::new(560.0, 255.0),
            Vec2::new(850.0, 105.0),
            Vec2::new(1050.0, 255.0),
            Vec2::new(1340.0, 140.0),
            Vec2::new(1490.0, 235.0),
            Vec2::new(1790.0, 205.0),
            Vec2::new(2300.0, 105.0),
        ];
        Self::build(
            "2. Magara Kosusu",
            2400.0, 320.0,
            Vec2::new(50.0, 180.0),
            700.0,
            chain,
            orbs,
            vec![
                Vec2::new(660.0, 277.0),
                Vec2::new(1180.0, 277.0),
                Vec2::new(1650.0, 262.0),
                Vec2::new(2020.0, 277.0),
            ],
            Vec2::new(2360.0, 195.0),
        )
    }

    // --- Seviye 3: uzun yaylar ve son tırmanış ------------------------------
    fn level_sky() -> Self {
        let chain = vec![
            Vec2::new(-40.0, 260.0),
            Vec2::new(120.0, 260.0),
            Vec2::new(280.0, 180.0),
            Vec2::new(440.0, 260.0),
            Vec2::new(600.0, 200.0),
            Vec2::new(760.0, 280.0),
            Vec2::new(980.0, 300.0),
            Vec2::new(1150.0, 140.0),  // büyük rampa
            Vec2::new(1330.0, 285.0),
            Vec2::new(1500.0, 285.0),
            Vec2::new(1650.0, 190.0),
            Vec2::new(1800.0, 285.0),
            Vec2::new(1980.0, 285.0),
            Vec2::new(2140.0, 130.0),  // son tırmanış
            Vec2::new(2300.0, 285.0),
            Vec2::new(2440.0, 285.0),
        ];
        let orbs = vec![
            Vec2::new(280.0, 130.0),
            Vec2::new(450.0, 215.0),
            Vec2::new(610.0, 155.0),
            Vec2::new(900.0, 255.0),
            Vec2::new(1160.0, 85.0),
            Vec2::new(1350.0, 235.0),
            Vec2::new(1660.0, 135.0),
            Vec2::new(1900.0, 235.0),
            Vec2::new(2150.0, 75.0),
        ];
        Self::build(
            "3. Gokyuzu Tirmanisi",
            2400.0, 320.0,
            Vec2::new(50.0, 200.0),
            640.0,
            chain,
            orbs,
            vec![
                Vec2::new(940.0, 277.0),
                Vec2::new(1420.0, 262.0),
                Vec2::new(2040.0, 262.0),
            ],
            Vec2::new(2370.0, 215.0),
        )
    }
}

/// Seviye dosyasını (insan-okur metin formatı) ayrıştırır.
/// Yerleşik seviyeler kod içinde tanımlıdır; bu işlev modülerliktir.
pub fn parse_level(src: &str) -> Result<Level, String> {
    let mut name = String::from("Seviye");
    let (mut w, mut h) = (2400.0, 320.0);
    let mut spawn = Vec2::new(50.0, 100.0);
    let mut death = 600.0;
    let mut chain: Vec<Vec2> = Vec::new();
    let mut orbs = Vec::new();
    let mut spikes = Vec::new();
    let mut goal = Vec2::new(2300.0, 200.0);

    for (ln, raw) in src.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let tok: Vec<&str> = line.split_whitespace().collect();
        let num = |i: usize| -> Result<f32, String> {
            tok.get(i)
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| format!("satir {}: sayi bekleniyordu", ln + 1))
        };
        let pts = |start: usize| -> Result<Vec<Vec2>, String> {
            let v: Result<Vec<f32>, _> = tok[start..].iter().map(|s| s.parse::<f32>()).collect();
            let v = v.map_err(|_| format!("satir {}: koordinat hatasi", ln + 1))?;
            if v.len() % 2 != 0 {
                return Err(format!("satir {}: nokta cifti bozuk", ln + 1));
            }
            Ok(v.chunks(2).map(|c| Vec2::new(c[0], c[1])).collect())
        };
        match tok[0] {
            "name" => name = tok[1..].join(" "),
            "size" => { w = num(1)?; h = num(2)?; }
            "spawn" => { spawn = Vec2::new(num(1)?, num(2)?); }
            "death" => death = num(1)?,
            "ground" => chain = pts(1)?,
            "orb" => orbs.push(Vec2::new(num(1)?, num(2)?)),
            "spike" => spikes.push(Vec2::new(num(1)?, num(2)?)),
            "goal" => goal = Vec2::new(num(1)?, num(2)?),
            other => return Err(format!("satir {}: bilinmeyen komut '{}'", ln + 1, other)),
        }
    }
    if chain.len() < 2 {
        return Err("en az 2 ground noktasi gerekli".into());
    }
    Ok(Level::build(&name, w, h, spawn, death, chain, orbs, spikes, goal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal() {
        let lv = parse_level("name Test\nsize 800 320\nspawn 10 10\ndeath 500\n\
                             ground 0 300 400 200 800 300\norb 100 100\ngoal 700 250")
            .unwrap();
        assert_eq!(lv.segments.len(), 2);
        assert_eq!(lv.orbs.len(), 1);
        assert_eq!(lv.name, "Test");
    }

    #[test]
    fn parse_bad() {
        assert!(parse_level("spawn 10").is_err());
    }
}
