use std::{
    any::Any,
    sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use winit::event::WindowEvent;

use super::{entity::Entity, scene::Scene};

pub trait Component: Any + Send + Sync + 'static {
    fn get_entity(&self) -> Option<Arc<RwLock<Entity>>>;
    fn set_entity(&mut self, entity: &Arc<RwLock<Entity>>);

    fn on_start(&mut self, scene: &mut Scene);
    fn on_update(&mut self, scene: &mut Scene);
    fn on_event(&mut self, scene: &mut Scene, event: &WindowEvent);
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
        self.inner.read()
    }

    pub fn get_mut<'a>(&'a mut self) -> LockResult<RwLockWriteGuard<'a, C>> {
        self.inner.write()
    }
}

impl DynComponentRef {
    pub fn new<C: Component>(component: C) -> Self {
        let inner = Arc::new(RwLock::new(component));
        Self { inner }
    }

    pub fn on_start(&self, scene: &mut Scene) {
        let mut inner = self.inner.write().unwrap();
        inner.on_start(scene);
    }

    pub fn on_update(&self, scene: &mut Scene) {
        let mut inner = self.inner.write().unwrap();
        inner.on_start(scene);
    }

    pub fn on_event(&self, scene: &mut Scene, event: &WindowEvent) {
        let mut inner = self.inner.write().unwrap();
        inner.on_event(scene, event);
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
