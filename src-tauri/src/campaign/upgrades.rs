use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ecs::components::InterceptorType;
use crate::engine::config::{self, InterceptorProfile};

/// Max upgrade level for any axis.
pub const MAX_UPGRADE_LEVEL: u32 = 3;

/// Per-level multiplier for thrust upgrades (+15% per level).
pub const THRUST_UPGRADE_MULT: f32 = 0.15;
/// Per-level multiplier for blast radius upgrades (+20% per level).
pub const YIELD_UPGRADE_MULT: f32 = 0.20;
/// Per-level multiplier for guidance (proximity detonation) upgrades (+25% per level).
pub const GUIDANCE_UPGRADE_MULT: f32 = 0.25;

/// Unlock requirements: (wave_number_min, resource_cost).
pub fn unlock_gate(itype: InterceptorType) -> (u32, u32) {
    match itype {
        InterceptorType::Standard => (1, 0),
        InterceptorType::Sprint => (8, 200),
        InterceptorType::Exoatmospheric => (15, 300),
        InterceptorType::AreaDenial => (22, 400),
    }
}

/// Cost for a given upgrade axis at a given current level.
/// Returns None if already at max level.
pub fn upgrade_cost(axis: UpgradeAxis, current_level: u32) -> Option<u32> {
    if current_level >= MAX_UPGRADE_LEVEL {
        return None;
    }
    let base = match axis {
        UpgradeAxis::Thrust => [50, 100, 150],
        UpgradeAxis::Yield => [60, 120, 180],
        UpgradeAxis::Guidance => [40, 80, 120],
    };
    Some(base[current_level as usize])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpgradeAxis {
    Thrust,
    Yield,
    Guidance,
}

impl UpgradeAxis {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "thrust" => Self::Thrust,
            "yield" => Self::Yield,
            "guidance" => Self::Guidance,
            _ => Self::Thrust,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Thrust => "thrust",
            Self::Yield => "yield",
            Self::Guidance => "guidance",
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TypeUpgrades {
    pub thrust_level: u32,
    pub yield_level: u32,
    pub guidance_level: u32,
}

impl TypeUpgrades {
    pub fn level_for(&self, axis: UpgradeAxis) -> u32 {
        match axis {
            UpgradeAxis::Thrust => self.thrust_level,
            UpgradeAxis::Yield => self.yield_level,
            UpgradeAxis::Guidance => self.guidance_level,
        }
    }

    pub fn set_level(&mut self, axis: UpgradeAxis, level: u32) {
        match axis {
            UpgradeAxis::Thrust => self.thrust_level = level,
            UpgradeAxis::Yield => self.yield_level = level,
            UpgradeAxis::Guidance => self.guidance_level = level,
        }
    }
}

/// The tech tree tracks which interceptor types are unlocked and their upgrade levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechTree {
    pub unlocked_types: Vec<InterceptorType>,
    pub upgrades: HashMap<InterceptorType, TypeUpgrades>,
}

impl Default for TechTree {
    fn default() -> Self {
        let mut upgrades = HashMap::new();
        upgrades.insert(InterceptorType::Standard, TypeUpgrades::default());
        Self {
            unlocked_types: vec![InterceptorType::Standard],
            upgrades,
        }
    }
}

impl TechTree {
    /// Check if a type can be unlocked at the given wave number with available resources.
    pub fn can_unlock(&self, itype: InterceptorType, wave_number: u32, resources: u32) -> bool {
        if self.unlocked_types.contains(&itype) {
            return false;
        }
        let (min_wave, cost) = unlock_gate(itype);
        wave_number >= min_wave && resources >= cost
    }

    /// Unlock an interceptor type. Returns the cost, or an error.
    pub fn unlock(&mut self, itype: InterceptorType, wave_number: u32, resources: u32) -> Result<u32, String> {
        if self.unlocked_types.contains(&itype) {
            return Err("Type already unlocked".into());
        }
        let (min_wave, cost) = unlock_gate(itype);
        if wave_number < min_wave {
            return Err(format!("Requires wave {}, currently at wave {}", min_wave, wave_number));
        }
        if resources < cost {
            return Err(format!("Insufficient resources: have {}, need {}", resources, cost));
        }
        self.unlocked_types.push(itype);
        self.upgrades.insert(itype, TypeUpgrades::default());
        Ok(cost)
    }

    /// Apply an upgrade to a type. Returns the cost, or an error.
    pub fn apply_upgrade(&mut self, itype: InterceptorType, axis: UpgradeAxis, resources: u32) -> Result<u32, String> {
        if !self.unlocked_types.contains(&itype) {
            return Err("Type not unlocked".into());
        }
        let upgrades = self.upgrades.get(&itype).cloned().unwrap_or_default();
        let current = upgrades.level_for(axis);
        let cost = upgrade_cost(axis, current)
            .ok_or_else(|| format!("{} already at max level", axis.as_str()))?;
        if resources < cost {
            return Err(format!("Insufficient resources: have {}, need {}", resources, cost));
        }
        let entry = self.upgrades.entry(itype).or_default();
        entry.set_level(axis, current + 1);
        Ok(cost)
    }

    /// Get the effective interceptor profile with upgrades applied.
    pub fn effective_profile(&self, itype: InterceptorType) -> InterceptorProfile {
        let base = config::interceptor_profile(itype);
        let upgrades = self.upgrades.get(&itype);

        match upgrades {
            None => base,
            Some(u) => InterceptorProfile {
                thrust: base.thrust * (1.0 + u.thrust_level as f32 * THRUST_UPGRADE_MULT),
                burn_time: base.burn_time,
                ceiling: base.ceiling,
                mass: base.mass,
                drag_coeff: base.drag_coeff,
                cross_section: base.cross_section,
                yield_force: base.yield_force,
                blast_radius: base.blast_radius * (1.0 + u.yield_level as f32 * YIELD_UPGRADE_MULT),
            },
        }
    }

    pub fn is_unlocked(&self, itype: InterceptorType) -> bool {
        self.unlocked_types.contains(&itype)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_only_standard() {
        let tree = TechTree::default();
        assert_eq!(tree.unlocked_types.len(), 1);
        assert!(tree.is_unlocked(InterceptorType::Standard));
        assert!(!tree.is_unlocked(InterceptorType::Sprint));
    }

    #[test]
    fn sprint_unlockable_at_wave_8() {
        let tree = TechTree::default();
        assert!(!tree.can_unlock(InterceptorType::Sprint, 7, 1000));
        assert!(tree.can_unlock(InterceptorType::Sprint, 8, 200));
        assert!(!tree.can_unlock(InterceptorType::Sprint, 8, 199));
    }

    #[test]
    fn unlock_costs_resources() {
        let mut tree = TechTree::default();
        let cost = tree.unlock(InterceptorType::Sprint, 8, 200).unwrap();
        assert_eq!(cost, 200);
        assert!(tree.is_unlocked(InterceptorType::Sprint));
    }

    #[test]
    fn unlock_fails_if_already_unlocked() {
        let mut tree = TechTree::default();
        tree.unlock(InterceptorType::Sprint, 8, 200).unwrap();
        assert!(tree.unlock(InterceptorType::Sprint, 8, 200).is_err());
    }

    #[test]
    fn thrust_upgrade_gives_15_pct_per_level() {
        let mut tree = TechTree::default();
        let base = config::interceptor_profile(InterceptorType::Standard);

        tree.apply_upgrade(InterceptorType::Standard, UpgradeAxis::Thrust, 50).unwrap();
        let p1 = tree.effective_profile(InterceptorType::Standard);
        assert!((p1.thrust - base.thrust * 1.15).abs() < 0.01);

        tree.apply_upgrade(InterceptorType::Standard, UpgradeAxis::Thrust, 100).unwrap();
        let p2 = tree.effective_profile(InterceptorType::Standard);
        assert!((p2.thrust - base.thrust * 1.30).abs() < 0.01);
    }

    #[test]
    fn max_level_is_3() {
        let mut tree = TechTree::default();
        tree.apply_upgrade(InterceptorType::Standard, UpgradeAxis::Thrust, 50).unwrap();
        tree.apply_upgrade(InterceptorType::Standard, UpgradeAxis::Thrust, 100).unwrap();
        tree.apply_upgrade(InterceptorType::Standard, UpgradeAxis::Thrust, 150).unwrap();
        assert!(tree.apply_upgrade(InterceptorType::Standard, UpgradeAxis::Thrust, 999).is_err());
    }

    #[test]
    fn effective_profile_reflects_upgrades() {
        let mut tree = TechTree::default();
        let base = config::interceptor_profile(InterceptorType::Standard);

        tree.apply_upgrade(InterceptorType::Standard, UpgradeAxis::Yield, 60).unwrap();
        let p = tree.effective_profile(InterceptorType::Standard);

        // Blast radius should increase by 20%
        assert!((p.blast_radius - base.blast_radius * 1.20).abs() < 0.01);
        // Thrust unchanged
        assert_eq!(p.thrust, base.thrust);
    }

    #[test]
    fn upgrade_fails_on_locked_type() {
        let tree_default = TechTree::default();
        let mut tree = tree_default;
        assert!(tree.apply_upgrade(InterceptorType::Sprint, UpgradeAxis::Thrust, 999).is_err());
    }
}
