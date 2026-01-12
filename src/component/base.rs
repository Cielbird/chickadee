use std::{
    any::{self, Any},
    sync::{
        Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError, TryLockResult,
    },
};

use crate::error::*;
use crate::event::{OnEventContext, OnStartContext, OnUpdateContext};

use super::super::scene::Scene;

pub trait Component: Any + Send + Sync + 'static {
    fn on_start(&mut self, scene: &mut Scene, context: OnStartContext);
    fn on_update(&mut self, scene: &mut Scene, context: OnUpdateContext);
    fn on_event(&mut self, scene: &mut Scene, context: OnEventContext);
}

#[derive(Clone)]
pub struct DynComponentRef {
    type_id: any::TypeId,
    inner: Arc<RwLock<dyn Component>>,
}

#[derive(Clone)]
pub struct ComponentRef<C: Component> {
    inner: Arc<RwLock<C>>,
}

impl<C: Component> ComponentRef<C> {
    pub fn read<'a>(&'a self) -> LockResult<RwLockReadGuard<'a, C>> {
        self.inner.read()
    }

    #[allow(unused)]
    pub fn write<'a>(&'a mut self) -> LockResult<RwLockWriteGuard<'a, C>> {
        self.inner.write()
    }

    #[allow(unused)]
    pub fn try_read<'a>(&'a mut self) -> TryLockResult<RwLockReadGuard<'a, C>> {
        self.inner.try_read()
    }

    #[allow(unused)]
    pub fn try_write<'a>(&'a mut self) -> TryLockResult<RwLockWriteGuard<'a, C>> {
        self.inner.try_write()
    }
}

impl DynComponentRef {
    pub fn new<C: Component>(component: C) -> Self {
        let type_id = any::TypeId::of::<C>();
        let inner = Arc::new(RwLock::new(component));
        Self { type_id, inner }
    }

    pub fn try_on_start(&self, scene: &mut Scene, context: OnStartContext) -> TryLockResult<()> {
        let mut inner = self.inner.try_write().map_err(|err| match err {
            TryLockError::Poisoned(_err) => panic!("Component lock poisoned!"),
            TryLockError::WouldBlock => TryLockError::<()>::WouldBlock,
        })?;
        inner.on_start(scene, context);
        Ok(())
    }

    pub fn try_on_update(&self, scene: &mut Scene, context: OnUpdateContext) -> TryLockResult<()> {
        let mut inner = self.inner.try_write().map_err(|err| match err {
            TryLockError::Poisoned(_err) => panic!("Component lock poisoned!"),
            TryLockError::WouldBlock => TryLockError::<()>::WouldBlock,
        })?;
        inner.on_update(scene, context);
        Ok(())
    }

    pub fn try_on_event(&self, scene: &mut Scene, context: OnEventContext) -> TryLockResult<()> {
        let mut inner = self.inner.try_write().map_err(|err| match err {
            TryLockError::Poisoned(_err) => panic!("Component lock poisoned!"),
            TryLockError::WouldBlock => TryLockError::<()>::WouldBlock,
        })?;
        inner.on_event(scene, context);
        Ok(())
    }

    pub fn downcast<C: Component>(self) -> Result<ComponentRef<C>> {
        let is_type_match = self.type_id == any::TypeId::of::<C>();
        if is_type_match {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(crate::Error::ComponentDowncastError)
        }
    }

    unsafe fn downcast_unchecked<C: Component>(self) -> ComponentRef<C> {
        let raw = Arc::into_raw(self.inner);
        let data = raw as *const RwLock<C>;

        ComponentRef::<C> {
            inner: Arc::from_raw(data),
        }
    }
}

impl<C: Component> TryInto<ComponentRef<C>> for DynComponentRef {
    type Error = crate::Error;

    fn try_into(self) -> Result<ComponentRef<C>> {
        self.downcast()
    }
}
