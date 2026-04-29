use cgmath::vec3;

use crate::{transform::Transform, Component, Vector3};

/// Axis-aligned bounding box : fast and simple
#[derive(Debug)]
pub struct AxisAlignedBoundingBox {
    // the minimum x, y, and z positions
    pub min: Vector3,
    // the maximum x, y, and z positions
    pub max: Vector3,
}

impl AxisAlignedBoundingBox {
    pub fn new(min: Vector3, max: Vector3) -> Self {
        Self { min, max }
    }

    /// Get if the two boxes are overlapping
    pub fn contains_aabb(
        &self,
        transform: &Transform,
        other: &AxisAlignedBoundingBox,
        other_transform: &Transform,
    ) -> bool {
        // this is fastest
        Self::aabb_correction_vec(self, transform, other, other_transform).is_some()
    }

    /// Get the collision correction vector for this AABB if the other one is also a AABB
    pub fn aabb_correction_vec(
        a: &AxisAlignedBoundingBox,
        a_transform: &Transform,
        b: &AxisAlignedBoundingBox,
        b_transform: &Transform,
    ) -> Option<Vector3> {
        // TODO this doesn't take into account scale!
        let a_pos = a_transform.translation() + a.center();
        let b_pos = b_transform.translation() + b.center();

        let x_in_bounds;
        let dx;
        if a_pos.x > b_pos.x {
            dx = (b.max.x) - (a.min.x);
            x_in_bounds = dx > 0.;
        } else {
            dx = (b.min.x) - (a.max.x);
            x_in_bounds = dx < 0.;
        }

        let y_in_bounds;
        let dy;
        if a_pos.y > b_pos.y {
            dy = (b.max.y) - (a.min.y);
            y_in_bounds = dy > 0.;
        } else {
            dy = (b.min.y) - (a.max.y);
            y_in_bounds = dy < 0.;
        }

        let z_in_bounds;
        let dz;
        if a_pos.z > b_pos.z {
            dz = (b.max.z) - (a.min.z);
            z_in_bounds = dz > 0.;
        } else {
            dz = (b.min.z) - (a.max.z);
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

    fn center(&self) -> Vector3 {
        (self.max + self.min) / 2.
    }
}

// colliders could be treated seperately
impl Component for AxisAlignedBoundingBox {
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
        let box_a = AxisAlignedBoundingBox::new(vec3(0., 0., 0.), vec3(1., 1., 1.));
        let a_transform = Transform::from_translation(vec3(0., 1., 0.));
        let box_b = AxisAlignedBoundingBox::new(vec3(0., 0., 0.), vec3(1.5, 1., 1.));
        let b_transform = Transform::from_translation(vec3(1., 1., 1.));

        let vec =
            AxisAlignedBoundingBox::aabb_correction_vec(&box_a, &a_transform, &box_b, &b_transform)
                .unwrap();

        assert_eq!(vec, vec3(0., 0., -1.));
    }
}
