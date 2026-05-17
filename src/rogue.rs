#[allow(unused_imports)]
use crate::*;

const BACKSTAB_COST: u32 = 25;
const VENOM_EDGE_COST: u32 = 30;
const EVISCERATE_COST: u32 = 35;
const SMOKE_STEP_COST: u32 = 35;
const SMOKE_STEP_COOLDOWN: u32 = 4;

pub(crate) fn add_rogue_combo_point(c: &mut Character) {
    c.rogue.combo_points = (c.rogue.combo_points + 1).min(ROGUE_MAX_COMBO_POINTS);
}

pub(crate) fn backstab_base_percent_for_rank(rank: u32) -> u32 {
    90 + rank.saturating_sub(1).min(4) * 5
}

pub(crate) fn empowered_backstab_percent_for_rank(rank: u32) -> u32 {
    120 + rank.saturating_sub(1).min(4) * 10
}

pub(crate) fn venom_edge_percent_for_rank(rank: u32) -> u32 {
    70 + rank.saturating_sub(1).min(4) * 5
}

pub(crate) fn eviscerate_bonus_percent_for_rank(rank: u32) -> u32 {
    rank.saturating_sub(1).min(4) * 10
}

pub(crate) fn smoke_step_dodge_bonus_for_rank(rank: u32) -> i32 {
    20 + rank.saturating_sub(1).min(4) as i32 * 3
}

#[allow(dead_code)]
pub(crate) fn slip_away_dodge_bonus_for_rank(rank: u32) -> i32 {
    5 + rank.saturating_sub(1).min(4) as i32 * 2
}

pub(crate) fn backstab_multiplier(c: &Character) -> f32 {
    let percent = if empowered_backstab_ready(c) {
        empowered_backstab_percent_for_rank(c.rogue.backstab_rank)
    } else {
        backstab_base_percent_for_rank(c.rogue.backstab_rank)
    };
    percent as f32 / 100.0
}

pub(crate) fn backstab_multiplier_for_target(c: &Character, enemy_index: usize) -> f32 {
    let target_poisoned = c
        .active_dungeon
        .as_ref()
        .and_then(|d| d.enemies.get(enemy_index))
        .is_some_and(|enemy| enemy.poison_turns > 0);
    let percent = if target_poisoned {
        empowered_backstab_percent_for_rank(c.rogue.backstab_rank)
    } else {
        return backstab_multiplier(c);
    };
    percent as f32 / 100.0
}

pub(crate) fn venom_edge_multiplier(c: &Character) -> f32 {
    venom_edge_percent_for_rank(c.rogue.venom_edge_rank) as f32 / 100.0
}

pub(crate) fn poison_damage_for_rank(rank: u32) -> i32 {
    1 + rank.min(5).div_ceil(2) as i32
}

pub(crate) fn eviscerate_multiplier_for_points(points: u32) -> f32 {
    match points.min(5) {
        0 => 0.0,
        1 => 0.80,
        2 => 1.30,
        3 => 1.90,
        4 => 2.60,
        _ => 3.50,
    }
}

pub(crate) fn empowered_backstab_ready(c: &Character) -> bool {
    c.rogue.empowered_backstab_turns > 0
}

pub(crate) fn grant_rogue_movement_backstab(c: &mut Character) {
    if c.class == CharacterClass::Rogue {
        // Movement is a player action, so the dungeon loop immediately ticks this
        // down once. Two turns here leaves one empowered Backstab window.
        c.rogue.empowered_backstab_turns = c.rogue.empowered_backstab_turns.max(2);
    }
}

fn adjacent_rogue_target(c: &mut Character, skill: &str) -> Option<usize> {
    c.active_dungeon.as_ref()?;
    let target = adjacent_enemy_indices(c).first().copied();
    if target.is_none() {
        if let Some(d) = c.active_dungeon.as_mut() {
            log_event(
                &mut d.log,
                LogKind::Warn,
                format!("No adjacent enemy for {skill}."),
            );
        }
    }
    target
}

