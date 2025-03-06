use cgmath::{EuclideanSpace, Euler, InnerSpace, Matrix, Matrix3, Matrix4, One, Point3, Quaternion, Rad, Rotation, Rotation3, Vector3, Zero};

use cgmath::Transform as CgTransform;

pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    fn set_euler_rotation(&mut self, euler: Vector3<f32>) {
        let x_rotation = Quaternion::from_axis_angle(Vector3::unit_x(), Rad(euler.x));
        let y_rotation = Quaternion::from_axis_angle(Vector3::unit_y(), Rad(euler.y));
        let z_rotation = Quaternion::from_axis_angle(Vector3::unit_z(), Rad(euler.z));

        self.rotation = z_rotation * y_rotation * x_rotation;
    }

    fn forward(&self) -> Vector3<f32> {
        let rotation = Matrix4::from(self.rotation);
        rotation.z.truncate()
    }

    fn move_global(&mut self, vector: Vector3<f32>) {
        self.position += vector;
    }

    pub fn move_local(&mut self, move_vec: Vector3<f32>) {
        //let local_vec = self.rotation_matrix().inverse_transform().unwrap() * Matrix4::from_translation(vector);
        let translation = self.rotation_matrix() * Matrix4::from_translation(move_vec) * self.rotation_matrix().inverse_transform().unwrap();
        self.position = translation.transform_point(self.position);
    }

    pub fn rotate_euler_local(&mut self, euler: Vector3<f32>) {
        let x_rotation = Quaternion::from_angle_x(Rad(-euler.x)); // FIXME negative is ultra sus, but it works
        let y_rotation = Quaternion::from_angle_y(Rad(-euler.y));
        let z_rotation = Quaternion::from_angle_z(Rad(-euler.z));

        let rotation = z_rotation * y_rotation * x_rotation;
        self.rotation = self.rotation * rotation;
    }

    pub fn rotate_euler_global(&mut self, euler: Vector3<f32>) {
        let x_rotation = Quaternion::from_angle_x(Rad(-euler.x)); // FIXME negative is ultra sus, but it works
        let y_rotation = Quaternion::from_angle_y(Rad(-euler.y));
        let z_rotation = Quaternion::from_angle_z(Rad(-euler.z));

        let rotation = z_rotation * y_rotation * x_rotation;
        self.rotation = rotation * self.rotation;
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        self.position_matrix() * self.scale_matrix() * self.rotation_matrix()
    }

    fn rotation_matrix(&self) -> Matrix4<f32> {
        Matrix4::from(self.rotation)
    }

    fn scale_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

    fn position_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position.to_vec())
    }

    pub fn from_matrix(matrix: Matrix4<f32>) -> Self {    // 1. Extract translation (global position)
        let translation = Point3::from_homogeneous(matrix.w);
    
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
        let rotation = Quaternion::from(rotation_matrix);
    
        Self { 
            position: translation,
            rotation: rotation,
            scale: scale,
        }
    }
}
