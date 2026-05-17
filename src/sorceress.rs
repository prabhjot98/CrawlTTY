use crate::*;

pub(crate) const FIREBOLT_MANA_COST: u32 = 4;
pub(crate) const FROST_RING_MANA_COST: u32 = 8;
pub(crate) const CHAIN_SPARK_MANA_COST: u32 = 7;
pub(crate) const FROST_RING_COOLDOWN: u32 = 3;
pub(crate) const CHAIN_SPARK_COOLDOWN: u32 = 2;
pub(crate) const BURNING_TURNS: u32 = 3;
pub(crate) const FROZEN_TURNS: u32 = 1;
pub(crate) const CHAIN_SPARK_JUMP_RADIUS: i32 = 2;

pub(crate) fn firebolt_percent_for_rank(rank: u32) -> u32 {
    100 + rank.saturating_sub(1).min(4) * 10
}

pub(crate) fn firebolt_burn_chance_for_rank(rank: u32) -> u32 {
    25 + rank.saturating_sub(1).min(4) * 5
}

pub(crate) fn frost_ring_percent_for_rank(rank: u32) -> u32 {
    70 + rank.saturating_sub(1).min(4) * 10
}

pub(crate) fn frost_ring_freeze_chance_for_rank(rank: u32) -> u32 {
    20 + rank.saturating_sub(1).min(4) * 5
}

pub(crate) fn chain_spark_percent_for_rank(rank: u32) -> u32 {
    match rank.min(5) {
        0 | 1 => 80,
        2 => 90,
        3 => 95,
        4 => 105,
        _ => 110,
    }
}

pub(crate) fn chain_spark_hit_count_for_rank(rank: u32) -> usize {
    match rank.min(5) {
        0..=2 => 2,
        3 | 4 => 3,
        _ => 4,
    }
}

pub(crate) fn mana_shield_absorb_percent_for_rank(rank: u32) -> u32 {
    if rank == 0 {
        0
    } else {
        35 + rank.saturating_sub(1).min(4) * 5
    }
}

pub(crate) fn kindle_fire_bonus_percent_for_rank(rank: u32) -> u32 {
    if rank == 0 {
        0
    } else {
        10 + rank.saturating_sub(1).min(4) * 5
    }
}

pub(crate) fn static_charge_chance_for_rank(rank: u32) -> u32 {
    if rank == 0 {
        0
    } else {
        15 + rank.saturating_sub(1).min(4) * 5
    }
}

pub(crate) fn static_charge_damage_bonus_for_rank(rank: u32) -> u32 {
    static_charge_chance_for_rank(rank)
}

pub(crate) fn apply_shocked_if_stronger(enemy: &mut Enemy, bonus_percent: u32) {
    if bonus_percent >= enemy.shocked_bonus_percent {
        enemy.shocked_bonus_percent = bonus_percent;
    }
}

pub(crate) fn apply_shock_bonus_to_damage(enemy: &mut Enemy, damage: i32) -> i32 {
    let bonus = enemy.shocked_bonus_percent;
    if bonus == 0 {
        return damage;
    }
    enemy.shocked_bonus_percent = 0;
    damage + ((damage * bonus as i32) / 100)
}

pub(crate) fn spell_damage_range(c: &Character) -> (i32, i32) {
    let int_bonus = (c.effective_intelligence() as i32 / 3).max(0);
    (
        (c.equipped_weapon.damage_min + int_bonus).max(1),
        (c.equipped_weapon.damage_max + int_bonus).max(1),
    )
}

pub(crate) fn spell_damage_for_roll(c: &Character, percent: u32, roll: f64) -> i32 {
    let (min, max) = spell_damage_range(c);
    let span = (max - min).max(0);
    let rolled = min + ((span as f64 * roll.clamp(0.0, 1.0)).round() as i32);
    ((rolled as f32) * (percent as f32 / 100.0))
        .round()
        .max(1.0) as i32
}

pub(crate) fn nearest_visible_enemy_index(c: &Character) -> Option<usize> {
    let d = c.active_dungeon.as_ref()?;
    d.enemies
        .iter()
        .enumerate()
        .filter(|(_, enemy)| {
            enemy.hp > 0 && clear_line_of_sight(d, d.player_x, d.player_y, enemy.x, enemy.y)
        })
        .min_by_key(|(_, enemy)| {
            let dx = (enemy.x - d.player_x).abs();
            let dy = (enemy.y - d.player_y).abs();
            (dx + dy, dy.max(dx), enemy.y, enemy.x)
        })
        .map(|(index, _)| index)
}

