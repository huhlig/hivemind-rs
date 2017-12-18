///
///
///
///
///
///
///

type EntityID = usize;

pub struct EntityMap {
    next_suffix_id: usize,
    next_slot_id: usize,
    free_slot_list: Vec<usize>,
    entities: Vec<Optional<Entity>>,
    // $($compname: ComponentData<$comptype>),*
}

pub enum Entity {
    Present(C),
    Missing,
}


pub enum Component<C> {
    Present(C),
    Missing,
}

/// Component Data
pub struct ComponentType<C> {
    data: Vec<Component<C>>,
    free: Vec<usize>,
}

impl<C> ComponentType<C> {
    pub fn new() -> ComponentType<C> {
        ComponentType {
            data: Vec::new(),
            free: Vec::new(),
        }
    }
    pub fn add(&mut self, component: C) -> usize {
        if self.free.is_empty() {
            let idx = self.data.len();
            self.data.push(Component::Present(component));
            return idx;
        } else {
            let idx = self.free.remove(0);
            self.data[idx] = component;
            return idx;
        }
    }
    pub fn remove(&mut self, idx: usize) {
        if idx < self.data.len() {
            self.data[idx] = Component::Missing;
            self.free.push(idx);
        } else {
            // Error
        }
    }
}

#[macro_export]
macro_rules! ECS {{
    $($compname:ident: $comptype:ty),*
} => {
    $crate::EntityManager::new();

    pub struct EntityMap {
        next_unique_id: usize,
        next_slot_id: usize,
        free_slot_list: Vec<usize>,
        entities: Vec<Option<Entity>>,
    }


    /// Core Entity System
    //#[derive(Serialize, Deserialize)]
    pub struct EntityManager {
        next_entity_id: usize,
        entities: Vec<Entity>,
        $($compname: ComponentData<$comptype>),*
    }

    impl EntityManager {
        pub fn new() -> EntityManager {
            EntityManager {
                next_entity_id: 0,
                entities: vec![Entity],
                $($compname: ComponentData::new()),*
            }
        }
        pub fn create_entity(&mut self) -> EntityID {
            let uid = next_uid;
            next_uid += 1;
            entities.push(Entity{
                uid: uid,
                $($compname: Component::Missing),*
            });
            (uid as EntityID)
        }
        $(pub fn add_$compname_component(&mut self, eid: EntityID, $compname: $comptype) {
            self.$compname.date.push($compname)
        }),*
    }

    /// Internal Entity containing Component Indexes
    #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
    pub struct Entity {
        /// Entity ID
        uid: usize,
        $($compname: Component),*
    }

}}

#[cfg(test)]
mod tests {
    //#[derive(Serialize, Deserialize)]
    pub struct Position {
        x: i32,
        y: i32,
    }

    //#[derive(Serialize, Deserialize)]
    pub struct Physics {
        weight: usize
    }

    #[test]
    pub fn test_ecs() {
        let entity_manager = ECS!(
            physics: Physics,
            position: Position,
        );

        let entity = entity_manager.create_entity();
    }
}