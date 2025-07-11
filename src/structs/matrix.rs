use std::f32::INFINITY;

use crate::{structs::color::CustomColor, utils::GradientMagnitudePoint, *};
use ::rand::{rngs::ThreadRng, Rng};

#[cfg(test)]
#[path = "../tests/matrix.rs"]
mod matrix_tests;

#[derive(Clone)]
pub struct HorizontalSeam {
    pub rows: Vec<usize>,
}
#[derive(Clone)]
pub struct VerticalSeam {
    pub columns: Vec<usize>,
}

#[derive(Clone)]
pub struct Matrix<T> {
    width: usize,
    pub vector: Vec<T>,
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
            vector: vector,
        }
    }
    pub fn carve_horizontal_seam(&mut self, seam: &HorizontalSeam) {
        let column_vectors: Vec<Vec<T>> = (0..self.width)
            .into_iter()
            .map(|column| {
                let mut vector_result: Vec<T> = self
                    .vector
                    .iter()
                    .skip(column)
                    .step_by(self.width)
                    .cloned()
                    .collect();
                let row = seam.rows[column];
                vector_result.remove(row);

                return vector_result;
            })
            .collect::<Vec<Vec<T>>>();

        let result = (0..(self.height() - 1))
            .into_iter()
            .map(|row| {
                column_vectors
                    .iter()
                    .map(|column_vector| column_vector[row])
                    .collect::<Vec<T>>()
            })
            .collect::<Vec<Vec<T>>>()
            .concat();

        self.vector = result;
    }
    pub fn carve_vertical_seam(&mut self, seam: &VerticalSeam) {
        let resulting_vector: Vec<T> = (0..self.height())
            .map(|row| {
                let mut row_vector = self
                    .vector
                    .iter()
                    .skip(row * self.width)
                    .take(self.width)
                    .cloned()
                    .collect::<Vec<T>>();
                let column = seam.columns[row];
                row_vector.remove(column);
                return row_vector;
            })
            .collect::<Vec<Vec<T>>>()
            .concat();

        self.vector = resulting_vector;
        self.width = self.width - 1;
    }
}

