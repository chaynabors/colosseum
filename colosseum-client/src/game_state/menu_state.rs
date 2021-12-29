// Copyright 2021 Chay Nabors.

mod connecting_state;
mod navigation_state;
mod splash_state;

use std::rc::Rc;
use std::time::Duration;

use gear::event::Event;
use gear::event::NetworkEvent;
use gear::event::WindowEvent;
use gear::math::Point3;
use gear::math::Transform;
use gear::math::UnitQuaternion;
use gear::Engine;
use gear::Loadable;
use gear::Model;

use self::connecting_state::ConnectingState;
use self::navigation_state::NavigationState;
use self::splash_state::SplashState;
use super::state_transition::StateTransition;
use super::GameState;
use crate::camera::Camera;
use crate::config::Config;

pub enum MenuSubState {
    SplashState(SplashState),
    ConnectingState(ConnectingState),
    NavigationState(NavigationState),
}

pub struct MenuState {
    pub config: Rc<Config>,
    pub sub_states: Vec<MenuSubState>,
    pub camera: Camera,
    pub time: Duration,
    pub model: Model,
}

impl MenuState {
    pub fn new(config: Rc<Config>, engine: &Engine) -> Self {
        let size = engine.window.size();

        Self {
            config: config.clone(),
            sub_states: vec![MenuSubState::SplashState(SplashState::new(config))],
            camera: Camera {
                position: Point3::new(1.0_f32.cos(), 5., 1.0_f32.sin()),
                rotation: UnitQuaternion::default(),
                fov: 90.,
                znear: 0.1,
                aspect_ratio: size[0] as f32 / size[1] as f32,
            },
            time: Duration::ZERO,
            model: Model::load("content/colosseum.obj").unwrap(),
        }
    }

    pub fn handle_event(&mut self, event: &Event, engine: &mut Engine) -> Option<GameState> {
        let mut state = self.sub_states.pop().unwrap();

        match event {
            Event::UpdateEvent { delta_time } => {
                self.time = self.time.saturating_add(*delta_time);
                let distance_multiplier = 2.5;
                self.camera.position = Point3::new(
                    self.time.as_secs_f32().cos() * distance_multiplier,
                    2.5,
                    self.time.as_secs_f32().sin() * distance_multiplier,
                );
                self.camera.look_at(Point3::origin());

                engine
                    .renderer
                    .set_clear_color([0.05, 0.05, 0.05, 1.0])
                    .set_view(self.camera.view())
                    .set_projection(self.camera.projection())
                    .draw_model(&self.model, Transform::identity())
                    .submit();
            },
            Event::WindowEvent(event) => {
                if let WindowEvent::Resized(size) = event {
                    self.camera.resize(*size);
                }
            },
            Event::NetworkEvent(event) => match event {
                NetworkEvent::Timeout(address) | NetworkEvent::Disconnect(address) => {
                    if *address == self.config.server_address {
                        while self.sub_states.len() > 1 {
                            self.sub_states.pop();
                        }

                        return None;
                    }
                },
                _ => (),
            },
            _ => (),
        }

        let state_transition = match &mut state {
            MenuSubState::SplashState(state) => state.handle_event(event, engine),
            MenuSubState::ConnectingState(state) => state.handle_event(event),
            MenuSubState::NavigationState(navstate) => {
                let state_transition = navstate.handle_event(event, engine);
                if let StateTransition::Old = state_transition {
                    state = self.sub_states.pop().unwrap();
                }
                state_transition
            },
        };

        match state_transition {
            StateTransition::None => self.sub_states.push(state),
            StateTransition::Old => (),
            StateTransition::New(new_state) => {
                self.sub_states.push(state);
                self.sub_states.push(new_state);
            },
            StateTransition::Change(new_state) => {
                self.sub_states.push(state);
                return Some(new_state);
            },
        }

        None
    }
}
