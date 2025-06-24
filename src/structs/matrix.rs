use crate::*;
use ::rand::{rngs::ThreadRng, Rng};

#[cfg(test)]
#[path = "../tests/matrix.rs"]
mod matrix_tests;

#[derive(Clone)]
pub struct Seam {
    pub indices: Vec<usize>,
    pub is_vertical: bool,
}

#[derive(Clone)]
pub struct Matrix<T> {
    pub width: usize,
    pub vector: Vec<T>,
    // we need to remember which item had which index before carving to be able to optimize carving and extraction
    pub original_indices: Vec<usize>,
}
impl<T> Matrix<T>
where
    T: Clone + std::marker::Send + Sync + Copy,
{
    pub fn height(&self) -> usize {
        self.vector.len() / self.width
    }
    pub fn new(vector: Vec<T>, width: usize) -> Self {
        Matrix {
            width: width,
            original_indices: (0..vector.len()).collect(),
            vector: vector,
        }
    }
    fn carve_horizontal_seam(&mut self, seam: &Seam) {
        let height = self.height();

        let column_vectors: Vec<(Vec<T>, Vec<usize>)> = (0..self.width)
            .into_par_iter()
            .map(|column| {
                let mut vector_result = Vec::with_capacity(self.height() - 1);
                let mut original_indices_result = Vec::with_capacity(self.height() - 1);
                for row in 0..height {
                    let index = self.original_indices[row * self.width + column];
                    if seam.indices[column] != index {
                        vector_result.push(self.vector[row * self.width + column]);
                        original_indices_result.push(index);
                    }
                }

                return (vector_result, original_indices_result);
            })
            .collect::<Vec<(Vec<T>, Vec<usize>)>>();

        let result = (0..(self.height() - 1))
            .into_par_iter()
            .map(|row| {
                column_vectors
                    .iter()
                    .map(|column_vector| (column_vector.0[row], column_vector.1[row]))
                    .collect::<Vec<(T, usize)>>()
            })
            .collect::<Vec<Vec<(T, usize)>>>()
            .concat();

        *self = Matrix {
            width: self.width,
            vector: result.iter().map(|item| item.0).collect(),
            original_indices: result.iter().map(|item| item.1).collect(),
        };
    }
    fn carve_vertical_seam(&mut self, seam: &Seam) {
        let mut indices_to_remove = seam.indices.iter();

        let mut resulting_vector: Vec<T> = Vec::with_capacity(self.vector.len() - self.height());
        let mut resulting_original_indices: Vec<usize> =
            Vec::with_capacity(self.vector.len() - self.height());

        let mut index_to_remove = match indices_to_remove.next() {
            None => {
                return;
            }
            Some(index_to_remove) => *index_to_remove,
        };

        for (index, value) in self.vector.iter().enumerate() {
            let original_index = self.original_indices[index];
            if index_to_remove == original_index {
                index_to_remove = match indices_to_remove.next() {
                    None => {
                        continue;
                    }
                    Some(index_to_remove) => *index_to_remove,
                };
                continue;
            }
            resulting_vector.push(*value);
            resulting_original_indices.push(original_index);
        }

        *self = Matrix {
            width: self.width - 1,
            vector: resulting_vector,
            original_indices: resulting_original_indices,
        };
    }
    pub fn carve_seam(&mut self, seam: &Seam) {
        if seam.is_vertical {
            self.carve_vertical_seam(seam);
        } else {
            self.carve_horizontal_seam(seam);
        }
    }
    fn recover_vertical_seam(&mut self, seam: &Seam, original_matrix: &Self) {
        let recovered_vector_length = self.vector.len() + self.height();

        let mut seam_iterator = seam.indices.iter();
        let mut next_original_index = match seam_iterator.next() {
            Some(index) => *index,
            None => recovered_vector_length,
        };
        let mut recovered_vector = Vec::with_capacity(recovered_vector_length);
        let mut recovered_original_indices = Vec::with_capacity(recovered_vector_length);

        for index in 0..self.vector.len() {
            let original_index = self.original_indices[index];
            if original_index >= next_original_index {
                let temp_index = next_original_index;
                next_original_index = match seam_iterator.next() {
                    Some(index) => *index,
                    None => recovered_vector_length,
                };
                recovered_vector.push(original_matrix.vector[temp_index]);
                recovered_original_indices.push(temp_index);
            }
            recovered_vector.push(self.vector[index]);
            recovered_original_indices.push(original_index);
        }
        if next_original_index < recovered_vector_length {
            recovered_vector.push(original_matrix.vector[next_original_index]);
            recovered_original_indices.push(next_original_index);
        }

        self.vector = recovered_vector;
        self.original_indices = recovered_original_indices;
        self.width = self.width + 1;
    }
    fn recover_horizontal_seam(&mut self, seam: &Seam, original_matrix: &Self) {
        let recovered_height = self.height() + 1;

        let column_vectors: Vec<(Vec<T>, Vec<usize>)> = (0..self.width)
            .into_iter()
            .map(|column| {
                let seam_index = seam.indices[column];

                let mut recovered_values = Vec::with_capacity(recovered_height);
                let mut recovered_original_indices = Vec::with_capacity(recovered_height);
                let mut original_column = Vec::with_capacity(self.height());
                let mut seam_inserted = false;
                for row in 0..(recovered_height - 1) {
                    let index = row * self.width + column;
                    let original_index = self.original_indices[index];
                    if original_index >= seam_index && !seam_inserted {
                        recovered_values.push(original_matrix.vector[seam_index]);
                        recovered_original_indices.push(seam_index);
                        seam_inserted = true;
                    }
                    original_column.push(original_index);

                    recovered_values.push(self.vector[index]);
                    recovered_original_indices.push(original_index);
                }

                return (recovered_values, recovered_original_indices);
            })
            .collect::<Vec<(Vec<T>, Vec<usize>)>>();

        let result = (0..recovered_height)
            .into_par_iter()
            .map(|row| {
                column_vectors
                    .iter()
                    .map(|column_vector| (column_vector.0[row], column_vector.1[row]))
                    .collect::<Vec<(T, usize)>>()
            })
            .collect::<Vec<Vec<(T, usize)>>>()
            .concat();

        *self = Matrix {
            width: self.width,
            vector: result.iter().map(|item| item.0).collect(),
            original_indices: result.iter().map(|item| item.1).collect(),
        };
    }
}

