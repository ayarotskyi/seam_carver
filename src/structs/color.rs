#[derive(Clone, Copy, Debug)]
pub struct CustomColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub is_inserted: bool,
}

impl PartialEq for CustomColor {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}
