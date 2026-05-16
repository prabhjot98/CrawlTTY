#![allow(dead_code)]

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectAvailability {
    Available,
    Completed,
    Locked(&'static str),
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct TownProjectDefinition {
    pub(crate) project: TownProject,
    pub(crate) group: &'static str,
    pub(crate) name: &'static str,
    pub(crate) cost: u32,
    pub(crate) benefit: &'static str,
}

pub(crate) const TOWN_PROJECTS: &[TownProjectDefinition] = &[
    TownProjectDefinition {
        project: TownProject::RebuildForge,
        group: "Smith",
        name: "Rebuild the Forge",
        cost: 150,
        benefit: "Unlock salvage and shard-only gear upgrades.",
    },
    TownProjectDefinition {
        project: TownProject::ReinforcedAnvil,
        group: "Smith",
        name: "Reinforced Anvil",
        cost: 300,
        benefit: "Salvaged gear yields +1 shard.",
    },
    TownProjectDefinition {
        project: TownProject::SocketBench,
        group: "Smith",
        name: "Socket Bench",
        cost: 600,
        benefit: "Unlock socket infrastructure for future gem and rune systems.",
    },
    TownProjectDefinition {
        project: TownProject::StorehouseShelves,
        group: "Quartermaster",
        name: "Storehouse Shelves",
        cost: 200,
        benefit: "Expand town storage infrastructure.",
    },
    TownProjectDefinition {
        project: TownProject::HireAppraiser,
        group: "Appraiser",
        name: "Hire Appraiser",
        cost: 250,
        benefit: "Improve sell prices from 25% to 30%.",
    },
    TownProjectDefinition {
        project: TownProject::HerbGarden,
        group: "Alchemist",
        name: "Herb Garden",
        cost: 350,
        benefit: "Unlock growing herbs.",
    },
    TownProjectDefinition {
        project: TownProject::Distillery,
        group: "Alchemist",
        name: "Distillery",
        cost: 500,
        benefit: "Unlock potion crafting infrastructure.",
    },
];

pub(crate) fn town_project_definition(project: TownProject) -> &'static TownProjectDefinition {
    TOWN_PROJECTS
        .iter()
        .find(|definition| definition.project == project)
        .expect("all town projects have definitions")
}

pub(crate) fn has_completed_project(c: &Character, project: TownProject) -> bool {
    c.completed_town_projects.contains(&project)
}

pub(crate) fn town_project_availability(
    c: &Character,
    project: TownProject,
) -> ProjectAvailability {
    if has_completed_project(c, project) {
        return ProjectAvailability::Completed;
    }
    match project {
        TownProject::RebuildForge | TownProject::StorehouseShelves | TownProject::HireAppraiser => {
            ProjectAvailability::Available
        }
        TownProject::ReinforcedAnvil => {
            if has_completed_project(c, TownProject::RebuildForge) {
                ProjectAvailability::Available
            } else {
                ProjectAvailability::Locked("Requires Rebuild the Forge.")
            }
        }
        TownProject::SocketBench => {
            if !c.act1_completed {
                ProjectAvailability::Locked("Requires Act I completed.")
            } else if !has_completed_project(c, TownProject::ReinforcedAnvil) {
                ProjectAvailability::Locked("Requires Reinforced Anvil.")
            } else {
                ProjectAvailability::Available
            }
        }
        TownProject::HerbGarden => {
            if c.act1_completed {
                ProjectAvailability::Available
            } else {
                ProjectAvailability::Locked("Requires Act I completed.")
            }
        }
        TownProject::Distillery => {
            if has_completed_project(c, TownProject::HerbGarden) {
                ProjectAvailability::Available
            } else {
                ProjectAvailability::Locked("Requires Herb Garden.")
            }
        }
    }
}

pub(crate) fn complete_town_project(c: &mut Character, project: TownProject) -> String {
    let definition = town_project_definition(project);
    match town_project_availability(c, project) {
        ProjectAvailability::Completed => {
            return format!("{} is already complete.", definition.name);
        }
        ProjectAvailability::Locked(reason) => return reason.to_string(),
        ProjectAvailability::Available => {}
    }
    if c.gold < definition.cost {
        return format!(
            "Need {} gold to complete {}.",
            definition.cost, definition.name
        );
    }
    c.gold -= definition.cost;
    c.completed_town_projects.push(project);
    format!("Completed project: {}.", definition.name)
}

#[cfg(test)]
pub(crate) fn complete_project_for_test(c: &mut Character, project: TownProject) {
    if !has_completed_project(c, project) {
        c.completed_town_projects.push(project);
    }
}
