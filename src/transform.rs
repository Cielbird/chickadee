use cgmath::{EuclideanSpace, InnerSpace, Matrix3, One, Rad, Rotation3};

use cgmath::Transform as CgTransform;

use crate::Matrix4;
use crate::Vector3;

///
/// +z is out of the screen
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: cgmath::Point3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
}

pub enum TransformSpace {
    Local,
    Global,
}

impl Transform {
    #[allow(unused)]
    pub fn set_euler_rotation(&mut self, euler: Vector3) {
        let x_rotation =
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), Rad(euler.x));
        let y_rotation =
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), Rad(euler.y));
        let z_rotation =
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), Rad(euler.z));

        self.rotation = z_rotation * y_rotation * x_rotation;
    }

    /// The identity transform: T(x) = x
    pub fn identity() -> Self {
        Self {
            position: cgmath::point3(0., 0., 0.),
            rotation: cgmath::Quaternion::one(),
            scale: cgmath::Vector3::new(1., 1., 1.),
        }
    }

    #[allow(unused)]
    pub fn forward(&self) -> Vector3 {
        let rotation = cgmath::Matrix4::from(self.rotation);
        rotation.z.truncate()
    }

    pub fn translate(&mut self, translation: Vector3, space: TransformSpace) {
        match space {
            TransformSpace::Local => {
                self.translate_local(translation);
            }
            TransformSpace::Global => {
                self.translate_global(translation);
            }
        }
    }

    /// Translate the transform with global coordinates
    pub fn translate_global(&mut self, translation: Vector3) {
        self.position += translation;
    }

    /// Translate the transform relative to its rotation
    pub fn translate_local(&mut self, translation: Vector3) {
        // TODO take into account scale..?
        let translation = self.rotation_matrix()
            * Matrix4::from_translation(translation)
            * self.rotation_matrix().inverse_transform().unwrap();
        self.position = translation.transform_point(self.position);
    }

    #[allow(unused)]
    pub fn rotate_euler_local(&mut self, euler: Vector3) {
        let x_rotation = cgmath::Quaternion::from_angle_x(Rad(-euler.x));
        let y_rotation = cgmath::Quaternion::from_angle_y(Rad(-euler.y));
        let z_rotation = cgmath::Quaternion::from_angle_z(Rad(-euler.z));

        let rotation = z_rotation * y_rotation * x_rotation;
        self.rotation = self.rotation * rotation;
    }

    pub fn rotate_euler_global(&mut self, euler: Vector3) {
        let x_rotation = cgmath::Quaternion::from_angle_x(Rad(-euler.x));
        let y_rotation = cgmath::Quaternion::from_angle_y(Rad(-euler.y));
        let z_rotation = cgmath::Quaternion::from_angle_z(Rad(-euler.z));

        let rotation = z_rotation * y_rotation * x_rotation;
        self.rotation = rotation * self.rotation;
    }

    pub fn matrix(&self) -> Matrix4 {
        self.position_matrix() * self.scale_matrix() * self.rotation_matrix()
    }

    fn rotation_matrix(&self) -> Matrix4 {
        Matrix4::from(self.rotation)
    }

    fn scale_matrix(&self) -> Matrix4 {
        Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

    fn position_matrix(&self) -> Matrix4 {
        Matrix4::from_translation(self.position.to_vec())
    }

    #[allow(unused)]
    pub fn from_matrix(matrix: Matrix4) -> Self {
        // 1. Extract translation (global position)
        let translation = cgmath::Point3::from_homogeneous(matrix.w);

        // 2. Extract scale from the x, y, z columns (basis vectors)
        let scale_x = matrix.x.truncate().magnitude();
        let scale_y = matrix.y.truncate().magnitude();
        let scale_z = matrix.z.truncate().magnitude();
        let scale = Vector3::new(scale_x, scale_y, scale_z);

        // 3. Remove scale from the 3x3 portion to get the pure rotation matrix
        let rotation_matrix = Matrix3::from_cols(
            matrix.x.truncate() / scale_x,
            matrix.y.truncate() / scale_y,
            matrix.z.truncate() / scale_z,
        );
        // Convert the rotation matrix into a quaternion
        let rotation = cgmath::Quaternion::from(rotation_matrix);

        Self {
            position: translation,
            rotation,
            scale,
        }
    }
}

impl std::ops::Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        let out_mat = self.matrix() * rhs.matrix();
        Transform::from_matrix(out_mat)
    }
}
