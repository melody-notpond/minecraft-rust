use std::time::Duration;

use glium::{
    glutin::event::{ElementState, KeyboardInput, VirtualKeyCode},
    Frame, Surface,
};

#[derive(Clone, Debug)]
pub struct Camera {
    position: [f32; 3],
    direction: [f32; 3],
    velocity: [f32; 3],
    pressed: [bool; 6], // W S A D UP DOWN
    speed: f32,
    sensitivity: f32,
    pub fov: f32,
    pub z_far: f32,
    pub z_near: f32,
}

impl Camera {
    pub fn new(speed: f32, sensitivity: f32, fov: f32) -> Camera {
        Camera {
            position: [0.0; 3],
            direction: [0.0, 0.0, -1.0],
            velocity: [0.0; 3],
            pressed: [false; 6],
            speed,
            sensitivity,
            fov: fov.to_radians(),
            z_far: 1024.0,
            z_near: 0.01,
        }
    }

    pub fn get_pos(&self) -> [f32; 3] {
        self.position
    }

    pub fn is_moving(&self) -> bool {
        self.velocity[0].abs() > f32::EPSILON
            || self.velocity[1].abs() > f32::EPSILON
            || self.velocity[2].abs() > f32::EPSILON
    }

    pub fn tick(&mut self, delta: Duration) {
        self.position[0] += (self.velocity[0] * self.direction[0]
            + self.velocity[2] * self.direction[2])
            * delta.as_secs_f32()
            / (1.0 - self.direction[1] * self.direction[1]).sqrt();
        self.position[1] += self.velocity[1] * delta.as_secs_f32();
        self.position[2] += (self.velocity[0] * self.direction[2]
            - self.velocity[2] * self.direction[0])
            * delta.as_secs_f32()
            / (1.0 - self.direction[1] * self.direction[1]).sqrt();
    }

    pub fn move_self(&mut self, input: KeyboardInput) -> bool {
        let pressed = matches!(input.state, ElementState::Pressed);
        let mult = if pressed { 1.0 } else { -1.0 };

        match input.virtual_keycode {
            Some(VirtualKeyCode::W) if self.pressed[0] != pressed => {
                self.velocity[0] += self.speed * mult;
                self.pressed[0] = pressed;
                true
            }

            Some(VirtualKeyCode::S) if self.pressed[1] != pressed => {
                self.velocity[0] -= self.speed * mult;
                self.pressed[1] = pressed;
                true
            }

            Some(VirtualKeyCode::A) if self.pressed[2] != pressed => {
                self.velocity[2] += self.speed * mult;
                self.pressed[2] = pressed;
                true
            }

            Some(VirtualKeyCode::D) if self.pressed[3] != pressed => {
                self.velocity[2] -= self.speed * mult;
                self.pressed[3] = pressed;
                true
            }

            Some(VirtualKeyCode::Space) if self.pressed[4] != pressed => {
                self.velocity[1] += self.speed * mult;
                self.pressed[4] = pressed;
                true
            }

            Some(VirtualKeyCode::LShift) if self.pressed[5] != pressed => {
                self.velocity[1] -= self.speed * mult;
                self.pressed[5] = pressed;
                true
            }

            _ => false,
        }
    }

    pub fn turn_self(&mut self, dx: i32, dy: i32) {
        let dx = dx as f32 * self.sensitivity;
        let dy = dy as f32 * self.sensitivity;

        // SINE, COSINE, COSINE, SINE!
        // COSINE, COSINE... SINE-SINE!
        let [old_x, _, old_z] = self.direction;
        self.direction[0] = old_x * dx.cos() - old_z * dx.sin();
        self.direction[2] = old_z * dx.cos() + old_x * dx.sin();

        if dy.abs() > f32::EPSILON
            && !((self.direction[1] > 0.999 && dy < 0.0)
                || (self.direction[1] < -0.999 && dy > 0.0))
        {
            let old_factor = (1.0 - self.direction[1] * self.direction[1]).sqrt();
            self.direction[0] /= old_factor;
            self.direction[2] /= old_factor;

            let new_factor = old_factor * dy.cos() + self.direction[1] * dy.sin();
            self.direction[1] = self.direction[1] * dy.cos() - old_factor * dy.sin();
            self.direction[0] *= new_factor;
            self.direction[2] *= new_factor;
        }
    }

    pub fn perspective(&self, target: &Frame) -> [[f32; 4]; 4] {
        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;
        let fov: f32 = self.fov;
        let z_far = self.z_far;
        let z_near = self.z_near;

        let f = 1.0 / (fov / 2.0).tan();

        [
            [-f * aspect_ratio, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (z_far + z_near) / (z_far - z_near), 1.0],
            [0.0, 0.0, -(2.0 * z_far * z_near) / (z_far - z_near), 0.0],
        ]
    }

    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        let up = &[0.0, 1.0, 0.0f32];

        let f = self.direction;
        let s = [
            up[1] * f[2] - up[2] * f[1],
            up[2] * f[0] - up[0] * f[2],
            up[0] * f[1] - up[1] * f[0],
        ];
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        let s = [s[0] / len, s[1] / len, s[2] / len];

        let u = [
            f[1] * s[2] - f[2] * s[1],
            f[2] * s[0] - f[0] * s[2],
            f[0] * s[1] - f[1] * s[0],
        ];

        let p = [
            -self.position[0] * s[0] - self.position[1] * s[1] - self.position[2] * s[2],
            -self.position[0] * u[0] - self.position[1] * u[1] - self.position[2] * u[2],
            -self.position[0] * f[0] - self.position[1] * f[1] - self.position[2] * f[2],
        ];

        [
            [s[0], u[0], f[0], 0.0],
            [s[1], u[1], f[1], 0.0],
            [s[2], u[2], f[2], 0.0],
            [p[0], p[1], p[2], 1.0],
        ]
    }
}
