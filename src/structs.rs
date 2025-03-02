#[derive(Clone)]
pub struct WindowSize {
    pub height: usize,
    pub width: usize,
}

#[derive(Clone)]
pub struct Matrix<T> {
    pub width: usize,
    pub vector: Vec<T>,
}

impl<T> Matrix<T> {
    pub fn height(&self) -> usize {
        self.vector.len() / self.width
    }
}

pub struct SeamHolder {
    pub vertical_seams: Vec<Vec<usize>>,
    pub horizontal_seams: Vec<Vec<usize>>,
}
