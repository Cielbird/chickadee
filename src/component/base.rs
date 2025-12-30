use std::{
    any::Any,
    sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::event::{OnEventContext, OnStartContext, OnUpdateContext};

use super::super::scene::Scene;

pub trait Component: Any + Send + Sync + 'static {
    fn on_start(&mut self, scene: &mut Scene, context: OnStartContext);
    fn on_update(&mut self, scene: &mut Scene, context: OnUpdateContext);
    fn on_event(&mut self, scene: &mut Scene, context: OnEventContext);
}

#[derive(Clone)]
pub struct DynComponentRef {
    inner: Arc<RwLock<dyn Component>>,
}

#[derive(Clone)]
pub struct ComponentRef<C: Component> {
    inner: Arc<RwLock<C>>,
}

impl<C: Component> ComponentRef<C> {
    pub fn get_ref<'a>(&'a self) -> LockResult<RwLockReadGuard<'a, C>> {
        // TODO use AsRef trait
        self.inner.read()
    }

    #[allow(unused)]
    pub fn get_mut<'a>(&'a mut self) -> LockResult<RwLockWriteGuard<'a, C>> {
        // TODO use AsMut trait
        self.inner.write()
    }
}

impl DynComponentRef {
    pub fn new<C: Component>(component: C) -> Self {
        let inner = Arc::new(RwLock::new(component));
        Self { inner }
    }

    pub fn on_start(&self, scene: &mut Scene, context: OnStartContext) {
        let mut inner = self.inner.write().unwrap();
        inner.on_start(scene, context);
    }

    pub fn on_update(&self, scene: &mut Scene, context: OnUpdateContext) {
        let mut inner = self.inner.write().unwrap();
        inner.on_update(scene, context);
    }

    pub fn on_event(&self, scene: &mut Scene, context: OnEventContext) {
        let mut inner = self.inner.write().unwrap();
        inner.on_event(scene, context);
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
    type Error = String;

    fn try_into(self) -> Result<ComponentRef<C>, Self::Error> {
        let is_type_match = {
            let inner = self.inner.read().unwrap();
            (&*inner as &dyn Any).is::<C>()
        };
        if is_type_match {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(format!("Can't downcast to type C!"))
        }
    }
}