impl Matrix<f32> {
    pub fn extract_vertical_seam(&self, rng: &mut ThreadRng) -> (Seam, f32) {
        let mut dp_result = self.vector.clone();
        let width = self.width;
        let height = self.height();

        // fill in the vector using dynamic programming
        for i in 1..height {
            for j in 0..width {
                dp_result[i * width + j] = dp_result[(i - 1) * width + j]
                    .min(if j == 0 {
                        dp_result[(i - 1) * width + j]
                    } else {
                        dp_result[(i - 1) * width + (j - 1)]
                    })
                    .min(if j == width - 1 {
                        dp_result[(i - 1) * width + j]
                    } else {
                        dp_result[(i - 1) * width + (j + 1)]
                    })
                    + dp_result[i * width + j];
            }
        }

        let mut indices = vec![0; height];

        // calculate the last element in seam by randomly
        // selecting one of the minimum points in the last row
        let mut min_indices = Vec::with_capacity(width);
        let mut current_min = dp_result[(height - 1) * width];
        dp_result
            .iter()
            .enumerate()
            .skip((height - 1) * width)
            .for_each(|(index, value)| {
                if *value < current_min {
                    min_indices.truncate(0);
                    min_indices.push(index);
                    current_min = *value;
                }
                if *value == current_min {
                    min_indices.push(index);
                }
            });
        indices[height - 1] = min_indices[rng.gen_range(0..min_indices.len())];

        // calculate the rest of the indexes for the seam
        for i in (0..height - 1).rev() {
            let index = {
                let mid_index = indices[i + 1] - width;
                let left_index = if mid_index == i * width {
                    mid_index
                } else {
                    mid_index - 1
                };

                let mut min_index = if dp_result[left_index] < dp_result[mid_index] {
                    left_index
                } else {
                    mid_index
                };

                let right_index = if mid_index == width * (i + 1) - 1 {
                    mid_index
                } else {
                    mid_index + 1
                };

                min_index = if dp_result[min_index] < dp_result[right_index] {
                    min_index
                } else {
                    right_index
                };

                min_index
            };
            indices[i] = index;
        }

        (
            Seam {
                indices: indices
                    .iter()
                    .map(|index| self.original_indices[*index])
                    .collect(),
                is_vertical: true,
            },
            indices
                .iter()
                .map(|index| self.vector[*index])
                .reduce(|acc, value| acc + value)
                .unwrap(),
        )
    }
    pub fn extract_horizontal_seam(&self, rng: &mut ThreadRng) -> (Seam, f32) {
        let mut dp_result = self.vector.clone();
        let width = self.width;
        let height = self.vector.len() / width;

        // fill in the vector using dynamic programming
        for i in 0..height {
            for j in 1..width {
                dp_result[i * width + j] = dp_result[i * width + j - 1]
                    .min(if i == 0 {
                        dp_result[i * width + j - 1]
                    } else {
                        dp_result[(i - 1) * width + j - 1]
                    })
                    .min(if i == height - 1 {
                        dp_result[i * width + j - 1]
                    } else {
                        dp_result[(i + 1) * width + j - 1]
                    })
                    + dp_result[i * width + j];
            }
        }

        let mut indices = vec![0; width];

        // calculate the last element in seam by randomly
        // selecting one of the minimum points in the last column
        let mut min_indices = Vec::with_capacity(height);
        let mut current_min = dp_result[width - 1];
        dp_result
            .iter()
            .enumerate()
            .skip(width - 1)
            .step_by(width)
            .for_each(|(index, value)| {
                if *value < current_min {
                    min_indices.truncate(0);
                    min_indices.push(index);
                    current_min = *value;
                }
                if *value == current_min {
                    min_indices.push(index);
                }
            });
        indices[width - 1] = min_indices[rng.gen_range(0..min_indices.len())];

        // calculate the rest of the indexes for the seam
        for i in (0..width - 1).rev() {
            let index = {
                let mid_index = indices[i + 1] - 1;
                let top_index = if mid_index >= width {
                    mid_index - width
                } else {
                    mid_index
                };

                let mut min_index = if dp_result[top_index] < dp_result[mid_index] {
                    top_index
                } else {
                    mid_index
                };

                let bottom_index = if mid_index >= self.vector.len() - width {
                    mid_index
                } else {
                    mid_index + width
                };

                min_index = if dp_result[min_index] < dp_result[bottom_index] {
                    min_index
                } else {
                    bottom_index
                };

                min_index
            };
            indices[i] = index;
        }

        (
            Seam {
                indices: indices
                    .iter()
                    .map(|index| self.original_indices[*index])
                    .collect(),
                is_vertical: false,
            },
            indices
                .iter()
                .map(|index| self.vector[*index])
                .reduce(|acc, value| acc + value)
                .unwrap(),
        )
    }
}
