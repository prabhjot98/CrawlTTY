use crate::*;

pub(crate) fn enter_dungeon(c: &mut Character) -> String {
    if c.act2_completed {
        return "The Glass Wastes are conquered. Rest, trade, or start a new exile.".to_string();
    }
    if c.glass_tyrant_defeated {
        return "The Glass Tyrant is shattered. Return to Warden Mara (t) to complete Act II."
            .to_string();
    }
    if c.act1_completed {
        c.active_dungeon = Some(generate_dungeon(ACT2_START_FLOOR));
        return String::new();
    }
    if c.bellkeeper_defeated {
        return "The Bellkeeper is dead. Return to Warden Mara (t) to complete Act I.".to_string();
    }
    c.active_dungeon = Some(generate_dungeon(1));
    String::new()
}

#[derive(Clone)]
pub(crate) struct Room {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) w: i32,
    pub(crate) h: i32,
}

impl Room {
    pub(crate) fn center(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }

    pub(crate) fn intersects(&self, other: &Room) -> bool {
        self.x <= other.x + other.w + 1
            && self.x + self.w + 1 >= other.x
            && self.y <= other.y + other.h + 1
            && self.y + self.h + 1 >= other.y
    }
}

pub(crate) fn generate_dungeon(floor: u32) -> Dungeon {
    let mut rng = rand::thread_rng();
    let mut tiles = vec!['#'; (MAP_W * MAP_H) as usize];
    let act_floor = act_floor(floor);
    let is_act2 = floor >= ACT2_START_FLOOR;
    let target_rooms = if is_act2 {
        match act_floor {
            1..=3 => rng.gen_range(6..=9),
            4..=7 => rng.gen_range(8..=11),
            _ => rng.gen_range(6..=8),
        }
    } else {
        match floor {
            1 => rng.gen_range(6..=8),
            2..=4 => rng.gen_range(7..=9),
            5..=9 => rng.gen_range(8..=10),
            _ => rng.gen_range(5..=7),
        }
    };
    let mut rooms: Vec<Room> = Vec::new();

    for _ in 0..120 {
        if rooms.len() >= target_rooms {
            break;
        }
        let room = Room {
            w: rng.gen_range(5..=11),
            h: rng.gen_range(4..=7),
            x: rng.gen_range(1..MAP_W - 12),
            y: rng.gen_range(1..MAP_H - 8),
        };
        if rooms.iter().any(|existing| room.intersects(existing)) {
            continue;
        }
        carve_room(&mut tiles, &room);
        if let Some(previous) = rooms.last() {
            carve_corridor(&mut tiles, previous.center(), room.center());
        }
        rooms.push(room);
    }

    if rooms.is_empty() {
        let fallback = Room {
            x: 2,
            y: 2,
            w: 10,
            h: 6,
        };
        carve_room(&mut tiles, &fallback);
        rooms.push(fallback);
    }

    let start = rooms.first().unwrap().center();
    let stairs = farthest_room_center(&rooms, start);
    let mut occupied = vec![start, stairs];
    let mut enemies = Vec::new();
    let enemy_count = if is_act2 {
        match act_floor {
            1..=3 => 7,
            4..=7 => 9,
            _ => 6,
        }
    } else {
        match floor {
            1 => 5,
            2..=4 => 7,
            5..=9 => 8,
            _ => 5,
        }
    };
    for _ in 0..enemy_count {
        let (x, y) = random_room_floor(&rooms, &mut rng, &occupied);
        occupied.push((x, y));
        let e = if is_act2 {
            match act_floor {
                1..=2 => {
                    if rng.gen_bool(0.55) {
                        dune_stalker(x, y)
                    } else {
                        glass_wraith(x, y)
                    }
                }
                3..=5 => {
                    if rng.gen_bool(0.40) {
                        glass_wraith(x, y)
                    } else if rng.gen_bool(0.55) {
                        ember_magus(x, y)
                    } else {
                        obsidian_guard(x, y)
                    }
                }
                _ => {
                    if rng.gen_bool(0.35) {
                        ember_magus(x, y)
                    } else if rng.gen_bool(0.50) {
                        obsidian_guard(x, y)
                    } else {
                        glass_wraith(x, y)
                    }
                }
            }
        } else {
            match floor {
                1 => {
                    if rng.gen_bool(0.55) {
                        rat(x, y)
                    } else {
                        skeleton(x, y)
                    }
                }
                2..=4 => {
                    if rng.gen_bool(0.45) {
                        skeleton(x, y)
                    } else {
                        cultist(x, y)
                    }
                }
                5..=9 => {
                    if rng.gen_bool(0.35) {
                        skeleton(x, y)
                    } else if rng.gen_bool(0.50) {
                        cultist(x, y)
                    } else {
                        boneguard(x, y)
                    }
                }
                _ => {
                    if rng.gen_bool(0.45) {
                        cultist(x, y)
                    } else {
                        boneguard(x, y)
                    }
                }
            }
        };
        enemies.push(scale_enemy_for_floor(e, floor));
    }
    if is_act2 {
        if (2..ACT2_FLOORS).contains(&act_floor) && act_floor % 2 == 1 {
            let mut pos = farthest_room_center(&rooms, start);
            if occupied.contains(&pos) {
                pos = random_room_floor(&rooms, &mut rng, &occupied);
            }
            occupied.push(pos);
            enemies.push(scale_enemy_for_floor(
                elite_glass_wraith(pos.0, pos.1),
                floor,
            ));
        }
        if act_floor == ACT2_FLOORS {
            enemies.push(scale_enemy_for_floor(
                glass_tyrant(stairs.0, stairs.1),
                floor,
            ));
        }
    } else {
        if (2..ACT1_FLOORS).contains(&floor) && floor % 2 == 0 {
            let mut pos = farthest_room_center(&rooms, start);
            if occupied.contains(&pos) {
                pos = random_room_floor(&rooms, &mut rng, &occupied);
            }
            occupied.push(pos);
            enemies.push(scale_enemy_for_floor(elite_skeleton(pos.0, pos.1), floor));
        }
        if floor == ACT1_FLOORS {
            enemies.push(scale_enemy_for_floor(bellkeeper(stairs.0, stairs.1), floor));
        }
    }

    let chest_count = rng.gen_range(1..=3);
    let mut chests = Vec::new();
    for _ in 0..chest_count {
        let (x, y) = random_room_floor(&rooms, &mut rng, &occupied);
        occupied.push((x, y));
        chests.push(Chest {
            x,
            y,
            opened: false,
        });
    }

    Dungeon {
        floor,
        player_x: start.0,
        player_y: start.1,
        stairs_x: stairs.0,
        stairs_y: stairs.1,
        enemies,
        chests,
        log: vec![format!(
            "[INFO] Entered {} floor {}.",
            act_name(floor),
            act_floor
        )],
        tiles,
        bell_wave_tiles: Vec::new(),
        boss_turn_counter: 0,
        log_turn: 0,
    }
}