impl Matrix<GradientMagnitudePoint> {
    pub fn extract_vertical_seam(
        &self,
        rng: &mut ThreadRng,
        avoid_inserted: bool,
    ) -> (VerticalSeam, f32) {
        let mut dp_result = self
            .vector
            .iter()
            .map(|point| {
                if point.is_inserted {
                    if avoid_inserted {
                        INFINITY
                    } else {
                        0.0
                    }
                } else {
                    point.value
                }
            })
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

        let mut columns = vec![0; height];
        let mut total_energy = 0.0;

        // calculate the last element in seam by randomly
        // selecting one of the minimum points in the last row
        let mut min_columns = Vec::with_capacity(width);
        let mut current_min = dp_result[(height - 1) * width];
        dp_result
            .iter()
            .skip((height - 1) * width)
            .enumerate()
            .for_each(|(column, value)| {
                if *value < current_min {
                    min_columns.truncate(0);
                    min_columns.push(column);
                    current_min = *value;
                } else if *value == current_min {
                    min_columns.push(column);
                }
            });
        let last_column = min_columns[rng.gen_range(0..min_columns.len())];
        columns[height - 1] = last_column;
        total_energy =
            total_energy + self.vector[self.vector.len() - self.width + last_column].value;

        // calculate the rest of the indexes for the seam
        for row in (0..height - 1).rev() {
            let column = {
                let mid_column = columns[row + 1];
                let left_column = if mid_column == 0 {
                    mid_column
                } else {
                    mid_column - 1
                };

                let mut min_column = if dp_result[row * self.width + left_column]
                    < dp_result[row * self.width + mid_column]
                {
                    left_column
                } else {
                    mid_column
                };

                let right_column = if mid_column == width - 1 {
                    mid_column
                } else {
                    mid_column + 1
                };

                min_column = if dp_result[row * self.width + min_column]
                    < dp_result[row * self.width + right_column]
                {
                    min_column
                } else {
                    right_column
                };
                min_column
            };
            columns[row] = column;
            total_energy = total_energy + self.vector[self.width * row + column].value;
        }

        (VerticalSeam { columns: columns }, total_energy)
    }
    pub fn extract_horizontal_seam(
        &self,
        rng: &mut ThreadRng,
        avoid_inserted: bool,
    ) -> (HorizontalSeam, f32) {
        let mut dp_result = self
            .vector
            .iter()
            .map(|point| {
                if point.is_inserted {
                    if avoid_inserted {
                        INFINITY
                    } else {
                        0.0
                    }
                } else {
                    point.value
                }
            })
            .collect::<Vec<f32>>();
        let width = self.width;
        let height = self.vector.len() / width;

        // fill in the vector using dynamic programming
        for j in 1..width {
            for i in 0..height {
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

        let mut rows = vec![0; width];
        let mut total_energy = 0.0;

        // calculate the last element in seam by randomly
        // selecting one of the minimum points in the last column
        let mut min_rows = Vec::with_capacity(height);
        let mut current_min = dp_result[width - 1];
        dp_result
            .iter()
            .skip(width - 1)
            .step_by(width)
            .enumerate()
            .for_each(|(row, value)| {
                if *value < current_min {
                    min_rows.truncate(0);
                    min_rows.push(row);
                    current_min = *value;
                } else if *value == current_min {
                    min_rows.push(row);
                }
            });
        let last_row = min_rows[rng.gen_range(0..min_rows.len())];
        rows[width - 1] = last_row;
        total_energy = total_energy + self.vector[self.width * (last_row + 1) - 1].value;

        // calculate the rest of the indexes for the seam
        for column in (0..width - 1).rev() {
            let row = {
                let mid_row = rows[column + 1];
                let top_row = if mid_row > 0 { mid_row - 1 } else { mid_row };

                let mut min_row = if dp_result[self.width * top_row + column]
                    < dp_result[self.width * mid_row + column]
                {
                    top_row
                } else {
                    mid_row
                };

                let bottom_row = if mid_row >= height - 1 {
                    mid_row
                } else {
                    mid_row + 1
                };

                min_row = if dp_result[self.width * min_row + column]
                    < dp_result[self.width * bottom_row + column]
                {
                    min_row
                } else {
                    bottom_row
                };

                min_row
            };
            rows[column] = row;
            total_energy = total_energy + self.vector[self.width * row + column].value;
        }

        (HorizontalSeam { rows: rows }, total_energy)
    }
}

impl Matrix<CustomColor> {
    pub fn insert_vertical_seam(&mut self, seam: &VerticalSeam) {
        let columns = &seam.columns;

        let resulting_vector = columns
            .iter()
            .cloned()
            .enumerate()
            .map(|(row, column)| {
                let avg = {
                    let fold = self
                        .vector
                        .iter()
                        .skip(self.width * row + (if column > 0 { column - 1 } else { column }))
                        .take((self.width - column).min(if column > 0 { 3 } else { 2 }))
                        .fold((0, (0.0, 0.0, 0.0)), |acc, value| {
                            (
                                acc.0 + 1,
                                (acc.1 .0 + value.r, acc.1 .1 + value.g, acc.1 .2 + value.b),
                            )
                        });
                    CustomColor {
                        r: fold.1 .0 / (fold.0 as f32),
                        g: fold.1 .1 / (fold.0 as f32),
                        b: fold.1 .2 / (fold.0 as f32),
                        is_inserted: true,
                    }
                };

                let mut row_vector = self
                    .vector
                    .iter()
                    .cloned()
                    .skip(self.width * row)
                    .take(self.width)
                    .collect::<Vec<CustomColor>>();
                row_vector.insert(column + 1, avg);
                row_vector[column].is_inserted = true;
                if column < row_vector.len() - 2 {
                    row_vector[column + 2].is_inserted = true;
                }
                return row_vector;
            })
            .collect::<Vec<Vec<CustomColor>>>()
            .concat();

        self.vector = resulting_vector;
        self.width = self.width + 1;
    }
    pub fn insert_horizontal_seam(&mut self, seam: &HorizontalSeam) {
        let height = self.height();

        let rows = &seam.rows;

        let column_vectors = rows
            .iter()
            .cloned()
            .enumerate()
            .map(|(column, row)| {
                let avg = {
                    let starting_row = if row > 0 { row - 1 } else { row };
                    let fold = self
                        .vector
                        .iter()
                        .skip(self.width * starting_row + column)
                        .step_by(self.width)
                        .take((height - starting_row).min(if row > 0 { 3 } else { 2 }))
                        .fold((0, (0.0, 0.0, 0.0)), |acc, value| {
                            (
                                acc.0 + 1,
                                (acc.1 .0 + value.r, acc.1 .1 + value.g, acc.1 .2 + value.b),
                            )
                        });
                    CustomColor {
                        r: fold.1 .0 / (fold.0 as f32),
                        g: fold.1 .1 / (fold.0 as f32),
                        b: fold.1 .2 / (fold.0 as f32),
                        is_inserted: true,
                    }
                };

                let mut column_vector = self
                    .vector
                    .iter()
                    .cloned()
                    .skip(column)
                    .step_by(self.width)
                    .collect::<Vec<CustomColor>>();
                column_vector.insert(row + 1, avg);
                column_vector[row].is_inserted = true;
                if row < column_vector.len() - 2 {
                    column_vector[row + 2].is_inserted = true;
                }
                return column_vector;
            })
            .collect::<Vec<Vec<CustomColor>>>();

        let result = (0..(height + 1))
            .into_iter()
            .map(|row| {
                column_vectors
                    .iter()
                    .map(|column_vector| column_vector[row])
                    .collect::<Vec<CustomColor>>()
            })
            .collect::<Vec<Vec<CustomColor>>>()
            .concat();
        self.vector = result;
    }
}
