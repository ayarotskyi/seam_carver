use crate::*;
use ::rand::{rngs::ThreadRng, Rng};

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

#[cfg(test)]
mod tests {
    use ::rand::thread_rng;

    use super::*;

    impl<T: PartialEq> PartialEq for Matrix<T> {
        fn eq(&self, other: &Self) -> bool {
            if self.vector.len() != other.vector.len() || self.width != other.width {
                return false;
            }

            for index in 0..self.vector.len() {
                if self.vector[index] != other.vector[index] {
                    return false;
                }
            }

            return true;
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    enum BgColor {
        Black,
        Red,
        Green,
        Yellow,
        Blue,
        Magenta,
        Cyan,
        White,
    }

    impl BgColor {
        fn to_ansi_code(&self) -> u8 {
            match self {
                BgColor::Black => 40,
                BgColor::Red => 41,
                BgColor::Green => 42,
                BgColor::Yellow => 43,
                BgColor::Blue => 44,
                BgColor::Magenta => 45,
                BgColor::Cyan => 46,
                BgColor::White => 47,
            }
        }
    }

    fn with_background(text: &str, bg_color: &BgColor) -> String {
        let ansi_code = bg_color.to_ansi_code();
        format!("\x1b[30;{}m{}\x1b[0m", ansi_code, text)
    }

    fn draw_matrix(matrix: Matrix<BgColor>) -> String {
        let mut result = String::new();

        for row in matrix.vector.chunks(matrix.width) {
            for bg_color in row {
                result.push_str(&with_background("  ", bg_color));
            }
            result.push_str("\n");
        }

        return result;
    }

    fn assert_matrices_equal(
        input: Matrix<BgColor>,
        output: Matrix<BgColor>,
        expected: Matrix<BgColor>,
    ) {
        assert!(
            output == expected,
            "Original: \n{}Output: \n{}Expected: \n{}",
            draw_matrix(input),
            draw_matrix(output),
            draw_matrix(expected)
        );
    }

    #[test]
    fn test_horizontal_carving() {
        let matrix = Matrix::new(
            Vec::from([
                BgColor::Black,
                BgColor::Red,
                BgColor::Green,
                BgColor::Yellow,
                BgColor::Blue,
                BgColor::Magenta,
                BgColor::Red,
                BgColor::White,
                BgColor::Cyan,
                BgColor::Black,
                BgColor::Blue,
                BgColor::Yellow,
                BgColor::Cyan,
                BgColor::Red,
                BgColor::Green,
                BgColor::White,
            ]),
            4,
        );
        let mut output = matrix.clone();
        output.carve_seam(&Seam {
            indices: vec![0, 5, 6, 11],
            is_vertical: false,
        });
        assert_matrices_equal(
            matrix,
            output,
            Matrix::new(
                Vec::from([
                    BgColor::Blue,
                    BgColor::Red,
                    BgColor::Green,
                    BgColor::Yellow,
                    BgColor::Cyan,
                    BgColor::Black,
                    BgColor::Blue,
                    BgColor::White,
                    BgColor::Cyan,
                    BgColor::Red,
                    BgColor::Green,
                    BgColor::White,
                ]),
                4,
            ),
        );
    }

    #[test]
    fn test_vertical_carving() {
        let matrix = Matrix::new(
            Vec::from([
                BgColor::Black,
                BgColor::Red,
                BgColor::Green,
                BgColor::Yellow,
                BgColor::Blue,
                BgColor::Magenta,
                BgColor::Red,
                BgColor::White,
                BgColor::Cyan,
                BgColor::Black,
                BgColor::Blue,
                BgColor::Yellow,
                BgColor::Cyan,
                BgColor::Red,
                BgColor::Green,
                BgColor::White,
            ]),
            4,
        );
        let mut output = matrix.clone();
        output.carve_seam(&Seam {
            indices: vec![0, 5, 10, 15],
            is_vertical: true,
        });
        assert_matrices_equal(
            matrix,
            output,
            Matrix::new(
                Vec::from([
                    BgColor::Red,
                    BgColor::Green,
                    BgColor::Yellow,
                    BgColor::Blue,
                    BgColor::Red,
                    BgColor::White,
                    BgColor::Cyan,
                    BgColor::Black,
                    BgColor::Yellow,
                    BgColor::Cyan,
                    BgColor::Red,
                    BgColor::Green,
                ]),
                3,
            ),
        );
    }

    // after horizontal carving not all original indices are ordered,
    // which caused bugs in previous versions of the vertical carver
    #[test]
    fn test_vertical_incorrect_original_index_order() {
        let matrix = Matrix {
            vector: Vec::from([
                BgColor::Black,
                BgColor::Red,
                BgColor::Green,
                BgColor::Yellow,
                BgColor::Blue,
                BgColor::Magenta,
                BgColor::Red,
                BgColor::White,
                BgColor::Cyan,
                BgColor::Black,
                BgColor::Blue,
                BgColor::Yellow,
                BgColor::Cyan,
                BgColor::Red,
                BgColor::Green,
                BgColor::White,
            ]),
            width: 4,
            original_indices: Vec::from([0, 5, 2, 3, 4, 9, 6, 7, 8, 13, 10, 11, 12, 17, 14, 15]),
        };
        let mut output = matrix.clone();
        output.carve_seam(&Seam {
            indices: vec![0, 9, 10, 15],
            is_vertical: true,
        });
        assert_matrices_equal(
            matrix,
            output,
            Matrix::new(
                Vec::from([
                    BgColor::Red,
                    BgColor::Green,
                    BgColor::Yellow,
                    BgColor::Blue,
                    BgColor::Red,
                    BgColor::White,
                    BgColor::Cyan,
                    BgColor::Black,
                    BgColor::Yellow,
                    BgColor::Cyan,
                    BgColor::Red,
                    BgColor::Green,
                ]),
                3,
            ),
        );
    }

    #[test]
    fn test_horizontal_seam_extraction() {
        let mut rng = thread_rng();
        let energy_matrix: Matrix<f32> =
            Matrix::new(Vec::from([0.0, 1.0, 3.0, 2.0, 0.0, 1.0, 3.0, 2.0, 0.0]), 3);
        let (seam, _) = energy_matrix.extract_horizontal_seam(&mut rng);
        assert_eq!(seam.indices, [0, 4, 8]);
    }

    #[test]
    fn test_vertical_seam_extraction() {
        let mut rng = thread_rng();
        let energy_matrix: Matrix<f32> =
            Matrix::new(Vec::from([0.0, 1.0, 3.0, 2.0, 0.0, 1.0, 3.0, 2.0, 0.0]), 3);
        let (seam, _) = energy_matrix.extract_vertical_seam(&mut rng);
        assert_eq!(seam.indices, [0, 4, 8]);
    }
}
