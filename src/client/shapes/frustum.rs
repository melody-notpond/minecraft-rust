use nalgebra::Vector3;

pub struct Plane {
    pub normal: [f32; 3],
    pub distance: f32,
}

impl Plane {
    fn get_signed_distance_to_plan(&self, point: [f32; 3]) -> f32 {
        Vector3::from(point).dot(&Vector3::from(self.normal)) - self.distance
    }
}

pub struct Frustum {
    pub top: Plane,
    pub bottom: Plane,
    pub left: Plane,
    pub right: Plane,
    pub far: Plane,
    pub near: Plane,
}

#[derive(Debug)]
pub struct Aabb {
    pub centre: [f32; 3],
    pub extents: [f32; 3],
}

impl Aabb {
    fn is_on_or_forward_plane(&self, plane: &Plane) -> bool {
        let r = self.extents[0] * plane.normal[0].abs()
            + self.extents[1] * plane.normal[1].abs()
            + self.extents[2] * plane.normal[2].abs();

        -r <= plane.get_signed_distance_to_plan(self.centre)
    }

    pub fn is_in_frustum(&self, frustum: &Frustum) -> bool {
        true
        //self.is_on_or_forward_plane(&frustum.near)
        //&& self.is_on_or_forward_plane(&frustum.far)
        //&& self.is_on_or_forward_plane(&frustum.left)
        //&& self.is_on_or_forward_plane(&frustum.right)
        //&& self.is_on_or_forward_plane(&frustum.top)
        //&& self.is_on_or_forward_plane(&frustum.bottom)
    }
}