pub(crate) fn clear_line_of_sight(
    d: &Dungeon,
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
) -> bool {
    let mut x0 = from_x;
    let mut y0 = from_y;
    let x1 = to_x;
    let y1 = to_y;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if (x0, y0) != (from_x, from_y)
            && (x0, y0) != (to_x, to_y)
            && dungeon_tile(d, x0, y0) == '#'
        {
            return false;
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    true
}

fn log_sorceress_warning(c: &mut Character, message: impl Into<String>) {
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(&mut d.log, LogKind::Warn, message.into());
    }
}

fn spend_sorceress_mana(c: &mut Character, amount: u32) {
    c.mana = c.mana.saturating_sub(amount);
    if c.mana == 0 {
        c.sorceress.mana_shield_active = false;
    }
}

fn spell_damage_after_mitigation(
    c: &Character,
    enemy: &Enemy,
    raw_damage: i32,
    is_fire: bool,
) -> i32 {
    let mut adjusted = raw_damage;
    if is_fire && enemy.burning_turns > 0 && c.sorceress.kindle_rank > 0 {
        let bonus = kindle_fire_bonus_percent_for_rank(c.sorceress.kindle_rank) as i32;
        adjusted += (adjusted * bonus) / 100;
    }
    (adjusted - effective_enemy_armor(enemy)).max(1)
}

fn resolve_spell_death(
    c: &mut Character,
    mut d: Dungeon,
    enemy_index: usize,
    source: &'static str,
    ground_items_before_death: usize,
) -> bool {
    if resolve_enemy_death(c, &mut d, enemy_index, EnemyDeathCause::Effect { source }) {
        finish_boss_defeat_after_effect_kill(c, d, ground_items_before_death);
        true
    } else {
        maybe_drop_loot_in_dungeon(c, &mut d, enemy_index, false);
        c.active_dungeon = Some(d);
        true
    }
}

pub(crate) fn use_firebolt_with_rolls(
    c: &mut Character,
    hit_roll: f64,
    burn_roll: f64,
    damage_roll: f64,
) -> bool {
    if c.mana < FIREBOLT_MANA_COST {
        log_sorceress_warning(c, "Not enough mana for Firebolt.");
        return false;
    }
    let Some(enemy_index) = nearest_visible_enemy_index(c) else {
        log_sorceress_warning(c, "No enemy in sight.");
        return false;
    };
    spend_sorceress_mana(c, FIREBOLT_MANA_COST);

    let Some(mut d) = c.active_dungeon.take() else {
        return false;
    };
    if enemy_index >= d.enemies.len() || d.enemies[enemy_index].hp <= 0 {
        c.active_dungeon = Some(d);
        return true;
    }
    let hit_chance = player_attack_hit_chance(c, &d.enemies[enemy_index]);
    if hit_roll >= hit_chance {
        let name = d.enemies[enemy_index].name.clone();
        log_event(
            &mut d.log,
            LogKind::Miss,
            format!("Firebolt misses {name}."),
        );
        c.active_dungeon = Some(d);
        return true;
    }

    let rank = c.sorceress.firebolt_rank;
    let raw = spell_damage_for_roll(c, firebolt_percent_for_rank(rank), damage_roll);
    let damage = spell_damage_after_mitigation(c, &d.enemies[enemy_index], raw, true);
    let ground_items_before_death = d.ground_items.len();
    let (name, damage, hp_text, killed) = {
        let enemy = &mut d.enemies[enemy_index];
        let damage = apply_shock_bonus_to_damage(enemy, damage);
        enemy.hp -= damage;
        (
            enemy.name.clone(),
            damage,
            enemy_hp_text(enemy),
            enemy.hp <= 0,
        )
    };
    log_event(
        &mut d.log,
        LogKind::Hit,
        format!(
            "Firebolt hits {name} for {}. {hp_text}.",
            damage_text(damage)
        ),
    );
    if killed {
        return resolve_spell_death(c, d, enemy_index, "Firebolt", ground_items_before_death);
    }
    if burn_roll < firebolt_burn_chance_for_rank(rank) as f64 / 100.0 {
        let burning_damage = 1 + rank.div_ceil(2) as i32;
        if let Some(enemy) = d.enemies.get_mut(enemy_index) {
            enemy.burning_turns = enemy.burning_turns.max(BURNING_TURNS);
            enemy.burning_damage = enemy.burning_damage.max(burning_damage);
        }
        log_event(
            &mut d.log,
            LogKind::Status,
            format!("Firebolt burns {name}."),
        );
    }
    c.active_dungeon = Some(d);
    true
}