pub(crate) fn act_floor(floor: u32) -> u32 {
    if floor >= ACT2_START_FLOOR {
        floor - ACT1_FLOORS
    } else {
        floor
    }
}

pub(crate) fn act_name(floor: u32) -> &'static str {
    if floor >= ACT2_START_FLOOR {
        "Glass Wastes"
    } else {
        "Hollow Crypts"
    }
}

pub(crate) fn floor_difficulty_multiplier(floor: u32) -> f32 {
    if floor >= ACT2_START_FLOOR {
        4.5 + (act_floor(floor).saturating_sub(1) as f32
            / ACT2_FLOORS.saturating_sub(1).max(1) as f32)
            * 2.0
    } else {
        floor_reward_multiplier(floor) * 2.0
    }
}

pub(crate) fn floor_reward_multiplier(floor: u32) -> f32 {
    if floor >= ACT2_START_FLOOR {
        2.2 + (act_floor(floor).saturating_sub(1) as f32
            / ACT2_FLOORS.saturating_sub(1).max(1) as f32)
            * 1.3
    } else {
        1.0 + floor.saturating_sub(1) as f32 / ACT1_FLOORS.saturating_sub(1).max(1) as f32
    }
}

pub(crate) fn scale_enemy_for_floor(mut enemy: Enemy, floor: u32) -> Enemy {
    let difficulty_multiplier = floor_difficulty_multiplier(floor);
    let reward_multiplier = floor_reward_multiplier(floor);
    enemy.max_hp = scale_i32(enemy.max_hp, difficulty_multiplier);
    enemy.hp = enemy.max_hp;
    enemy.damage_min = scale_i32(enemy.damage_min, difficulty_multiplier);
    enemy.damage_max = scale_i32(enemy.damage_max, difficulty_multiplier).max(enemy.damage_min);
    enemy.armor += 1 + (floor.saturating_sub(1) / 3) as i32;
    enemy.xp = scale_u32(enemy.xp, reward_multiplier);
    enemy.gold_min = scale_u32(enemy.gold_min, reward_multiplier);
    enemy.gold_max = scale_u32(enemy.gold_max, reward_multiplier).max(enemy.gold_min);
    enemy
}

pub(crate) fn scale_i32(value: i32, multiplier: f32) -> i32 {
    ((value as f32) * multiplier).round().max(1.0) as i32
}

