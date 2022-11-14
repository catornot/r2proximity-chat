#[derive(Debug, Clone, Copy, Default)]
pub struct Comms {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl From<(i32, i32, i32)> for Comms {
    fn from(tuple: (i32, i32, i32)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
            z: tuple.2,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<(i32, i32, i32)> for Comms {
    fn into(self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }
}