pub(crate) fn use_firebolt(c: &mut Character) -> bool {
    let mut rng = rand::thread_rng();
    use_firebolt_with_rolls(
        c,
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
    )
}

fn frost_ring_targets(c: &Character) -> Vec<usize> {
    let Some(d) = c.active_dungeon.as_ref() else {
        return Vec::new();
    };
    d.enemies
        .iter()
        .enumerate()
        .filter(|(_, enemy)| {
            enemy.hp > 0
                && (enemy.x - d.player_x).abs() <= 1
                && (enemy.y - d.player_y).abs() <= 1
                && (enemy.x, enemy.y) != (d.player_x, d.player_y)
        })
        .map(|(index, _)| index)
        .collect()
}

#[cfg(test)]
pub(crate) fn use_frost_ring_with_rolls(
    c: &mut Character,
    hit_roll: f64,
    freeze_roll: f64,
    damage_roll: f64,
) -> bool {
    use_frost_ring_with_roll_source(c, || hit_roll, || freeze_roll, || damage_roll)
}

fn use_frost_ring_with_roll_source(
    c: &mut Character,
    mut hit_roll: impl FnMut() -> f64,
    mut freeze_roll: impl FnMut() -> f64,
    mut damage_roll: impl FnMut() -> f64,
) -> bool {
    if c.sorceress.frost_ring_cooldown > 0 {
        log_sorceress_warning(
            c,
            format!(
                "Frost Ring is on cooldown for {} more turns.",
                c.sorceress.frost_ring_cooldown
            ),
        );
        return false;
    }
    if c.mana < FROST_RING_MANA_COST {
        log_sorceress_warning(c, "Not enough mana for Frost Ring.");
        return false;
    }
    let targets = frost_ring_targets(c);
    if targets.is_empty() {
        log_sorceress_warning(c, "No enemies in Frost Ring range.");
        return false;
    }
    spend_sorceress_mana(c, FROST_RING_MANA_COST);
    c.sorceress.frost_ring_cooldown = FROST_RING_COOLDOWN;

    let Some(mut d) = c.active_dungeon.take() else {
        return false;
    };
    let rank = c.sorceress.frost_ring_rank;
    for enemy_index in targets {
        if enemy_index >= d.enemies.len() || d.enemies[enemy_index].hp <= 0 {
            continue;
        }
        let name = d.enemies[enemy_index].name.clone();
        if hit_roll() >= player_attack_hit_chance(c, &d.enemies[enemy_index]) {
            log_event(
                &mut d.log,
                LogKind::Miss,
                format!("Frost Ring misses {name}."),
            );
            continue;
        }
        let raw = spell_damage_for_roll(c, frost_ring_percent_for_rank(rank), damage_roll());
        let damage = spell_damage_after_mitigation(c, &d.enemies[enemy_index], raw, false);
        let ground_items_before_death = d.ground_items.len();
        let (damage, hp_text, killed) = {
            let enemy = &mut d.enemies[enemy_index];
            let damage = apply_shock_bonus_to_damage(enemy, damage);
            enemy.hp -= damage;
            (damage, enemy_hp_text(enemy), enemy.hp <= 0)
        };
        log_event(
            &mut d.log,
            LogKind::Hit,
            format!(
                "Frost Ring hits {name} for {}. {hp_text}.",
                damage_text(damage)
            ),
        );
        if killed {
            if resolve_enemy_death(
                c,
                &mut d,
                enemy_index,
                EnemyDeathCause::Effect {
                    source: "Frost Ring",
                },
            ) {
                finish_boss_defeat_after_effect_kill(c, d, ground_items_before_death);
                return true;
            }
            maybe_drop_loot_in_dungeon(c, &mut d, enemy_index, false);
            continue;
        }
        if freeze_roll() < frost_ring_freeze_chance_for_rank(rank) as f64 / 100.0 {
            if let Some(enemy) = d.enemies.get_mut(enemy_index) {
                enemy.frozen_turns = enemy.frozen_turns.max(FROZEN_TURNS);
            }
            log_event(
                &mut d.log,
                LogKind::Status,
                format!("Frost Ring freezes {name}."),
            );
        }
    }
    c.active_dungeon = Some(d);
    true
}

