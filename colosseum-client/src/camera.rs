// Copyright 2021 Chay Nabors.

use gear::math::Isometry3;
use gear::math::Matrix4;
use gear::math::Point3;
use gear::math::UnitQuaternion;
use gear::math::Vector3;
use gear::math_ext::reversed_infinite_perspective_rh_zo;

pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub fov: f32,
    pub znear: f32,
    pub aspect_ratio: f32,
}

impl Camera {
    // pub fn roll(&self) -> f32 {
    //    self.rotation.euler_angles().0
    //}
    // pub fn pitch(&self) -> f32 {
    //    self.rotation.euler_angles().1
    //}
    // pub fn yaw(&self) -> f32 {
    //    self.rotation.euler_angles().2
    //}

    pub fn view(&self) -> Isometry3<f32> {
        Isometry3::look_at_rh(
            &self.position,
            &(self.position - (self.rotation.inverse() * Vector3::z())),
            &Vector3::y_axis(),
        )
    }

    pub fn projection(&self) -> Matrix4<f32> {
        reversed_infinite_perspective_rh_zo(self.aspect_ratio, self.fov, self.znear)
    }

    pub fn resize(&mut self, size: [u32; 2]) {
        self.aspect_ratio = size[0] as f32 / size[1] as f32;
    }

    pub fn look_at(&mut self, target: Point3<f32>) {
        self.rotation = UnitQuaternion::look_at_rh(&(target - self.position), &Vector3::y_axis());
    }
}
