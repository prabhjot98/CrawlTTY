use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectAvailability {
    Available,
    Completed,
    Locked(&'static str),
}

#[derive(Debug, Clone, Copy)]
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
        benefit: "Unlock free gem insertion, removal, and replacement.",
    },
    TownProjectDefinition {
        project: TownProject::StorehouseShelves,
        group: "Quartermaster",
        name: "Storehouse Shelves",
        cost: 200,
        benefit: "Expand the bag to 5 x 4.",
    },
    TownProjectDefinition {
        project: TownProject::PackHooks,
        group: "Quartermaster",
        name: "Pack Hooks",
        cost: 350,
        benefit: "Expand the bag to 5 x 5.",
    },
    TownProjectDefinition {
        project: TownProject::OilclothSatchel,
        group: "Quartermaster",
        name: "Oilcloth Satchel",
        cost: 500,
        benefit: "Expand the bag to 6 x 5.",
    },
    TownProjectDefinition {
        project: TownProject::QuartermasterLedger,
        group: "Quartermaster",
        name: "Quartermaster Ledger",
        cost: 700,
        benefit: "Expand the bag to 6 x 6.",
    },
    TownProjectDefinition {
        project: TownProject::ReinforcedPack,
        group: "Quartermaster",
        name: "Reinforced Pack",
        cost: 950,
        benefit: "Expand the bag to 7 x 6.",
    },
    TownProjectDefinition {
        project: TownProject::StitchedPockets,
        group: "Quartermaster",
        name: "Stitched Pockets",
        cost: 1200,
        benefit: "Expand the bag to 7 x 7.",
    },
    TownProjectDefinition {
        project: TownProject::DeepRucksack,
        group: "Quartermaster",
        name: "Deep Rucksack",
        cost: 1500,
        benefit: "Expand the bag to 8 x 7.",
    },
    TownProjectDefinition {
        project: TownProject::ExilesTrunk,
        group: "Quartermaster",
        name: "Exile's Trunk",
        cost: 1900,
        benefit: "Expand the bag to 8 x 8.",
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

pub(crate) fn bag_dimensions(c: &Character) -> (u16, u16) {
    let mut dimensions = (STARTING_BAG_COLUMNS, STARTING_BAG_ROWS);
    for (project, upgraded) in BAG_UPGRADE_PROJECTS {
        if has_completed_project(c, *project) {
            dimensions = *upgraded;
        }
    }
    dimensions
}

pub(crate) const BAG_UPGRADE_PROJECTS: &[(TownProject, (u16, u16))] = &[
    (TownProject::StorehouseShelves, (5, 4)),
    (TownProject::PackHooks, (5, 5)),
    (TownProject::OilclothSatchel, (6, 5)),
    (TownProject::QuartermasterLedger, (6, 6)),
    (TownProject::ReinforcedPack, (7, 6)),
    (TownProject::StitchedPockets, (7, 7)),
    (TownProject::DeepRucksack, (8, 7)),
    (TownProject::ExilesTrunk, (8, 8)),
];

fn previous_bag_project(project: TownProject) -> Option<TownProject> {
    let index = BAG_UPGRADE_PROJECTS
        .iter()
        .position(|(candidate, _)| *candidate == project)?;
    index
        .checked_sub(1)
        .map(|previous| BAG_UPGRADE_PROJECTS[previous].0)
}

fn is_bag_upgrade_project(project: TownProject) -> bool {
    BAG_UPGRADE_PROJECTS
        .iter()
        .any(|(candidate, _)| *candidate == project)
}

fn resize_bag_for_projects(c: &mut Character) {
    let (columns, rows) = bag_dimensions(c);
    c.inventory.columns = columns;
    c.inventory.rows = rows;
}

fn bag_project_lock_reason(project: TownProject) -> Option<&'static str> {
    match project {
        TownProject::PackHooks => Some("Requires Storehouse Shelves."),
        TownProject::OilclothSatchel => Some("Requires Pack Hooks."),
        TownProject::QuartermasterLedger => Some("Requires Oilcloth Satchel."),
        TownProject::ReinforcedPack => Some("Requires Quartermaster Ledger."),
        TownProject::StitchedPockets => Some("Requires Reinforced Pack."),
        TownProject::DeepRucksack => Some("Requires Stitched Pockets."),
        TownProject::ExilesTrunk => Some("Requires Deep Rucksack."),
        _ => None,
    }
}

pub(crate) fn town_project_availability(
    c: &Character,
    project: TownProject,
) -> ProjectAvailability {
    if has_completed_project(c, project) {
        return ProjectAvailability::Completed;
    }
    match project {
        TownProject::StorehouseShelves
        | TownProject::PackHooks
        | TownProject::OilclothSatchel
        | TownProject::QuartermasterLedger
        | TownProject::ReinforcedPack
        | TownProject::StitchedPockets
        | TownProject::DeepRucksack
        | TownProject::ExilesTrunk => {
            if let Some(previous) = previous_bag_project(project) {
                if has_completed_project(c, previous) {
                    ProjectAvailability::Available
                } else {
                    ProjectAvailability::Locked(
                        bag_project_lock_reason(project).expect("bag project has lock reason"),
                    )
                }
            } else {
                ProjectAvailability::Available
            }
        }
        TownProject::RebuildForge | TownProject::HireAppraiser => ProjectAvailability::Available,
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

pub(crate) fn town_project_status_text(c: &Character, project: TownProject) -> String {
    match town_project_availability(c, project) {
        ProjectAvailability::Available => "Available".to_string(),
        ProjectAvailability::Completed => "Complete".to_string(),
        ProjectAvailability::Locked(reason) => format!("Locked: {reason}"),
    }
}

pub(crate) fn town_project_row_text(c: &Character, project: TownProject) -> String {
    let definition = town_project_definition(project);
    format!(
        "[{}] {} - {} gold - {} - {}",
        definition.group,
        definition.name,
        definition.cost,
        town_project_status_text(c, project),
        definition.benefit
    )
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
    if is_bag_upgrade_project(project) {
        resize_bag_for_projects(c);
    }
    format!("Completed project: {}.", definition.name)
}

#[cfg(test)]
pub(crate) fn complete_project_for_test(c: &mut Character, project: TownProject) {
    if !has_completed_project(c, project) {
        c.completed_town_projects.push(project);
        if is_bag_upgrade_project(project) {
            resize_bag_for_projects(c);
        }
    }
}
