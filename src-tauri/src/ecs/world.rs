use super::components::*;
use super::entity::{EntityAllocator, EntityId};

/// SoA (Struct of Arrays) ECS World.
/// Each component type has its own Vec<Option<T>> storage, indexed by entity index.
pub struct World {
    pub allocator: EntityAllocator,
    alive: Vec<bool>,

    // Component storage — one Vec per component type
    pub transforms: Vec<Option<Transform>>,
    pub velocities: Vec<Option<Velocity>>,
    pub ballistics: Vec<Option<Ballistic>>,
    pub warheads: Vec<Option<Warhead>>,
    pub interceptors: Vec<Option<Interceptor>>,
    pub lifetimes: Vec<Option<Lifetime>>,
    pub healths: Vec<Option<Health>>,
    pub reentry_glows: Vec<Option<ReentryGlow>>,
    pub shockwaves: Vec<Option<Shockwave>>,
    pub markers: Vec<Option<EntityMarker>>,
    pub battery_states: Vec<Option<BatteryState>>,
    pub mirv_carriers: Vec<Option<MirvCarrier>>,
    pub detected: Vec<Option<Detected>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
            alive: Vec::new(),
            transforms: Vec::new(),
            velocities: Vec::new(),
            ballistics: Vec::new(),
            warheads: Vec::new(),
            interceptors: Vec::new(),
            lifetimes: Vec::new(),
            healths: Vec::new(),
            reentry_glows: Vec::new(),
            shockwaves: Vec::new(),
            markers: Vec::new(),
            battery_states: Vec::new(),
            mirv_carriers: Vec::new(),
            detected: Vec::new(),
        }
    }

    pub fn spawn(&mut self) -> EntityId {
        let id = self.allocator.allocate();
        let idx = id.index as usize;

        // Grow all storage to accommodate
        while self.alive.len() <= idx {
            self.alive.push(false);
            self.transforms.push(None);
            self.velocities.push(None);
            self.ballistics.push(None);
            self.warheads.push(None);
            self.interceptors.push(None);
            self.lifetimes.push(None);
            self.healths.push(None);
            self.reentry_glows.push(None);
            self.shockwaves.push(None);
            self.markers.push(None);
            self.battery_states.push(None);
            self.mirv_carriers.push(None);
            self.detected.push(None);
        }

        self.alive[idx] = true;
        id
    }

    pub fn despawn(&mut self, id: EntityId) {
        if !self.allocator.is_alive(id) {
            return;
        }
        let idx = id.index as usize;
        self.alive[idx] = false;
        self.transforms[idx] = None;
        self.velocities[idx] = None;
        self.ballistics[idx] = None;
        self.warheads[idx] = None;
        self.interceptors[idx] = None;
        self.lifetimes[idx] = None;
        self.healths[idx] = None;
        self.reentry_glows[idx] = None;
        self.shockwaves[idx] = None;
        self.markers[idx] = None;
        self.battery_states[idx] = None;
        self.mirv_carriers[idx] = None;
        self.detected[idx] = None;
        self.allocator.deallocate(id);
    }

    pub fn is_alive(&self, id: EntityId) -> bool {
        self.allocator.is_alive(id)
            && (id.index as usize) < self.alive.len()
            && self.alive[id.index as usize]
    }

    pub fn entity_count(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }

    /// Iterate over all alive entity indices
    pub fn alive_entities(&self) -> Vec<usize> {
        self.alive
            .iter()
            .enumerate()
            .filter_map(|(i, &alive)| if alive { Some(i) } else { None })
            .collect()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_and_despawn() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(world.is_alive(e));
        assert_eq!(world.entity_count(), 1);

        world.despawn(e);
        assert!(!world.is_alive(e));
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn component_storage() {
        let mut world = World::new();
        let e = world.spawn();
        let idx = e.index as usize;

        world.transforms[idx] = Some(Transform {
            x: 10.0,
            y: 20.0,
            rotation: 0.0,
        });
        world.velocities[idx] = Some(Velocity { vx: 1.0, vy: -2.0 });

        assert!(world.transforms[idx].is_some());
        assert!(world.velocities[idx].is_some());
        assert!(world.ballistics[idx].is_none());

        world.despawn(e);
        assert!(world.transforms[idx].is_none());
    }
}