pub(crate) fn damage_enemy_with_rogue_effect(
    c: &mut Character,
    enemy_index: usize,
    source: &'static str,
    damage: i32,
) -> DamageEnemyOutcome {
    let Some(mut d) = c.active_dungeon.take() else {
        return DamageEnemyOutcome::NoTarget;
    };
    let ground_items_before_death = d.ground_items.len();
    let mut outcome = DamageEnemyOutcome::NoTarget;
    let mut killed = false;
    if let Some(enemy) = d.enemies.get_mut(enemy_index) {
        if enemy.hp > 0 {
            enemy.hp -= damage;
            killed = enemy.hp <= 0;
            outcome = if killed {
                DamageEnemyOutcome::Killed
            } else {
                DamageEnemyOutcome::Hit
            };
            log_event(
                &mut d.log,
                LogKind::Hit,
                format!(
                    "{source} deals {}. {}.",
                    damage_text(damage),
                    enemy_hp_text(enemy)
                ),
            );
        }
    }
    if killed && resolve_enemy_death(c, &mut d, enemy_index, EnemyDeathCause::Effect { source }) {
        finish_boss_defeat_after_effect_kill(c, d, ground_items_before_death);
        return DamageEnemyOutcome::BossDefeated;
    }
    c.active_dungeon = Some(d);
    outcome
}

pub(crate) fn use_backstab(c: &mut Character) -> bool {
    if !c.spend_rogue_energy(BACKSTAB_COST) {
        log_rogue_warning(c, "Not enough Energy for Backstab.");
        return false;
    }
    let Some(index) = adjacent_rogue_target(c, "Backstab") else {
        c.restore_rogue_energy(BACKSTAB_COST);
        return false;
    };
    let multiplier = backstab_multiplier_for_target(c, index);
    let outcome = damage_enemy(c, index, multiplier, "backstab");
    if outcome == DamageEnemyOutcome::BossDefeated {
        return true;
    }
    add_rogue_combo_point(c);
    c.rogue.empowered_backstab_turns = 0;
    true
}

pub(crate) fn use_venom_edge(c: &mut Character) -> bool {
    if !c.spend_rogue_energy(VENOM_EDGE_COST) {
        log_rogue_warning(c, "Not enough Energy for Venom Edge.");
        return false;
    }
    let Some(index) = adjacent_rogue_target(c, "Venom Edge") else {
        c.restore_rogue_energy(VENOM_EDGE_COST);
        return false;
    };
    let outcome = damage_enemy(c, index, venom_edge_multiplier(c), "venom edge");
    if outcome == DamageEnemyOutcome::BossDefeated {
        return true;
    }
    let poison_damage = poison_damage_for_rank(c.rogue.venom_edge_rank);
    if let Some(enemy) = c
        .active_dungeon
        .as_mut()
        .and_then(|d| d.enemies.get_mut(index))
    {
        if enemy.hp > 0 {
            enemy.poison_turns = enemy.poison_turns.max(3);
            enemy.poison_damage = enemy.poison_damage.max(poison_damage);
        }
    }
    add_rogue_combo_point(c);
    true
}

pub(crate) fn use_eviscerate(c: &mut Character) -> bool {
    let points = c.rogue.combo_points;
    if points == 0 {
        log_rogue_warning(c, "Eviscerate requires combo points.");
        return false;
    }
    if !c.spend_rogue_energy(EVISCERATE_COST) {
        log_rogue_warning(c, "Not enough Energy for Eviscerate.");
        return false;
    }
    let Some(index) = adjacent_rogue_target(c, "Eviscerate") else {
        c.restore_rogue_energy(EVISCERATE_COST);
        return false;
    };
    let multiplier = eviscerate_multiplier_for_points(points)
        + (eviscerate_bonus_percent_for_rank(c.rogue.eviscerate_rank) as f32 / 100.0);
    let outcome = damage_enemy(c, index, multiplier, "eviscerate");
    if outcome == DamageEnemyOutcome::BossDefeated {
        return true;
    }
    c.rogue.combo_points = 0;
    let poison_bonus = {
        let Some(d) = c.active_dungeon.as_mut() else {
            return true;
        };
        let Some(enemy) = d.enemies.get_mut(index) else {
            return true;
        };
        if enemy.hp > 0 && enemy.poison_turns > 0 {
            enemy.poison_turns = enemy.poison_turns.saturating_sub(1);
            Some(enemy.poison_damage.max(1) * points as i32)
        } else {
            None
        }
    };
    if let Some(bonus) = poison_bonus {
        if damage_enemy_with_rogue_effect(c, index, "Eviscerate poison", bonus)
            == DamageEnemyOutcome::BossDefeated
        {
            return true;
        }
    }
    if c.rogue.slip_away_rank > 0 {
        c.rogue.smoke_protection_turns = c.rogue.smoke_protection_turns.max(1);
    }
    true
}

