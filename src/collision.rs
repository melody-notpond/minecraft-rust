use crate::blocks::Block;

#[derive(Debug)]
pub struct Aabb {
    pub centre: [f32; 3],
    pub extents: [f32; 3],
}

pub trait Chunk {
    fn get_block(&self, x: usize, y: usize, z: usize) -> Block;
}

pub trait DetectCollision<Other> {
    fn is_colliding(&self, other: &Other) -> bool;
}

impl DetectCollision<Aabb> for Aabb {
    fn is_colliding(&self, other: &Aabb) -> bool {
        let collision_x = self.centre[0] + self.extents[0] >= other.centre[0] - other.extents[0]
            && other.centre[0] + other.extents[0] >= self.centre[0] - self.extents[0];
        let collision_y = self.centre[1] + self.extents[1] >= other.centre[1] - other.extents[1]
            && other.centre[1] + other.extents[1] >= self.centre[1] - self.extents[1];
        let collision_z = self.centre[2] + self.extents[2] >= other.centre[2] - other.extents[2]
            && other.centre[2] + other.extents[2] >= self.centre[2] - self.extents[2];

        collision_x && collision_y && collision_z
    }
}

