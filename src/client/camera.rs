use std::{time::Duration, convert::TryInto, collections::HashMap};

use glium::{
    glutin::event::{ElementState, KeyboardInput, VirtualKeyCode},
    Frame, Surface, Display,
};
use nalgebra::Vector3;

use crate::blocks::{Block, CHUNK_SIZE};

use super::{shapes::frustum::{Frustum, Plane}, chunk::ChunkWaiter};

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
            direction: [1.0, 0.0, 0.0],
            velocity: [0.0; 3],
            pressed: [false; 6],
            speed,
            sensitivity,
            fov: fov.to_radians(),
            z_far: 1024.0,
            z_near: 0.1,
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
                self.velocity[2] -= self.speed * mult;
                self.pressed[2] = pressed;
                true
            }

            Some(VirtualKeyCode::D) if self.pressed[3] != pressed => {
                self.velocity[2] += self.speed * mult;
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
        self.direction[0] = old_x * dx.cos() + old_z * dx.sin();
        self.direction[2] = old_z * dx.cos() - old_x * dx.sin();

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
            [f * aspect_ratio, 0.0, 0.0, 0.0],
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

    pub fn frustum(&self, target: &Frame) -> Frustum {
        // https://learnopengl.com/Guest-Articles/2021/Scene/Frustum-Culling
        let (width, height) = target.get_dimensions();
        let half_v_side = self.z_far * (self.fov * 0.5).tan();
        let half_h_side = half_v_side * height as f32 / width as f32;
        let front = Vector3::from(self.direction).normalize();
        let front_mult_far = self.z_far * front;
        let right = front.cross(&Vector3::from([0.0, 1.0, 0.0]));
        let up = right.cross(&front);

        let mut frustum = Frustum {
            top: Plane {
                normal: right.cross(&(front_mult_far - up * half_v_side)).normalize().try_into().unwrap(),
                distance: 0.0,
            },
            bottom: Plane {
                normal: (front_mult_far + up * half_v_side).cross(&right).normalize().try_into().unwrap(),
                distance: 0.0,
            },
            left: Plane {
                normal: (front_mult_far - right * half_h_side).cross(&up).normalize().try_into().unwrap(),
                distance: 0.0,
            },
            right: Plane {
                normal: up.cross(&(front_mult_far + right * half_h_side)).normalize().try_into().unwrap(),
                distance: 0.0,
            },
            far: Plane {
                normal: (-front).try_into().unwrap(),
                distance: 0.0,
            },
            near: Plane {
                normal: front.try_into().unwrap(),
                distance: 0.0,
            },
        };

        frustum.near.distance = (Vector3::from(self.position) + self.z_near * front).norm();
        frustum.far.distance = Vector3::from(frustum.far.normal).dot(&(Vector3::from(self.position) + front_mult_far)).abs();
        frustum.left.distance = Vector3::from(frustum.left.normal).dot(&Vector3::from(self.position)).abs();
        frustum.right.distance = Vector3::from(frustum.right.normal).dot(&Vector3::from(self.position)).abs();
        frustum.top.distance = Vector3::from(frustum.top.normal).dot(&Vector3::from(self.position)).abs();
        frustum.bottom.distance = Vector3::from(frustum.bottom.normal).dot(&Vector3::from(self.position)).abs();

        frustum
    }

    pub fn raycast(&self, display: &Display, chunks: &mut HashMap<(i32, i32, i32), ChunkWaiter>, action: RaycastAction) {
        let mut pos = self.position;

        for _ in 0..16 {
            let mut pos2 = [(pos[0] * 2.0).floor() as i32, (pos[1] * 2.0).floor() as i32, (pos[2] * 2.0).floor() as i32];
            pos = [pos[0] + self.direction[0] * 0.25, pos[1] + self.direction[1] * 0.25, pos[2] + self.direction[2] * 0.25];

            if pos2[0] < 0 {
                pos2[0] -= CHUNK_SIZE as i32;
            }
            if pos2[1] < 0 {
                pos2[1] -= CHUNK_SIZE as i32;
            }
            if pos2[2] < 0 {
                pos2[2] -= CHUNK_SIZE as i32;
            }

            let (chunk_x, chunk_y, chunk_z) = (pos2[0] / CHUNK_SIZE as i32, pos2[1] / CHUNK_SIZE as i32, pos2[2] / CHUNK_SIZE as i32);
            let chunk = match chunks.get_mut(&(chunk_x, chunk_y, chunk_z)) {
                Some(ChunkWaiter::Chunk(chunk)) => chunk,
                _ => continue,
            };

            if pos2[0] < 0 {
                pos2[0] += CHUNK_SIZE as i32;
            }
            if pos2[1] < 0 {
                pos2[1] += CHUNK_SIZE as i32;
            }
            if pos2[2] < 0 {
                pos2[2] += CHUNK_SIZE as i32;
            }

            let (x, y, z) = ((pos2[0] - chunk_x * CHUNK_SIZE as i32) % CHUNK_SIZE as i32, (pos2[1] - chunk_y * CHUNK_SIZE as i32) % CHUNK_SIZE as i32, (pos2[2] - chunk_z * CHUNK_SIZE as i32) % CHUNK_SIZE as i32);
            let (x, y, z) = (x as usize, y as usize, z as usize);
            let block = chunk.block_mut(x, y, z);

            if block.is_solid().unwrap_or(false) {
                match action {
                    RaycastAction::Place(block) => {
                        pos = [pos[0] - self.direction[0] * 0.5, pos[1] - self.direction[1] * 0.5, pos[2] - self.direction[2] * 0.5];
                        let mut pos2 = [(pos[0] * 2.0).floor() as i32, (pos[1] * 2.0).floor() as i32, (pos[2] * 2.0).floor() as i32];

                        if pos2[0] < 0 {
                            pos2[0] -= CHUNK_SIZE as i32;
                        }
                        if pos2[1] < 0 {
                            pos2[1] -= CHUNK_SIZE as i32;
                        }
                        if pos2[2] < 0 {
                            pos2[2] -= CHUNK_SIZE as i32;
                        }

                        let (chunk_x, chunk_y, chunk_z) = (pos2[0] / CHUNK_SIZE as i32, pos2[1] / CHUNK_SIZE as i32, pos2[2] / CHUNK_SIZE as i32);
                        let chunk = match chunks.get_mut(&(chunk_x, chunk_y, chunk_z)) {
                            Some(ChunkWaiter::Chunk(chunk)) => chunk,
                            _ => continue,
                        };
                        if pos2[0] < 0 {
                            pos2[0] += CHUNK_SIZE as i32;
                        }
                        if pos2[1] < 0 {
                            pos2[1] += CHUNK_SIZE as i32;
                        }
                        if pos2[2] < 0 {
                            pos2[2] += CHUNK_SIZE as i32;
                        }

                        let (x, y, z) = ((pos2[0] - chunk_x * CHUNK_SIZE as i32) % CHUNK_SIZE as i32, (pos2[1] - chunk_y * CHUNK_SIZE as i32) % CHUNK_SIZE as i32, (pos2[2] - chunk_z * CHUNK_SIZE as i32) % CHUNK_SIZE as i32);
                        let (x, y, z) = (x as usize, y as usize, z as usize);
                        *chunk.block_mut(x, y, z) = block;
                        chunk.invalidate_mesh();

                        if x == 0 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x - 1, chunk_y, chunk_z)) {
                                chunk.invalidate_mesh();
                            }
                        } else if x == CHUNK_SIZE - 1 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x + 1, chunk_y, chunk_z)) {
                                chunk.invalidate_mesh();
                            }
                        }

                        if y == 0 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x, chunk_y - 1, chunk_z)) {
                                chunk.invalidate_mesh();
                            }
                        } else if y == CHUNK_SIZE - 1 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x, chunk_y + 1, chunk_z)) {
                                chunk.invalidate_mesh();
                            }
                        }

                        if z == 0 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x, chunk_y, chunk_z - 1)) {
                                chunk.invalidate_mesh();
                            }
                        } else if z == CHUNK_SIZE - 1 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x, chunk_y, chunk_z + 1)) {
                                chunk.invalidate_mesh();
                            }
                        }
                    }

                    RaycastAction::Remove => {
                        *block = Block::air();
                        chunk.invalidate_mesh();

                        if x == 0 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x - 1, chunk_y, chunk_z)) {
                                chunk.invalidate_mesh();
                            }
                        } else if x == CHUNK_SIZE - 1 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x + 1, chunk_y, chunk_z)) {
                                chunk.invalidate_mesh();
                            }
                        }

                        if y == 0 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x, chunk_y - 1, chunk_z)) {
                                chunk.invalidate_mesh();
                            }
                        } else if y == CHUNK_SIZE - 1 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x, chunk_y + 1, chunk_z)) {
                                chunk.invalidate_mesh();
                            }
                        }

                        if z == 0 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x, chunk_y, chunk_z - 1)) {
                                chunk.invalidate_mesh();
                            }
                        } else if z == CHUNK_SIZE - 1 {
                            if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x, chunk_y, chunk_z + 1)) {
                                chunk.invalidate_mesh();
                            }
                        }
                    }

                    RaycastAction::Unselect => {
                        chunk.invalidate_selection();
                        chunk.select(display, None);
                    }

                    RaycastAction::Select => {
                        chunk.invalidate_selection();
                        chunk.select(display, Some((x, y, z)));
                    }
                }

                break;
            }
        }
    }
}

pub enum RaycastAction {
    Place(Block),
    Remove,
    Unselect,
    Select,
}
