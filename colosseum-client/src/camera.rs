// Copyright 2021 Chay Nabors.

use nalgebra::Isometry3;
use nalgebra::Matrix4;
use nalgebra::Point3;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;
use nalgebra_glm::reversed_infinite_perspective_rh_zo;
use winit::dpi::PhysicalPosition;
use winit::dpi::PhysicalSize;

pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub fov: f32,
}

impl Camera {
    pub fn new(position: Point3<f32>, rotation: UnitQuaternion<f32>, fov: f32) -> Self {
        Self {
            position,
            rotation,
            fov,
        }
    }

    pub fn view(&self) -> Isometry3<f32> {
        Isometry3::look_at_rh(
            &self.position,
            &Point3::origin(),
            //&(self.position - (self.rotation.inverse() * Vector3::z())),
            &Vector3::y_axis(),
        )
    }

    pub fn projection(&self, resolution: PhysicalSize<u32>) -> Matrix4<f32> {
        reversed_infinite_perspective_rh_zo(resolution.width as f32 / resolution.height as f32, self.fov, 0.1)
    }

    //pub fn look_at(&mut self, target: Point3<f32>) {
    //    self.rotation = UnitQuaternion::look_at_rh(&(target - self.position), &Vector3::y_axis());
    //}
}
