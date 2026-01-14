use cgmath::num_traits::zero;
use cgmath::{InnerSpace, Matrix, Matrix3, Matrix4, Rad, SquareMatrix, Zero};

use crate::Vector3;

/// An orthonormal transform
/// +z is out of the screen
/// Represented by a 4x4 homogenous transformation matrix:
/// a_00    a_01    a_02    t_0
/// a_10    a_11    a_12    t_1
/// a_20    a_21    a_22    t_2
/// 0       0       0       1
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Represents the top left 3x3 matrix: rotation and scale
    a: cgmath::Matrix3<f32>,
    /// represents the translation
    t: cgmath::Vector3<f32>,
}

impl Transform {
    /// The identity transform: T(x) = x
    pub fn identity() -> Self {
        Self {
            a: cgmath::Matrix3::identity(),
            t: cgmath::Vector3::zero(),
        }
        .check_invariants()
    }

    pub fn from_angle_x(theta: f32) -> Self {
        Self {
            a: Matrix3::from_angle_x(Rad(theta)),
            t: zero(),
        }
        .check_invariants()
    }

    pub fn from_angle_y(theta: f32) -> Self {
        Self {
            a: Matrix3::from_angle_y(Rad(theta)),
            t: zero(),
        }
        .check_invariants()
    }

    pub fn from_angle_z(theta: f32) -> Self {
        Self {
            a: Matrix3::from_angle_z(Rad(theta)),
            t: zero(),
        }
        .check_invariants()
    }

    pub fn inverse(self) -> Self {
        // a should always be orthonormal
        let a = self.a.transpose();

        let t = a * -self.t;
        Self { a, t }.check_invariants()
    }

    pub fn as_matrix(self) -> cgmath::Matrix4<f32> {
        let c0 = self.a.x;
        let c1 = self.a.y;
        let c2 = self.a.z;
        let c3 = self.t;

        Matrix4::new(
            c0.x, c0.y, c0.z, 0., c1.x, c1.y, c1.z, 0., c2.x, c2.y, c2.z, 0., c3.x, c3.y, c3.z, 1.,
        )
    }

    /// Get the translation of the transform
    pub fn translation(&self) -> Vector3 {
        self.t
    }

    /// Construct transformation that applies the `vec` as a translation
    pub fn from_translation(vec: cgmath::Vector3<f32>) -> Self {
        Self {
            a: Matrix3::identity(),
            t: vec,
        }
        .check_invariants()
    }

    /// Checks all the conditions that a transform must abide by:
    /// - The 3x3 `a` matrix should be orthonormal
    /// - No NaN values
    #[cfg(debug_assertions)]
    pub fn check_invariants(self) -> Self {
        if !self.a.is_finite() {
            panic!("Transform should not be infinite");
        } else {
            const EPS: f32 = 1e-3; // lets be leniant
            if approx::abs_diff_ne!(self.a.x.dot(self.a.y), 0., epsilon = EPS)
                || approx::abs_diff_ne!(self.a.y.dot(self.a.z), 0., epsilon = EPS)
                || approx::abs_diff_ne!(self.a.z.dot(self.a.x), 0., epsilon = EPS)
                || approx::abs_diff_ne!(self.a.x.magnitude2(), 1., epsilon = EPS)
            {
                panic!(
                    "`a` component of transform is not orthonormal: {:?}, {},{},{} should all be 0, and {} should be 1",
                    self.a, self.a.x.dot(self.a.y), self.a.y.dot(self.a.z),self.a.z.dot(self.a.x),self.a.x.magnitude2()
                );
            }
        }

        self
    }
}

impl std::ops::Mul<Transform> for Transform {
    type Output = Self;

    fn mul(self, rhs: Transform) -> Transform {
        // T = a + t
        let a = self.a * rhs.a;
        let t = self.a * rhs.t + self.t;

        Self { a, t }.check_invariants()
    }
}

impl std::ops::Mul<Vector3> for Transform {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Self::Output {
        self.a * rhs + self.t
    }
}
