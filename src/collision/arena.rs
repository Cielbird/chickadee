use crate::{component::ComponentRef, entity::EntityId, Collider, TransformComponent};

pub struct CollisionArena {
    static_colliders: Vec<ColliderInfo>,
    dynamic_colliders: Vec<ColliderInfo>,
}

struct ColliderInfo(Collider, EntityId, ComponentRef<TransformComponent>);

impl CollisionArena {
    pub fn new() -> Self {
        Self {
            static_colliders: vec![],
            dynamic_colliders: vec![],
        }
    }

    pub fn collider_pass(&mut self) {
        let num_dynamic = self.dynamic_colliders.len();
        let num_static = self.static_colliders.len();
        if !(num_dynamic >= 1 && (num_static + num_dynamic) >= 2) {
            // no collisions to happen
            return;
        }

        for (a_idx, a_collider) in self.dynamic_colliders.iter().enumerate() {
            let ColliderInfo(col_a, a, a_trans) = a_collider;
            let mut a_trans = a_trans.clone();
            let mut a_trans = a_trans.write().unwrap();

            for b_collider in self.dynamic_colliders.iter().skip(a_idx + 1) {
                let ColliderInfo(col_b, b, b_trans) = b_collider;
                if a == b {
                    // an entity cannot collide with itself
                    continue;
                }

                let mut b_trans = b_trans.clone();
                let mut b_trans = b_trans.write().unwrap();

                let vec = Collider::get_correction_vec(
                    &col_a,
                    &a_trans.global(),
                    &col_b,
                    &b_trans.global(),
                );

                match vec {
                    Some(vec) => {
                        // a and b are both dynamic
                        a_trans.translate_global(vec / 2.);
                        b_trans.translate_global(-vec / 2.);
                    }
                    // no collision
                    None => continue,
                }
            }

            for b_collider in self.static_colliders.iter() {
                let ColliderInfo(col_b, b, b_trans) = b_collider;
                if a == b {
                    // an entity cannot collide with itself
                    continue;
                }

                let b_trans = b_trans.clone();
                let b_trans = b_trans.read().unwrap();

                let vec = Collider::get_correction_vec(
                    &col_a,
                    &a_trans.global(),
                    &col_b,
                    &b_trans.global(),
                );

                match vec {
                    Some(vec) => {
                        // only a is dynamic
                        a_trans.translate_global(vec);
                    }
                    // no collision
                    None => continue,
                }
            }
        }
    }

    pub(crate) fn add_collider(
        &mut self,
        entity: EntityId,
        collider: Collider,
        transform: ComponentRef<TransformComponent>,
    ) {
        if collider.dynamic() {
            self.dynamic_colliders
                .push(ColliderInfo(collider, entity, transform))
        } else {
            self.static_colliders
                .push(ColliderInfo(collider, entity, transform))
        }
    }
}
