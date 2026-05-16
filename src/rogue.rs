#[allow(unused_imports)]
use crate::*;

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

pub(crate) fn use_backstab(c: &mut Character) -> bool {
    rogue_skill_not_ready(c, "Backstab")
}

pub(crate) fn use_venom_edge(c: &mut Character) -> bool {
    rogue_skill_not_ready(c, "Venom Edge")
}

pub(crate) fn use_eviscerate(c: &mut Character) -> bool {
    rogue_skill_not_ready(c, "Eviscerate")
}

pub(crate) fn use_smoke_step(c: &mut Character) -> bool {
    rogue_skill_not_ready(c, "Smoke Step")
}
