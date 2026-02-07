use crate::campaign::upgrades::TechTree;
use crate::ecs::components::*;
use crate::ecs::entity::EntityId;
use crate::ecs::world::World;

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    LaunchInterceptor {
        battery_id: u32,
        target_x: f32,
        target_y: f32,
        interceptor_type: InterceptorType,
    },
}

/// Process queued player commands: spawn interceptors from batteries.
/// Uses tech_tree.effective_profile() for physics values so upgrades apply.
/// Returns the number of interceptors successfully launched this tick.
pub fn run(world: &mut World, commands: &mut Vec<PlayerCommand>, battery_ids: &[EntityId], tech_tree: &TechTree) -> u32 {
    let cmds: Vec<PlayerCommand> = std::mem::take(commands);
    let mut launched = 0u32;

    for cmd in cmds {
        match cmd {
            PlayerCommand::LaunchInterceptor {
                battery_id,
                target_x,
                target_y,
                interceptor_type,
            } => {
                let Some(&bat_eid) = battery_ids.get(battery_id as usize) else {
                    continue;
                };
                if !world.is_alive(bat_eid) {
                    continue;
                }
                let bat_idx = bat_eid.index as usize;

                // Check ammo
                let has_ammo = world.battery_states[bat_idx]
                    .as_ref()
                    .is_some_and(|b| b.ammo > 0);
                if !has_ammo {
                    continue;
                }

                // Decrement ammo
                if let Some(ref mut bs) = world.battery_states[bat_idx] {
                    bs.ammo -= 1;
                }

                // Get battery position
                let bat_pos = match world.transforms[bat_idx] {
                    Some(t) => t,
                    None => continue,
                };

                // Look up physics profile (with upgrades applied)
                let profile = tech_tree.effective_profile(interceptor_type);

                // Calculate initial direction toward target
                let dx = target_x - bat_pos.x;
                let dy = target_y - bat_pos.y;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let dir_x = dx / dist;
                let dir_y = dy / dist;

                // Spawn interceptor entity
                let id = world.spawn();
                let idx = id.index as usize;

                world.transforms[idx] = Some(Transform {
                    x: bat_pos.x,
                    y: bat_pos.y,
                    rotation: dir_y.atan2(dir_x),
                });

                // Small initial velocity in target direction
                world.velocities[idx] = Some(Velocity {
                    vx: dir_x * 10.0,
                    vy: dir_y * 10.0,
                });

                world.interceptors[idx] = Some(Interceptor {
                    interceptor_type,
                    thrust: profile.thrust,
                    burn_time: profile.burn_time,
                    burn_remaining: profile.burn_time,
                    ceiling: profile.ceiling,
                    battery_id,
                    target_x,
                    target_y,
                    proximity_fuse_radius: profile.proximity_fuse_radius,
                });

                world.ballistics[idx] = Some(Ballistic {
                    drag_coefficient: profile.drag_coeff,
                    mass: profile.mass,
                    cross_section: profile.cross_section,
                });

                world.warheads[idx] = Some(Warhead {
                    yield_force: profile.yield_force,
                    blast_radius_base: profile.blast_radius,
                    warhead_type: WarheadType::Standard,
                });

                world.markers[idx] = Some(EntityMarker {
                    kind: EntityKind::Interceptor,
                });

                launched += 1;
            }
        }
    }

    launched
}