pub(crate) fn try_smoke_step(c: &mut Character, dx: i32, dy: i32) -> bool {
    if c.rogue.smoke_step_cooldown > 0 {
        log_rogue_warning(c, "Smoke Step is on cooldown.");
        return false;
    }
    if dx.abs() + dy.abs() == 0 || dx.abs() + dy.abs() > 2 || (dx != 0 && dy != 0) {
        log_rogue_warning(c, "Smoke Step must move 1 or 2 cardinal tiles.");
        return false;
    }
    if !c.spend_rogue_energy(SMOKE_STEP_COST) {
        log_rogue_warning(c, "Not enough Energy for Smoke Step.");
        return false;
    }
    let (nx, ny, blocked) = {
        let Some(d) = c.active_dungeon.as_ref() else {
            c.restore_rogue_energy(SMOKE_STEP_COST);
            return false;
        };
        let nx = d.player_x + dx;
        let ny = d.player_y + dy;
        let blocked = !smoke_step_path_is_clear(d, dx, dy);
        (nx, ny, blocked)
    };
    if blocked {
        c.restore_rogue_energy(SMOKE_STEP_COST);
        if let Some(d) = c.active_dungeon.as_mut() {
            log_event(
                &mut d.log,
                LogKind::Warn,
                "Smoke Step destination is blocked.",
            );
        }
        return false;
    }
    let Some(d) = c.active_dungeon.as_mut() else {
        c.restore_rogue_energy(SMOKE_STEP_COST);
        return false;
    };
    d.player_x = nx;
    d.player_y = ny;
    c.rogue.smoke_step_cooldown = SMOKE_STEP_COOLDOWN;
    c.rogue.smoke_protection_turns = 1;
    c.rogue.empowered_backstab_turns = c.rogue.empowered_backstab_turns.max(2);
    log_event(&mut d.log, LogKind::Status, "You vanish through smoke.");
    true
}

pub(crate) fn smoke_step_direction(c: &Character) -> Option<(i32, i32)> {
    let d = c.active_dungeon.as_ref()?;
    let directions = [
        (2, 0),
        (-2, 0),
        (0, 2),
        (0, -2),
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
    ];
    directions
        .into_iter()
        .find(|&(dx, dy)| smoke_step_path_is_clear(d, dx, dy))
}

pub(crate) fn use_smoke_step(c: &mut Character) -> bool {
    let Some((dx, dy)) = smoke_step_direction(c) else {
        log_rogue_warning(c, "No open tile for Smoke Step.");
        return false;
    };
    try_smoke_step(c, dx, dy)
}

fn smoke_step_path_is_clear(d: &Dungeon, dx: i32, dy: i32) -> bool {
    let step_x = dx.signum();
    let step_y = dy.signum();
    let steps = dx.abs().max(dy.abs());

    (1..=steps).all(|step| {
        let x = d.player_x + (step_x * step);
        let y = d.player_y + (step_y * step);
        dungeon_tile(d, x, y) != '#'
            && !d
                .enemies
                .iter()
                .any(|enemy| enemy.hp > 0 && enemy.x == x && enemy.y == y)
    })
}

fn log_rogue_warning(c: &mut Character, message: &str) {
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(&mut d.log, LogKind::Warn, message);
    }
}
