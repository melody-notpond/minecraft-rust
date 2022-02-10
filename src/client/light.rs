use crate::blocks::FaceDirection;

pub struct LightSource {
    red: u8,
    green: u8,
    blue: u8,
    intensity: u8,
    location: [f32; 3],
    updated: bool,
}

impl LightSource {
    pub fn new(red: u8, green: u8, blue: u8, intensity: u8, location: [f32; 3]) -> LightSource {
        LightSource {
            red,
            green,
            blue,
            intensity,
            location,
            updated: true,
        }
    }

    pub fn calculate_light_intensity(&self, x: i32, y: i32, z: i32, _dir: FaceDirection) -> u32 {
        let dx = self.location[0] - x as f32 * 0.5;
        let dy = self.location[1] - y as f32 * 0.5;
        let dz = self.location[2] - z as f32 * 0.5;
        (self.intensity as f32 - (dx * dx + dy * dy + dz * dz).sqrt()) as u32
    }

    pub fn red(&self) -> u8 {
        self.red
    }

    pub fn green(&self) -> u8 {
        self.green
    }

    pub fn blue(&self) -> u8 {
        self.blue
    }

    pub fn intensity(&self) -> u8 {
        self.intensity
    }

    pub fn location(&self) -> [f32; 3] {
        self.location
    }

    pub fn updated(&self) -> bool {
        self.updated
    }

    pub fn reset_updated(&mut self) {
        self.updated = false;
    }

    pub fn set_red(&mut self, red: u8) {
        self.red = red;
        self.updated = true;
    }

    pub fn set_green(&mut self, green: u8) {
        self.green = green;
        self.updated = true;
    }

    pub fn set_blue(&mut self, blue: u8) {
        self.blue = blue;
        self.updated = true;
    }

    pub fn set_intensity(&mut self, intensity: u8) {
        self.intensity = intensity;
        self.updated = true;
    }

    pub fn set_location(&mut self, location: [f32; 3]) {
        self.location = location;
        self.updated = true;
    }
}