pub(crate) fn scale_u32(value: u32, multiplier: f32) -> u32 {
    if value == 0 {
        0
    } else {
        ((value as f32) * multiplier).round().max(1.0) as u32
    }
}

pub(crate) fn tile_index(x: i32, y: i32) -> usize {
    (y * MAP_W + x) as usize
}

pub(crate) fn carve_room(tiles: &mut [char], room: &Room) {
    for y in room.y..room.y + room.h {
        for x in room.x..room.x + room.w {
            tiles[tile_index(x, y)] = '.';
        }
    }
}

pub(crate) fn carve_corridor(tiles: &mut [char], from: (i32, i32), to: (i32, i32)) {
    let mut x = from.0;
    let mut y = from.1;
    while x != to.0 {
        tiles[tile_index(x, y)] = '.';
        x += (to.0 - x).signum();
    }
    while y != to.1 {
        tiles[tile_index(x, y)] = '.';
        y += (to.1 - y).signum();
    }
    tiles[tile_index(x, y)] = '.';
}

pub(crate) fn farthest_room_center(rooms: &[Room], from: (i32, i32)) -> (i32, i32) {
    rooms
        .iter()
        .map(Room::center)
        .max_by_key(|(x, y)| (x - from.0).abs() + (y - from.1).abs())
        .unwrap_or((MAP_W - 3, MAP_H - 3))
}

pub(crate) fn random_room_floor(
    rooms: &[Room],
    rng: &mut impl Rng,
    occupied: &[(i32, i32)],
) -> (i32, i32) {
    for _ in 0..100 {
        let room = &rooms[rng.gen_range(0..rooms.len())];
        let pos = (
            rng.gen_range(room.x..room.x + room.w),
            rng.gen_range(room.y..room.y + room.h),
        );
        if !occupied.contains(&pos) {
            return pos;
        }
    }
    rooms
        .iter()
        .flat_map(|room| {
            (room.y..room.y + room.h)
                .flat_map(move |y| (room.x..room.x + room.w).map(move |x| (x, y)))
        })
        .find(|pos| !occupied.contains(pos))
        .unwrap_or_else(|| rooms.last().unwrap().center())
}

pub(crate) fn dungeon_tile(d: &Dungeon, x: i32, y: i32) -> char {
    if x < 0 || y < 0 || x >= MAP_W || y >= MAP_H {
        return '#';
    }
    if d.tiles.len() == (MAP_W * MAP_H) as usize {
        d.tiles[tile_index(x, y)]
    } else if x == 0 || y == 0 || x == MAP_W - 1 || y == MAP_H - 1 {
        '#'
    } else {
        '.'
    }
}

#[derive(Clone, Copy)]
pub(crate) struct EnemyStats {
    pub(crate) hp: i32,
    pub(crate) damage_min: i32,
    pub(crate) damage_max: i32,
    pub(crate) armor: i32,
    pub(crate) speed: i32,
}

#[derive(Clone, Copy)]
pub(crate) struct EnemyRewards {
    pub(crate) xp: u32,
    pub(crate) gold_min: u32,
    pub(crate) gold_max: u32,
}

pub(crate) fn enemy_stats(
    hp: i32,
    damage_min: i32,
    damage_max: i32,
    armor: i32,
    speed: i32,
) -> EnemyStats {
    EnemyStats {
        hp,
        damage_min,
        damage_max,
        armor,
        speed,
    }
}

pub(crate) fn enemy_rewards(xp: u32, gold_min: u32, gold_max: u32) -> EnemyRewards {
    EnemyRewards {
        xp,
        gold_min,
        gold_max,
    }
}

pub(crate) fn enemy(
    name: &str,
    glyph: char,
    x: i32,
    y: i32,
    stats: EnemyStats,
    rewards: EnemyRewards,
    is_boss: bool,
) -> Enemy {
    Enemy {
        name: name.to_string(),
        glyph,
        x,
        y,
        hp: stats.hp,
        max_hp: stats.hp,
        damage_min: stats.damage_min,
        damage_max: stats.damage_max,
        armor: stats.armor,
        speed: stats.speed,
        energy: 10,
        xp: rewards.xp,
        gold_min: rewards.gold_min,
        gold_max: rewards.gold_max,
        is_boss,
        stunned_turns: 0,
        bleed_turns: 0,
        bleed_damage: 0,
        armor_shred_turns: 0,
        vulnerable_turns: 0,
        guarding: false,
        elite_modifier: None,
    }
}

