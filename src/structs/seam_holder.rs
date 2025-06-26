use crate::structs::matrix::Seam;

#[cfg(test)]
#[path = "../tests/seam_holder.rs"]
mod seam_holder_tests;

pub struct SeamHolder {
    vertical_seams: Vec<Seam>,
    horizontal_seams: Vec<Seam>,
}

impl SeamHolder {
    pub fn new() -> Self {
        return SeamHolder {
            vertical_seams: Vec::new(),
            horizontal_seams: Vec::new(),
        };
    }
    pub fn push_seam(&mut self, seam: Seam) {
        if seam.is_vertical {
            self.vertical_seams.push(seam);
        } else {
            self.horizontal_seams.push(seam);
        }
    }
    pub fn pop_n_seams(&mut self, n: usize, vertical: bool) -> Vec<Seam> {
        if vertical {
            return self
                .vertical_seams
                .splice(
                    (if n < self.vertical_seams.len() {
                        self.vertical_seams.len() - n
                    } else {
                        0
                    })..self.vertical_seams.len(),
                    [],
                )
                .collect();
        } else {
            return self
                .horizontal_seams
                .splice(
                    (if n < self.horizontal_seams.len() {
                        self.horizontal_seams.len() - n
                    } else {
                        0
                    })..self.horizontal_seams.len(),
                    [],
                )
                .collect();
        }
    }
    pub fn insert_overflowing_indices(&mut self, overflowing_indices: Vec<usize>, vertical: bool) {}
}
