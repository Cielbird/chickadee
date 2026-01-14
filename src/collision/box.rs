use cgmath::vec3;

use crate::{transform::Transform, Component, Vector3};

/// Axis-aligned box collider : fast and simple
pub struct SimpleBoxCollider {
    /// The center point of the collider, relative to the entity's transform
    position: Vector3,
    /// Dimensions of the collider: x y and z measure from the mid point to the edge
    dimensions: Vector3,
}

impl SimpleBoxCollider {
    pub fn new(position: Vector3, dimensions: Vector3) -> Self {
        Self {
            position,
            dimensions,
        }
    }

    /// Get the collision correction vector for this box if the other one is also a box
    pub fn box_correction_vec(
        a: &SimpleBoxCollider,
        a_transform: &Transform,
        b: &SimpleBoxCollider,
        b_transform: &Transform,
    ) -> Option<Vector3> {
        // TODO this doesn't take into account rotation or scale!
        let a_pos = a_transform.translation() + a.position;
        let b_pos = b_transform.translation() + b.position;
        let a_min = a_pos - a.dimensions;
        let a_max = a_pos + a.dimensions;
        let b_min = b_pos - b.dimensions;
        let b_max = b_pos + b.dimensions;

        let x_in_bounds;
        let dx;
        if a_pos.x > b_pos.x {
            dx = (b_max.x) - (a_min.x);
            x_in_bounds = dx > 0.;
        } else {
            dx = (b_min.x) - (a_max.x);
            x_in_bounds = dx < 0.;
        }

        let y_in_bounds;
        let dy;
        if a_pos.y > b_pos.y {
            dy = (b_max.y) - (a_min.y);
            y_in_bounds = dy > 0.;
        } else {
            dy = (b_min.y) - (a_max.y);
            y_in_bounds = dy < 0.;
        }

        let z_in_bounds;
        let dz;
        if a_pos.z > b_pos.z {
            dz = (b_max.z) - (a_min.z);
            z_in_bounds = dz > 0.;
        } else {
            dz = (b_min.z) - (a_max.z);
            z_in_bounds = dz < 0.;
        }

        if !x_in_bounds || !y_in_bounds || !z_in_bounds {
            None
        } else {
            // take the axis of smallest displacement
            if dx.abs() < dy.abs() && dx.abs() < dz.abs() {
                Some(vec3(dx, 0., 0.))
            } else if dy.abs() < dz.abs() {
                Some(vec3(0., dy, 0.))
            } else {
                Some(vec3(0., 0., dz))
            }
        }
    }
}

// colliders could be treated seperately
impl Component for SimpleBoxCollider {
    fn on_start(&mut self, _scene: &mut crate::Scene, _context: crate::OnStartContext) {}

    fn on_update(&mut self, _scene: &mut crate::Scene, _context: crate::OnUpdateContext) {}

    fn on_event(&mut self, _scene: &mut crate::Scene, _context: crate::OnEventContext) {}
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_box_corretion_vec() {
        let box_a = SimpleBoxCollider::new(vec3(0., 0., 0.), vec3(1., 1., 1.));
        let a_transform = Transform::from_translation(vec3(0., 1., 0.));
        let box_b = SimpleBoxCollider::new(vec3(0., 0., 0.), vec3(1.5, 1., 1.));
        let b_transform = Transform::from_translation(vec3(1., 1., 1.));

        let vec =
            SimpleBoxCollider::box_correction_vec(&box_a, &a_transform, &box_b, &b_transform).unwrap();

        assert_eq!(vec, vec3(0., 0., -1.));
    }
}
