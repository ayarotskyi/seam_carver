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
    pub fn insert_overflowing_indices(&mut self, overflowing_indices: Vec<usize>, vertical: bool) {
        let mut seams: Vec<Seam> = if vertical {
            std::mem::replace(&mut self.vertical_seams, Vec::new())
        } else {
            std::mem::replace(&mut self.horizontal_seams, Vec::new())
        };
        let seams_len = seams.len();
        overflowing_indices
            .iter()
            .take(seams_len)
            .enumerate()
            .for_each(|(index, index_to_insert)| {
                let seam = seams
                    .get_mut(seams_len - overflowing_indices.len() + index)
                    .unwrap();
                let mut prev_index: Option<usize> = None;
                seam.indices
                    .iter_mut()
                    .for_each(|original_index| match prev_index {
                        Some(prev_index_value) => {
                            let temp = *original_index;
                            *original_index = prev_index_value;
                            prev_index = Some(temp);
                        }
                        None => {
                            if *original_index >= *index_to_insert {
                                prev_index = Some(*original_index);
                                *original_index = *index_to_insert;
                            }
                        }
                    });
                prev_index.inspect(|prev_index| {
                    seam.indices.push(*prev_index);
                });
            });

        let _ = std::mem::replace(
            if vertical {
                &mut self.vertical_seams
            } else {
                &mut self.horizontal_seams
            },
            seams,
        );
    }
}
