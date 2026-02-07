use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    pub index: u32,
    pub generation: u32,
}

impl EntityId {
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{}g{}", self.index, self.generation)
    }
}

pub struct EntityAllocator {
    generations: Vec<u32>,
    free_indices: Vec<u32>,
    next_index: u32,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_indices: Vec::new(),
            next_index: 0,
        }
    }

    pub fn allocate(&mut self) -> EntityId {
        if let Some(index) = self.free_indices.pop() {
            let generation = self.generations[index as usize];
            EntityId::new(index, generation)
        } else {
            let index = self.next_index;
            self.next_index += 1;
            self.generations.push(0);
            EntityId::new(index, 0)
        }
    }

    pub fn deallocate(&mut self, id: EntityId) {
        if (id.index as usize) < self.generations.len()
            && self.generations[id.index as usize] == id.generation
        {
            self.generations[id.index as usize] += 1;
            self.free_indices.push(id.index);
        }
    }

    pub fn is_alive(&self, id: EntityId) -> bool {
        (id.index as usize) < self.generations.len()
            && self.generations[id.index as usize] == id.generation
    }

    /// Get the current generation for an index (used by cleanup to reconstruct EntityIds)
    pub fn generation_of(&self, index: u32) -> Option<u32> {
        self.generations.get(index as usize).copied()
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_and_deallocate() {
        let mut alloc = EntityAllocator::new();
        let e0 = alloc.allocate();
        assert_eq!(e0.index, 0);
        assert_eq!(e0.generation, 0);
        assert!(alloc.is_alive(e0));

        alloc.deallocate(e0);
        assert!(!alloc.is_alive(e0));

        let e0_reuse = alloc.allocate();
        assert_eq!(e0_reuse.index, 0);
        assert_eq!(e0_reuse.generation, 1);
        assert!(alloc.is_alive(e0_reuse));
        assert!(!alloc.is_alive(e0));
    }

    #[test]
    fn sequential_allocation() {
        let mut alloc = EntityAllocator::new();
        let e0 = alloc.allocate();
        let e1 = alloc.allocate();
        let e2 = alloc.allocate();
        assert_eq!(e0.index, 0);
        assert_eq!(e1.index, 1);
        assert_eq!(e2.index, 2);
    }
}
