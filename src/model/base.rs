use crate::{
    component::Component,
    entity::transform::TransformComponent,
    model::{Material, Mesh},
    Camera,
};

use super::super::{
    error::*,
    event::{OnEventContext, OnStartContext, OnUpdateContext},
    scene::Scene,
};

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Component for Model {
    fn on_start(&mut self, _scene: &mut Scene, _context: OnStartContext) {}

    fn on_update(&mut self, _scene: &mut Scene, _context: OnUpdateContext) {}

    fn on_event(&mut self, _scene: &mut Scene, _context: OnEventContext) {}
}
