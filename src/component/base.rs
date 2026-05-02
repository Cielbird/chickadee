use std::{
    any::{self, Any},
    sync::TryLockResult,
};

use crate::event::{OnEventContext, OnStartContext, OnUpdateContext};
use crate::{component::ComponentId, error::*};

use super::super::scene::Scene;

pub trait Component: Any + Send + Sync + 'static {
    fn on_start(&mut self, scene: &mut Scene, context: OnStartContext);
    fn on_update(&mut self, scene: &mut Scene, context: OnUpdateContext);
    fn on_event(&mut self, scene: &mut Scene, context: OnEventContext);
}

pub struct DynComponentRef {
    type_id: any::TypeId,
    id: ComponentId,
    inner: Box<dyn Component>,
}

impl DynComponentRef {
    pub fn new<C: Component>(component: C) -> Self {
        let type_id = any::TypeId::of::<C>();
        let id = ComponentId::new();
        let inner = Box::new(component);
        Self { type_id, id, inner }
    }

    pub fn try_on_start(
        &mut self,
        scene: &mut Scene,
        context: OnStartContext,
    ) -> TryLockResult<()> {
        self.inner.on_start(scene, context);
        Ok(())
    }

    pub fn try_on_update(
        &mut self,
        scene: &mut Scene,
        context: OnUpdateContext,
    ) -> TryLockResult<()> {
        self.inner.on_update(scene, context);
        Ok(())
    }

    pub fn try_on_event(
        &mut self,
        scene: &mut Scene,
        context: OnEventContext,
    ) -> TryLockResult<()> {
        self.inner.on_event(scene, context);
        Ok(())
    }

    pub fn downcast_mut<C: Component>(&mut self) -> Result<&mut C> {
        let is_type_match = self.type_id == any::TypeId::of::<C>();
        if is_type_match {
            Ok(unsafe { self.downcast_mut_unchecked() })
        } else {
            Err(crate::Error::ComponentDowncastError)
        }
    }

    pub fn downcast_ref<C: Component>(&self) -> Result<&C> {
        let is_type_match = self.type_id == any::TypeId::of::<C>();
        if is_type_match {
            Ok(unsafe { self.downcast_ref_unchecked() })
        } else {
            Err(crate::Error::ComponentDowncastError)
        }
    }

    unsafe fn downcast_mut_unchecked<C: Component>(&mut self) -> &mut C {
        // SAFETY: caller guarantees type_id matches C, so this reinterpretation is valid
        &mut *(self.inner.as_mut() as *mut dyn Component as *mut C)
    }

    unsafe fn downcast_ref_unchecked<C: Component>(&self) -> &C {
        // SAFETY: caller guarantees type_id matches C, so this reinterpretation is valid
        &*(self.inner.as_ref() as *const dyn Component as *const C)
    }

    pub fn id(&self) -> ComponentId {
        self.id.clone()
    }
}
