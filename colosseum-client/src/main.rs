// Copyright 2021 Chay Nabors.

mod camera;
mod config;
mod error;
mod renderer;

use std::io::Write;

use camera::Camera;
use config::Config;
use error::Error;
use nalgebra::Point3;
use nalgebra::UnitQuaternion;
use renderer::Renderer;
use winit::dpi::PhysicalSize;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "[{}: {}] {}", record.level(), record.metadata().target(), record.args()))
        .filter(None, log::LevelFilter::Info)
        .filter(Some("gfx_backend_dx11"), log::LevelFilter::Warn)
        .filter(Some("gfx_backend_vulkan"), log::LevelFilter::Warn)
        .filter(Some("wgpu_core"), log::LevelFilter::Warn)
        .init();

    let config = serde_json::from_slice::<Config>(include_bytes!("../config.json")).unwrap();
    let resolution = config.resolution;
    let mut resolution = PhysicalSize::new(resolution[0], resolution[1]);

    let event_loop = EventLoop::new();
    let window = match WindowBuilder::new()
        .with_title("Colosseum")
        .with_inner_size(resolution)
        .build(&event_loop)
    {
        Ok(window) => window,
        Err(_) => panic!("{:?}", Error::WindowCreationFailed),
    };

    let camera = Camera::new(Point3::new(0.0, 0.0, 3.0), UnitQuaternion::identity(), 90.0);
    let mut renderer = Renderer::new(&window).await.unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    resolution = size;
                    renderer.resize(resolution);
                }
                winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    resolution = *new_inner_size;
                    renderer.resize(resolution);
                },
                _ => (),
            },
            winit::event::Event::MainEventsCleared => (),
            winit::event::Event::RedrawRequested(_) => if let Err(e) = {
                let view_projection = camera.projection(resolution) * camera.view().to_homogeneous();
                renderer.render(view_projection)
            } {
                panic!("{:?}", e);
            },
            _ => (),
        }
    });
}
