use std::{ops::{Mul, Add}, fmt::Display, collections::{HashMap, HashSet}};

use rand::Rng;

use crate::view;

pub struct Galaxy {
    pub stardate: f32,
    pub final_stardate: f32,
    pub quadrants: Vec<Quadrant>,
    pub scanned: HashSet<Pos>,
    pub enterprise: Enterprise
}

pub struct Quadrant {
    pub stars: Vec<Pos>,
    pub star_base: Option<StarBase>,
    pub klingons: Vec<Klingon>
}

pub struct StarBase {
    pub sector: Pos,
    pub repair_delay: f32,
}

pub struct Klingon {
    pub sector: Pos,
    pub energy: f32
}

impl Klingon {
    pub fn fire_on(&mut self, enterprise: &mut Enterprise) {
        let mut rng = rand::thread_rng();
        let attack_strength = rng.gen::<f32>();
        let dist_to_enterprise = self.sector.abs_diff(enterprise.sector) as f32;
        let hit_strength = self.energy * (2.0 + attack_strength) / dist_to_enterprise;
        
        self.energy /= 3.0 + attack_strength;

        enterprise.take_hit(self.sector, hit_strength as u16);
    }
}

pub struct Enterprise {
    pub destroyed: bool,
    pub damaged: HashMap<String, f32>,
    pub quadrant: Pos,
    pub sector: Pos,
    pub photon_torpedoes: u8,
    pub total_energy: u16,
    pub shields: u16,
}
impl Enterprise {
    fn take_hit(&mut self, sector: Pos, hit_strength: u16) {
        if self.destroyed {
            return;
        }
        
        view::enterprise_hit(&hit_strength, sector);

        if self.shields <= hit_strength {
            view::enterprise_destroyed();
            self.destroyed = true;
            return;
        }

        self.shields -= hit_strength;

        view::shields_hit(self.shields);
        
        if hit_strength >= 20 {
            self.take_damage(hit_strength)
        }
    }

    fn take_damage(&mut self, hit_strength: u16) {
        let mut rng = rand::thread_rng();

        let hit_past_shield = hit_strength as f32 / self.shields as f32;
        if rng.gen::<f32>() > 0.6 || hit_past_shield < 0.02 {
            return
        }

        let system = systems::KEYS[rng.gen_range(0..systems::KEYS.len())].to_string();
        let damage = hit_past_shield + rng.gen::<f32>() * 0.5;
        self.damage_system(&system, damage);
    }

    pub fn damage_system(&mut self, system: &str, damage: f32) {
        self.damaged.entry(system.to_string()).and_modify(|d| *d -= damage).or_insert(-damage);
    }

    pub fn repair_system(&mut self, system: &str, amount: f32) -> bool {
        let existing_damage = self.damaged[system];
        if existing_damage + amount >= -0.1 {
            self.damaged.remove(system);
            return true;
        }
    
        self.damaged.entry(system.to_string()).and_modify(|d| *d += amount);
        return false;
    }

    pub fn is_stranded(&self) -> bool {
        if self.total_energy < 10 || (self.shields + 10 > self.total_energy && self.damaged.contains_key(systems::SHIELD_CONTROL)) {
            view::stranded();
            return true;
        }
        return false;
    }
}

pub mod systems {

    pub const SHORT_RANGE_SCAN: &str = "SRS";
    pub const WARP_ENGINES: &str = "NAV";
    pub const SHIELD_CONTROL: &str = "SHE";
    pub const DAMAGE_CONTROL: &str = "DAM";
    pub const LONG_RANGE_SCAN: &str = "LRS";
    pub const COMPUTER: &str = "COM";
    pub const PHASERS: &str = "PHA";
    pub const TORPEDOES: &str = "TOR";

    pub const RESIGN: &str = "XXX";

    pub const KEYS: [&str; 8] = [
        SHORT_RANGE_SCAN, WARP_ENGINES, SHIELD_CONTROL, DAMAGE_CONTROL, LONG_RANGE_SCAN, COMPUTER, PHASERS, TORPEDOES
    ];

    pub fn name_for(key: &str) -> String {
        match key {
            SHORT_RANGE_SCAN => "Short Range Scanners".into(),
            WARP_ENGINES => "Warp Engines".into(),
            SHIELD_CONTROL => "Shield Control".into(),
            DAMAGE_CONTROL => "Damage Control".into(),
            LONG_RANGE_SCAN => "Long Range Scanners".into(),
            COMPUTER => "Library-Computer".into(),
            PHASERS => "Phaser Control".into(),
            TORPEDOES => "Photon Tubes".into(),
            _ => "Unknown".into()
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Hash, Eq)]
pub struct Pos(pub u8, pub u8);

impl Pos {
    pub fn as_index(&self) -> usize {
        (self.0 * 8 + self.1).into()
    }

    pub fn abs_diff(&self, other: Pos) -> u8 {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
    }

    pub fn dist(&self, other: Pos) -> u8 {
        let dx = other.0 as f32 - self.0 as f32;
        let dy = other.1 as f32 - self.1 as f32;
        (f32::powi(dx, 2) + f32::powi(dy, 2)).sqrt() as u8
    }

    pub fn direction(&self, other: Pos) -> f32 {
        // this is a replication of the original BASIC code
        let dx = other.0 as f32 - self.0 as f32;
        let dy = other.1 as f32 - self.1 as f32;
        let dx_dominant = dx.abs() > dy.abs();

        let frac = if dx_dominant { dy / dx } else { -dx / dy };
        let nearest_cardinal = 
            if dx_dominant {
                if dx > 0. { 7. } else { 3. }
            } else {
                if dy > 0. { 1. } else { 5. }
            };
        
        let mut dir = nearest_cardinal + frac;
        if dir < 1. {
            dir += 8.
        }
        dir
    }