pub(crate) fn use_frost_ring(c: &mut Character) -> bool {
    use_frost_ring_with_roll_source(
        c,
        || rand::thread_rng().gen_range(0.0..1.0),
        || rand::thread_rng().gen_range(0.0..1.0),
        || rand::thread_rng().gen_range(0.0..1.0),
    )
}

fn open_for_chain_jump(d: &Dungeon, x: i32, y: i32) -> bool {
    dungeon_tile(d, x, y) != '#'
}

fn chain_jump_can_step(d: &Dungeon, from: (i32, i32), to: (i32, i32)) -> bool {
    if !open_for_chain_jump(d, to.0, to.1) {
        return false;
    }
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    if dx != 0 && dy != 0 {
        open_for_chain_jump(d, from.0 + dx, from.1) && open_for_chain_jump(d, from.0, from.1 + dy)
    } else {
        true
    }
}

pub(crate) fn chain_jump_reachable_tiles(
    d: &Dungeon,
    start: (i32, i32),
    max_steps: i32,
) -> Vec<(i32, i32)> {
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    visited.insert(start);
    queue.push_back((start, 0));
    while let Some((pos, steps)) = queue.pop_front() {
        if steps >= max_steps {
            continue;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let next = (pos.0 + dx, pos.1 + dy);
                if visited.contains(&next) || !chain_jump_can_step(d, pos, next) {
                    continue;
                }
                visited.insert(next);
                queue.push_back((next, steps + 1));
            }
        }
    }
    visited.remove(&start);
    let mut tiles = visited.into_iter().collect::<Vec<_>>();
    tiles.sort_by_key(|(x, y)| ((x - start.0).abs().max((y - start.1).abs()), *y, *x));
    tiles
}

fn next_chain_target(d: &Dungeon, previous: (i32, i32), hit_indices: &[usize]) -> Option<usize> {
    let reachable = chain_jump_reachable_tiles(d, previous, CHAIN_SPARK_JUMP_RADIUS);
    d.enemies
        .iter()
        .enumerate()
        .filter(|(index, enemy)| {
            enemy.hp > 0 && !hit_indices.contains(index) && reachable.contains(&(enemy.x, enemy.y))
        })
        .min_by_key(|(_, enemy)| {
            (
                (enemy.x - previous.0)
                    .abs()
                    .max((enemy.y - previous.1).abs()),
                enemy.y,
                enemy.x,
            )
        })
        .map(|(index, _)| index)
}

#[cfg(test)]
pub(crate) fn use_chain_spark_with_rolls(
    c: &mut Character,
    hit_roll: f64,
    shock_roll: f64,
    damage_roll: f64,
) -> bool {
    use_chain_spark_with_roll_source(c, || hit_roll, || shock_roll, || damage_roll)
}

