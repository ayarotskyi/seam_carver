#[derive(Clone)]
pub struct WindowSize {
    pub height: usize,
    pub width: usize,
}

#[derive(Clone)]
pub struct Matrix<T> {
    pub width: usize,
    pub vector: Vec<T>,
    pub original_indices: Vec<usize>,
}
impl<T> Matrix<T> {
    pub fn height(&self) -> usize {
        self.vector.len() / self.width
    }
}

#[derive(Clone)]
pub struct Seam {
    pub indices: Vec<usize>,
}

pub struct SeamHolder {
    pub vertical_seams: Vec<Seam>,
    pub horizontal_seams: Vec<Seam>,
}