pub(crate) fn rat(x: i32, y: i32) -> Enemy {
    enemy(
        "Rat",
        'r',
        x,
        y,
        enemy_stats(6, 1, 2, 0, 11),
        enemy_rewards(8, 0, 3),
        false,
    )
}
pub(crate) fn skeleton(x: i32, y: i32) -> Enemy {
    enemy(
        "Skeleton",
        's',
        x,
        y,
        enemy_stats(12, 2, 4, 1, 9),
        enemy_rewards(18, 2, 8),
        false,
    )
}
pub(crate) fn cultist(x: i32, y: i32) -> Enemy {
    enemy(
        "Cultist",
        'c',
        x,
        y,
        enemy_stats(10, 2, 3, 0, 10),
        enemy_rewards(22, 5, 12),
        false,
    )
}
pub(crate) fn boneguard(x: i32, y: i32) -> Enemy {
    enemy(
        "Boneguard",
        'b',
        x,
        y,
        enemy_stats(18, 3, 5, 2, 8),
        enemy_rewards(35, 8, 18),
        false,
    )
}
pub(crate) fn elite_skeleton(x: i32, y: i32) -> Enemy {
    let modifier = random_elite_modifier();
    elite_skeleton_with_modifier(x, y, modifier)
}

pub(crate) fn elite_skeleton_with_modifier(x: i32, y: i32, modifier: EliteModifier) -> Enemy {
    let mut elite = enemy(
        "Elite Skeleton",
        'E',
        x,
        y,
        enemy_stats(24, 3, 6, 2, 10),
        enemy_rewards(54, 20, 40),
        false,
    );
    apply_elite_modifier(&mut elite, modifier);
    elite
}

pub(crate) fn random_elite_modifier() -> EliteModifier {
    match rand::thread_rng().gen_range(0..4) {
        0 => EliteModifier::Armored,
        1 => EliteModifier::Swift,
        2 => EliteModifier::Vampiric,
        _ => EliteModifier::Burning,
    }
}

pub(crate) fn apply_elite_modifier(enemy: &mut Enemy, modifier: EliteModifier) {
    enemy.name = format!("{} {}", elite_modifier_name(&modifier), enemy.name);
    if matches!(modifier, EliteModifier::Swift) {
        enemy.speed += 2;
    }
    enemy.elite_modifier = Some(modifier);
}

pub(crate) fn elite_modifier_name(modifier: &EliteModifier) -> &'static str {
    match modifier {
        EliteModifier::Armored => "Armored",
        EliteModifier::Swift => "Swift",
        EliteModifier::Vampiric => "Vampiric",
        EliteModifier::Burning => "Burning",
    }
}
pub(crate) fn bellkeeper(x: i32, y: i32) -> Enemy {
    enemy(
        "Bellkeeper",
        'B',
        x,
        y,
        enemy_stats(60, 5, 8, 3, 8),
        enemy_rewards(250, 100, 150),
        true,
    )
}
pub(crate) fn dune_stalker(x: i32, y: i32) -> Enemy {
    enemy(
        "Dune Stalker",
        'g',
        x,
        y,
        enemy_stats(16, 4, 7, 1, 13),
        enemy_rewards(42, 12, 24),
        false,
    )
}
pub(crate) fn glass_wraith(x: i32, y: i32) -> Enemy {
    enemy(
        "Glass Wraith",
        'w',
        x,
        y,
        enemy_stats(14, 5, 8, 0, 12),
        enemy_rewards(48, 14, 28),
        false,
    )
}
pub(crate) fn ember_magus(x: i32, y: i32) -> Enemy {
    enemy(
        "Ember Magus",
        'm',
        x,
        y,
        enemy_stats(18, 4, 9, 1, 10),
        enemy_rewards(58, 18, 34),
        false,
    )
}
pub(crate) fn obsidian_guard(x: i32, y: i32) -> Enemy {
    enemy(
        "Obsidian Guard",
        'o',
        x,
        y,
        enemy_stats(28, 5, 9, 4, 8),
        enemy_rewards(72, 22, 44),
        false,
    )
}
pub(crate) fn elite_glass_wraith(x: i32, y: i32) -> Enemy {
    let mut elite = glass_wraith(x, y);
    elite.name = "Mirrored Elite Glass Wraith".to_string();
    elite.glyph = 'E';
    elite.max_hp += 10;
    elite.hp = elite.max_hp;
    elite.damage_min += 1;
    elite.damage_max += 2;
    elite.xp += 45;
    apply_elite_modifier(&mut elite, random_elite_modifier());
    elite
}
pub(crate) fn glass_tyrant(x: i32, y: i32) -> Enemy {
    enemy(
        "Glass Tyrant",
        'T',
        x,
        y,
        enemy_stats(95, 8, 12, 5, 9),
        enemy_rewards(520, 220, 340),
        true,
    )
}
