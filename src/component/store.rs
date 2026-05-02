use crate::error::*;
use std::collections::HashMap;

use crate::{
    component::{ComponentId, DynComponentRef},
    Component,
};

pub struct ComponentStore {
    components: HashMap<ComponentId, DynComponentRef>,
}

impl ComponentStore {
    pub fn insert<C: Component>(&mut self, component: C) -> Result<ComponentId> {
        let component_ref = DynComponentRef::new(component);
        let component_id = component_ref.id();
        self.components.insert(component_id.clone(), component_ref);

        Ok(component_id)
    }

    pub fn swap(
        &mut self,
        id: &ComponentId,
        component: Option<DynComponentRef>,
    ) -> Option<DynComponentRef> {
        match component {
            Some(component) => self.components.insert(id.clone(), component),
            None => self.components.remove(id),
        }
    }

    pub fn get_mut<C: Component>(&mut self, id: &ComponentId) -> Option<&mut C> {
        match self.components.get_mut(id).map(|x| (*x).downcast_mut())? {
            Ok(comp) => Some(comp),
            Err(_) => None,
        }
    }

    pub fn get_ref<C: Component>(&self, id: &ComponentId) -> Option<&C> {
        match self.components.get(id).map(|x| (*x).downcast_ref())? {
            Ok(comp) => Some(comp),
            Err(_) => None,
        }
    }

    pub fn get_mut_disjoint_2<C1: Component, C2: Component>(&mut self, ids: [&ComponentId; 2]) -> (Option<&mut C1>, Option<&mut C2>) {
        let [c1, c2] = self.components.get_disjoint_mut(ids);
        let c1 = c1.map(|x| (*x).downcast_mut::<C1>().unwrap());
        let c2 = c2.map(|x| (*x).downcast_mut::<C2>().unwrap());
        (c1, c2)
    }

    pub fn get_mut_first<C: Component>(&mut self) -> Option<&mut C> {
        for (_, comp_ref) in &mut self.components {
            if let Ok(x) = comp_ref.downcast_mut() {
                return Some(x);
            }
        }

        None
    }

    pub fn get_ref_first<C: Component>(&self) -> Option<&C> {
        for (_, comp_ref) in &self.components {
            if let Ok(x) = comp_ref.downcast_ref() {
                return Some(x);
            }
        }

        None
    }

    pub fn get_id_first<C: Component>(&self) -> Option<ComponentId> {
        for (id, comp_ref) in &self.components {
            if let Ok(_) = comp_ref.downcast_ref::<C>() {
                return Some(id.clone());
            }
        }

        None
    }

    

    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
}
