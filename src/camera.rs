use cgmath::{prelude::*, Deg, Matrix4, Point3, Vector3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = 
    Matrix4::new(
        1., 0., 0., 0., 
        0., 1., 0., 0., 
        0., 0., 0.5, 0., 
        0., 0., 0.5, 1.
    );

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: (0.0, 1.0, -2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vector3::unit_y(),
            aspect: 1.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

impl Camera {
    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    zoom_in: bool,
    zoom_out: bool,
    up_pressed: bool,
    down_pressed: bool,
    left_pressed: bool,
    right_pressed: bool,
}

use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            zoom_in: false,
            zoom_out: false,
            up_pressed: false,
            down_pressed: false,
            left_pressed: false,
            right_pressed: false,
        }
    }
    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Up => {
                        self.zoom_in = is_pressed;
                        true
                    }
                    VirtualKeyCode::Down => {
                        self.zoom_out = is_pressed;
                        true
                    }
                    VirtualKeyCode::W => {
                        self.up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S => {
                        self.down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A => {
                        self.left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D => {
                        self.right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if self.zoom_in && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.zoom_out {
            camera.eye -= forward_norm * self.speed;
        }
        let right = forward_norm.cross(camera.up);

        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.left_pressed {
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.right_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }

        if self.up_pressed {
            camera.eye =
                camera.target - (forward - camera.up * self.speed).normalize() * forward_mag;
        }
        if self.down_pressed {
            camera.eye =
                camera.target - (forward + camera.up * self.speed).normalize() * forward_mag;
        }
    }
}
