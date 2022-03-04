pub struct LightSource {
    red: u8,
    green: u8,
    blue: u8,
    location: [f32; 3],
    updated: bool,
}

impl LightSource {
    pub fn new(red: u8, green: u8, blue: u8, location: [f32; 3]) -> LightSource {
        LightSource {
            red,
            green,
            blue,
            location,
            updated: true,
        }
    }

    pub fn as_uint(&self) -> u32 {
        ((self.red as u32 & 0xf) << 12) | ((self.green as u32 & 0xf) << 8) | ((self.blue as u32 & 0xf) << 4)
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

    pub fn set_location(&mut self, location: [f32; 3]) {
        self.location = location;
        self.updated = true;
    }
}