    pub fn as_galactic_sector(&self, containing_quadrant: Pos) -> Self {
        Pos(containing_quadrant.0 * 8 + self.0, containing_quadrant.1 * 8 + self.1)
    }

    pub fn to_local_quadrant_sector(&self) -> (Self, Self) {
        (Pos(self.0 / 8, self.1 / 8), Pos(self.0 % 8, self.1 % 8))
    }
}

impl Mul<u8> for Pos {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self::Output {
        Pos(self.0 * rhs, self.1 * rhs)
    }
}

impl Add<Pos> for Pos {
    type Output = Self;

    fn add(self, rhs: Pos) -> Self::Output {
        Pos(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} , {}", self.0 + 1, self.1 + 1)
    }
}

pub const COURSES : [(f32, f32); 9] = [
    (0., 1.),
    (-1., 1.),
    (-1., 0.),
    (-1., -1.),
    (0., -1.),
    (1., -1.),
    (1., 0.),
    (1., 1.),
    (0., 1.), // course 9 is equal to course 1
];

#[derive(PartialEq)]
pub enum SectorStatus {
    Empty, Star, StarBase, Klingon
}

pub const MAX_PHOTON_TORPEDOES: u8 = 28;
pub const MAX_ENERGY: u16 = 3000;

impl Galaxy {
    pub fn remaining_klingons(&self) -> u8 {
        let quadrants = &self.quadrants;
        quadrants.into_iter().map(|q| { q.klingons.len() as u8 }).sum::<u8>()
    }

    pub fn remaining_starbases(&self) -> u8 {
        let quadrants = &self.quadrants;
        quadrants.into_iter().filter(|q| q.star_base.is_some()).count() as u8
    }

    pub fn generate_new() -> Self {
        let quadrants = Self::generate_quadrants();

        let mut rng = rand::thread_rng();
        let stardate = rng.gen_range(20..=40) as f32 * 100.0;

        let enterprise = Self::new_captain(&quadrants);

        let mut scanned = HashSet::new();
        scanned.insert(enterprise.quadrant);

        Galaxy { 
            stardate,
            final_stardate: stardate + rng.gen_range(25..=35) as f32,
            quadrants, 
            scanned,
            enterprise
        }
    } 

    pub fn new_captain(quadrants: &Vec<Quadrant>) -> Enterprise {
        let mut rng = rand::thread_rng();
        let enterprise_quadrant = Pos(rng.gen_range(0..8), rng.gen_range(0..8));
        let enterprise_sector = quadrants[enterprise_quadrant.as_index()].find_empty_sector();
        Enterprise { 
            destroyed: false,
            damaged: HashMap::new(),
            quadrant: enterprise_quadrant, 
            sector: enterprise_sector,
            photon_torpedoes: MAX_PHOTON_TORPEDOES,
            total_energy: MAX_ENERGY,
            shields: 0 }
    }   

    fn generate_quadrants() -> Vec<Quadrant> {
        let mut rng = rand::thread_rng();
        let mut result = Vec::new();
        for _ in 0..64 {

            let mut quadrant = Quadrant { stars: Vec::new(), star_base: None, klingons: Vec::new() };
            let star_count = rng.gen_range(0..=7);
            for _ in 0..star_count {
                quadrant.stars.push(quadrant.find_empty_sector());
            }

            if rng.gen::<f64>() > 0.96 {
                quadrant.star_base = Some(StarBase { sector: quadrant.find_empty_sector(), repair_delay: rng.gen::<f32>() * 0.5 });
            }

            let klingon_count = 
                match rng.gen::<f64>() {
                    n if n > 0.98 => 3,
                    n if n > 0.95 => 2,
                    n if n > 0.8 => 1,
                    _ => 0
                };
                for _ in 0..klingon_count {
                    quadrant.klingons.push(Klingon { sector: quadrant.find_empty_sector(), energy: rng.gen_range(100..=300) as f32 });
                }

            result.push(quadrant);
        }
        result
    }
}

impl Quadrant {
    pub fn sector_status(&self, sector: Pos) -> SectorStatus {
        if self.stars.contains(&sector) {
            SectorStatus::Star
        } else if self.is_starbase(sector) {
            SectorStatus::StarBase
        } else if self.has_klingon(sector) {
            SectorStatus::Klingon
        } else {
            SectorStatus::Empty
        }
    }

    fn is_starbase(&self, sector: Pos) -> bool {
        match &self.star_base {
            None => false,
            Some(p) => p.sector == sector
        }
    }

    fn has_klingon(&self, sector: Pos) -> bool {
        let klingons = &self.klingons;
        klingons.into_iter().find(|k| k.sector == sector).is_some()
    }

    pub fn get_klingon(&mut self, sector: Pos) -> Option<&mut Klingon> {
        let klingons = &mut self.klingons;
        klingons.into_iter().find(|k| k.sector == sector)
    }

    pub fn find_empty_sector(&self) -> Pos {
        let mut rng = rand::thread_rng();
        loop {
            let pos = Pos(rng.gen_range(0..8), rng.gen_range(0..8));
            if self.sector_status(pos) == SectorStatus::Empty {
                return pos
            }
        }
    }

    pub fn docked_at_starbase(&self, enterprise_sector: Pos) -> bool {
        self.star_base.is_some() && self.star_base.as_ref().unwrap().sector.abs_diff(enterprise_sector) == 1
    }
}
