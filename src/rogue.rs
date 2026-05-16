#[allow(unused_imports)]
use crate::*;

const BACKSTAB_COST: u32 = 25;
const VENOM_EDGE_COST: u32 = 30;
const EVISCERATE_COST: u32 = 35;

fn rogue_skill_not_ready(c: &mut Character, skill: &str) -> bool {
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(
            &mut d.log,
            LogKind::Warn,
            format!("{skill} is not ready yet."),
        );
    }
    false
}

pub(crate) fn add_rogue_combo_point(c: &mut Character) {
    c.rogue.combo_points = (c.rogue.combo_points + 1).min(ROGUE_MAX_COMBO_POINTS);
}

pub(crate) fn backstab_multiplier(c: &Character) -> f32 {
    if empowered_backstab_ready(c) {
        1.20
    } else {
        0.90
    }
}

pub(crate) fn venom_edge_multiplier(_c: &Character) -> f32 {
    0.70
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
) {
    let Some(mut d) = c.active_dungeon.take() else {
        return;
    };
    let ground_items_before_death = d.ground_items.len();
    let mut killed = false;
    if let Some(enemy) = d.enemies.get_mut(enemy_index) {
        if enemy.hp > 0 {
            enemy.hp -= damage;
            killed = enemy.hp <= 0;
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
        return;
    }
    c.active_dungeon = Some(d);
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
    let multiplier = backstab_multiplier(c);
    damage_enemy(c, index, multiplier, "backstab");
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
    damage_enemy(c, index, venom_edge_multiplier(c), "venom edge");
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
    let multiplier = eviscerate_multiplier_for_points(points);
    damage_enemy(c, index, multiplier, "eviscerate");
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
        damage_enemy_with_rogue_effect(c, index, "Eviscerate poison", bonus);
    }
    if c.rogue.slip_away_rank > 0 {
        c.rogue.smoke_protection_turns = c.rogue.smoke_protection_turns.max(1);
    }
    true
}

pub(crate) fn use_smoke_step(c: &mut Character) -> bool {
    rogue_skill_not_ready(c, "Smoke Step")
}

fn log_rogue_warning(c: &mut Character, message: &str) {
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(&mut d.log, LogKind::Warn, message);
    }
}
