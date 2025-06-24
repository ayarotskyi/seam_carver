use crate::*;
use ::rand::{rngs::ThreadRng, Rng};

#[cfg(test)]
#[path = "../tests/matrix.rs"]
mod matrix_tests;

#[derive(Clone, Copy)]
pub struct MemoryPoint<T> {
    pub value: T,
    pub original_index: usize,
}

#[derive(Clone)]
pub struct Seam {
    pub indices: Vec<MemoryPoint<usize>>,
    pub is_vertical: bool,
}

#[derive(Clone)]
pub struct Matrix<T> {
    width: usize,
    pub vector: Vec<MemoryPoint<T>>,
}
impl<T> Matrix<T>
where
    T: Clone + std::marker::Send + Sync + Copy,
{
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.vector.len() / self.width
    }
    pub fn new(vector: Vec<T>, width: usize) -> Self {
        Matrix {
            width: width,
            vector: vector
                .into_iter()
                .enumerate()
                .map(|(index, value)| MemoryPoint {
                    value: value,
                    original_index: index,
                })
                .collect::<Vec<MemoryPoint<T>>>(),
        }
    }
    pub fn from_memory_points(vector: Vec<MemoryPoint<T>>, width: usize) -> Self {
        Matrix {
            width: width,
            vector: vector,
        }
    }
    fn carve_horizontal_seam(&mut self, seam: &Seam) {
        let height = self.height();

        let column_vectors: Vec<Vec<MemoryPoint<T>>> = (0..self.width)
            .into_par_iter()
            .map(|column| {
                let mut vector_result = Vec::with_capacity(self.height() - 1);
                for row in 0..height {
                    let memory_point = self.vector[row * self.width + column];
                    if seam.indices[column].original_index != memory_point.original_index {
                        vector_result.push(self.vector[row * self.width + column]);
                    }
                }

                return vector_result;
            })
            .collect::<Vec<Vec<MemoryPoint<T>>>>();

        let result = (0..(self.height() - 1))
            .into_par_iter()
            .map(|row| {
                column_vectors
                    .iter()
                    .map(|column_vector| column_vector[row])
                    .collect::<Vec<MemoryPoint<T>>>()
            })
            .collect::<Vec<Vec<MemoryPoint<T>>>>()
            .concat();

        self.vector = result;
    }
    fn carve_vertical_seam(&mut self, seam: &Seam) {
        let mut indices_to_remove = seam.indices.iter();

        let mut resulting_vector: Vec<MemoryPoint<T>> =
            Vec::with_capacity(self.vector.len() - self.height());

        let mut index_to_remove = match indices_to_remove.next() {
            None => {
                return;
            }
            Some(index_to_remove) => *index_to_remove,
        };

        for memory_point in self.vector.iter() {
            if index_to_remove.original_index == memory_point.original_index {
                index_to_remove = match indices_to_remove.next() {
                    None => {
                        continue;
                    }
                    Some(index_to_remove) => *index_to_remove,
                };
                continue;
            }
            resulting_vector.push(*memory_point);
        }

        self.vector = resulting_vector;
        self.width = self.width - 1;
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
            Some(index) => index.original_index,
            None => recovered_vector_length,
        };
        let mut recovered_vector = Vec::with_capacity(recovered_vector_length);

        self.vector.iter().for_each(|memory_point| {
            if memory_point.original_index >= next_original_index {
                let temp_index = next_original_index;
                next_original_index = match seam_iterator.next() {
                    Some(index) => index.original_index,
                    None => recovered_vector_length,
                };
                recovered_vector.push(original_matrix.vector[temp_index]);
            }
            recovered_vector.push(*memory_point);
        });
        if next_original_index < recovered_vector_length {
            recovered_vector.push(original_matrix.vector[next_original_index]);
        }

        self.vector = recovered_vector;
        self.width = self.width + 1;
    }
    fn recover_horizontal_seam(&mut self, seam: &Seam, original_matrix: &Self) {
        let recovered_height = self.height() + 1;

        let column_vectors: Vec<Vec<MemoryPoint<T>>> = (0..self.width)
            .into_iter()
            .map(|column| {
                let seam_index = seam.indices[column].original_index;

                let mut recovered_points = Vec::with_capacity(recovered_height);
                let mut seam_inserted = false;
                for row in 0..(recovered_height - 1) {
                    let index = row * self.width + column;
                    let memory_point = self.vector[index];
                    if memory_point.original_index >= seam_index && !seam_inserted {
                        recovered_points.push(original_matrix.vector[seam_index]);
                        seam_inserted = true;
                    }

                    recovered_points.push(memory_point);
                }

                return recovered_points;
            })
            .collect::<Vec<Vec<MemoryPoint<T>>>>();

        let result = (0..recovered_height)
            .into_par_iter()
            .map(|row| {
                column_vectors
                    .iter()
                    .map(|column_vector| column_vector[row])
                    .collect::<Vec<MemoryPoint<T>>>()
            })
            .collect::<Vec<Vec<MemoryPoint<T>>>>()
            .concat();

        self.vector = result;
    }
}

impl Matrix<f32> {
    pub fn extract_vertical_seam(&self, rng: &mut ThreadRng) -> (Seam, f32) {
        let mut dp_result = self
            .vector
            .iter()
            .map(|memory_point| memory_point.value)
            .collect::<Vec<f32>>();
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
                    .map(|index| MemoryPoint {
                        value: *index,
                        original_index: self.vector[*index].original_index,
                    })
                    .collect(),
                is_vertical: true,
            },
            indices
                .iter()
                .map(|index| self.vector[*index].value)
                .reduce(|acc, value| acc + value)
                .unwrap(),
        )
    }
    pub fn extract_horizontal_seam(&self, rng: &mut ThreadRng) -> (Seam, f32) {
        let mut dp_result = self
            .vector
            .iter()
            .map(|memory_point| memory_point.value)
            .collect::<Vec<f32>>();
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
                    .map(|index| MemoryPoint {
                        value: *index,
                        original_index: self.vector[*index].original_index,
                    })
                    .collect(),
                is_vertical: false,
            },
            indices
                .iter()
                .map(|index| self.vector[*index].value)
                .reduce(|acc, value| acc + value)
                .unwrap(),
        )
    }
}