fn use_chain_spark_with_roll_source(
    c: &mut Character,
    mut hit_roll: impl FnMut() -> f64,
    mut shock_roll: impl FnMut() -> f64,
    mut damage_roll: impl FnMut() -> f64,
) -> bool {
    if c.sorceress.chain_spark_cooldown > 0 {
        log_sorceress_warning(
            c,
            format!(
                "Chain Spark is on cooldown for {} more turns.",
                c.sorceress.chain_spark_cooldown
            ),
        );
        return false;
    }
    if c.mana < CHAIN_SPARK_MANA_COST {
        log_sorceress_warning(c, "Not enough mana for Chain Spark.");
        return false;
    }
    let Some(initial_target) = nearest_visible_enemy_index(c) else {
        log_sorceress_warning(c, "No enemy in sight.");
        return false;
    };
    spend_sorceress_mana(c, CHAIN_SPARK_MANA_COST);
    c.sorceress.chain_spark_cooldown = CHAIN_SPARK_COOLDOWN;

    let Some(mut d) = c.active_dungeon.take() else {
        return false;
    };
    let rank = c.sorceress.chain_spark_rank;
    let max_hits = chain_spark_hit_count_for_rank(rank);
    let mut current = initial_target;
    let mut hit_indices = Vec::new();

    while hit_indices.len() < max_hits {
        if current >= d.enemies.len() || d.enemies[current].hp <= 0 {
            break;
        }
        let name = d.enemies[current].name.clone();
        if hit_roll() >= player_attack_hit_chance(c, &d.enemies[current]) {
            log_event(
                &mut d.log,
                LogKind::Miss,
                format!("Chain Spark misses {name}."),
            );
            break;
        }

        let raw = spell_damage_for_roll(c, chain_spark_percent_for_rank(rank), damage_roll());
        let damage = spell_damage_after_mitigation(c, &d.enemies[current], raw, false);
        let ground_items_before_death = d.ground_items.len();
        let position = (d.enemies[current].x, d.enemies[current].y);
        let (damage, hp_text, killed) = {
            let enemy = &mut d.enemies[current];
            let damage = apply_shock_bonus_to_damage(enemy, damage);
            enemy.hp -= damage;
            (damage, enemy_hp_text(enemy), enemy.hp <= 0)
        };
        log_event(
            &mut d.log,
            LogKind::Hit,
            format!(
                "Chain Spark hits {name} for {}. {hp_text}.",
                damage_text(damage)
            ),
        );
        if !killed && c.sorceress.static_charge_rank > 0 {
            let shock_chance =
                static_charge_chance_for_rank(c.sorceress.static_charge_rank) as f64 / 100.0;
            if shock_roll() < shock_chance {
                let bonus = static_charge_damage_bonus_for_rank(c.sorceress.static_charge_rank);
                if let Some(enemy) = d.enemies.get_mut(current) {
                    apply_shocked_if_stronger(enemy, bonus);
                }
                log_event(
                    &mut d.log,
                    LogKind::Status,
                    format!("Chain Spark shocks {name}."),
                );
            }
        }
        hit_indices.push(current);
        if killed {
            if resolve_enemy_death(
                c,
                &mut d,
                current,
                EnemyDeathCause::Effect {
                    source: "Chain Spark",
                },
            ) {
                finish_boss_defeat_after_effect_kill(c, d, ground_items_before_death);
                return true;
            }
            maybe_drop_loot_in_dungeon(c, &mut d, current, false);
        }
        let Some(next) = next_chain_target(&d, position, &hit_indices) else {
            break;
        };
        current = next;
    }

    c.active_dungeon = Some(d);
    true
}

pub(crate) fn use_chain_spark(c: &mut Character) -> bool {
    use_chain_spark_with_roll_source(
        c,
        || rand::thread_rng().gen_range(0.0..1.0),
        || rand::thread_rng().gen_range(0.0..1.0),
        || rand::thread_rng().gen_range(0.0..1.0),
    )
}

pub(crate) fn toggle_mana_shield(c: &mut Character) -> bool {
    if c.sorceress.mana_shield_rank == 0 {
        let message = if c.sorceress.frost_ring_rank < 2 {
            "Mana Shield requires Frost Ring rank 2."
        } else {
            "Mana Shield is unlocked; spend a skill point to learn it."
        };
        if let Some(d) = c.active_dungeon.as_mut() {
            log_event(&mut d.log, LogKind::Warn, message);
        }
        return false;
    }
    if !c.sorceress.mana_shield_active && c.mana == 0 {
        if let Some(d) = c.active_dungeon.as_mut() {
            log_event(&mut d.log, LogKind::Warn, "Mana Shield requires mana.");
        }
        return false;
    }
    c.sorceress.mana_shield_active = !c.sorceress.mana_shield_active;
    if let Some(d) = c.active_dungeon.as_mut() {
        let state = if c.sorceress.mana_shield_active {
            "on"
        } else {
            "off"
        };
        log_event(
            &mut d.log,
            LogKind::Status,
            format!("Mana Shield toggled {state}."),
        );
    }
    false
}
