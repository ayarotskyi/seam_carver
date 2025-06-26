use ::rand::thread_rng;
use std::fmt::Debug;

use crate::structs::{
    matrix::{Matrix, MemoryPoint, Seam},
    seam_holder::SeamHolder,
};

impl<T: PartialEq> PartialEq for Matrix<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.vector.len() != other.vector.len() || self.width != other.width {
            return false;
        }

        for index in 0..self.vector.len() {
            if self.vector[index].value != other.vector[index].value
                && self.vector[index].original_index != other.vector[index].original_index
            {
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
    pub fn to_string(&self) -> String {
        let ansi_code = self.to_ansi_code();
        return format!("\x1b[30;{}m  \x1b[0m", ansi_code);
    }
}

impl Debug for BgColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.to_string();
        write!(f, "{}", string)
    }
}

fn draw_matrix(matrix: Matrix<BgColor>) -> String {
    let mut result = String::new();

    for row in matrix.vector.chunks(matrix.width) {
        for bg_color in row {
            result.push_str(&bg_color.value.to_string());
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
fn horizontal_carving() {
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
fn vertical_carving() {
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
fn vertical_seam_carving_unordered() {
    let matrix = Matrix::from_memory_points(
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
        ])
        .iter()
        .zip(Vec::from([
            0, 5, 2, 3, 4, 9, 6, 7, 8, 13, 10, 11, 12, 17, 14, 15,
        ]))
        .map(|(color, original_index)| MemoryPoint {
            value: *color,
            original_index: original_index,
        })
        .collect(),
        4,
    );
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
fn horizontal_seam_extraction() {
    let mut rng = thread_rng();
    let energy_matrix: Matrix<f32> =
        Matrix::new(Vec::from([0.0, 1.0, 3.0, 2.0, 0.0, 1.0, 3.0, 2.0, 0.0]), 3);
    let (seam, _) = energy_matrix.extract_horizontal_seam(&mut rng);
    assert_eq!(seam.indices, [0, 4, 8]);
}

#[test]
fn vertical_seam_extraction() {
    let mut rng = thread_rng();
    let energy_matrix: Matrix<f32> =
        Matrix::new(Vec::from([0.0, 1.0, 3.0, 2.0, 0.0, 1.0, 3.0, 2.0, 0.0]), 3);
    let (seam, _) = energy_matrix.extract_vertical_seam(&mut rng);
    assert_eq!(seam.indices, [0, 4, 8]);
}

#[test]
fn vertical_seam_recovery() {
    let matrix = Matrix::from_memory_points(
        [
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
        ]
        .iter()
        .zip(Vec::from([1, 2, 3, 4, 6, 7, 8, 9, 11, 12, 13, 14]))
        .map(|(color, original_index)| MemoryPoint {
            value: *color,
            original_index: original_index,
        })
        .collect(),
        3,
    );

    let seam = Seam {
        indices: Vec::from([0, 5, 10, 15]),
        is_vertical: true,
    };

    let original_matrix = Matrix::new(
        Vec::from([
            BgColor::Blue,
            BgColor::Red,
            BgColor::Green,
            BgColor::Yellow,
            BgColor::Blue,
            BgColor::Yellow,
            BgColor::Red,
            BgColor::White,
            BgColor::Cyan,
            BgColor::Black,
            BgColor::White,
            BgColor::Yellow,
            BgColor::Cyan,
            BgColor::Red,
            BgColor::Green,
            BgColor::Blue,
        ]),
        4,
    );

    let mut output = matrix.clone();
    output.recover_vertical_seam(&seam, &original_matrix);

    assert_matrices_equal(matrix, output, original_matrix);
}

#[test]
fn vertical_seam_recovery_unordered() {
    let matrix = Matrix::from_memory_points(
        Vec::from([
            BgColor::Yellow,
            BgColor::Yellow,
            BgColor::Red,
            BgColor::Cyan,
        ])
        .iter()
        .zip(Vec::from([3, 5, 6, 8]))
        .map(|(color, original_index)| MemoryPoint {
            value: *color,
            original_index: original_index,
        })
        .collect(),
        2,
    );

    let seam = Seam {
        indices: Vec::from([1, 7]),
        is_vertical: true,
    };

    let original_matrix = Matrix::new(
        Vec::from([
            BgColor::Blue,
            BgColor::Red,
            BgColor::Green,
            BgColor::Yellow,
            BgColor::Blue,
            BgColor::Yellow,
            BgColor::Red,
            BgColor::White,
            BgColor::Cyan,
        ]),
        3,
    );

    let expected_output = Matrix::new(
        Vec::from([
            BgColor::Yellow,
            BgColor::Red,
            BgColor::Yellow,
            BgColor::Red,
            BgColor::White,
            BgColor::Cyan,
        ]),
        3,
    );

    let mut output = matrix.clone();
    output.recover_vertical_seam(&seam, &original_matrix);

    assert_matrices_equal(matrix, output, expected_output);
}

#[test]
fn horizontal_seam_recovery() {
    let matrix = Matrix::from_memory_points(
        Vec::from([
            BgColor::Yellow,
            BgColor::Red,
            BgColor::Yellow,
            BgColor::Red,
            BgColor::White,
            BgColor::Cyan,
        ])
        .iter()
        .zip(Vec::from([1, 3, 5, 6, 7, 8]))
        .map(|(color, original_index)| MemoryPoint {
            value: *color,
            original_index: original_index,
        })
        .collect(),
        3,
    );

    let seam = Seam {
        indices: Vec::from([0, 4, 2]),
        is_vertical: false,
    };

    let original_matrix = Matrix::new(
        Vec::from([
            BgColor::Blue,
            BgColor::Red,
            BgColor::Green,
            BgColor::Yellow,
            BgColor::Blue,
            BgColor::Yellow,
            BgColor::Red,
            BgColor::White,
            BgColor::Cyan,
        ]),
        3,
    );

    let mut output = matrix.clone();
    output.recover_horizontal_seam(&seam, &original_matrix);

    assert_matrices_equal(matrix, output, original_matrix);
}

#[test]
fn mixed_seam_recovery() {
    let matrix = Matrix::from_memory_points(
        Vec::from([
            BgColor::Yellow,
            BgColor::Yellow,
            BgColor::Red,
            BgColor::Cyan,
        ])
        .iter()
        .zip(Vec::from([3, 5, 6, 8]))
        .map(|(color, original_index)| MemoryPoint {
            value: *color,
            original_index: original_index,
        })
        .collect(),
        2,
    );

    let horizontal_seam = Seam {
        indices: Vec::from([0, 4, 2]),
        is_vertical: false,
    };
    let vertical_seam = Seam {
        indices: Vec::from([1, 7]),
        is_vertical: true,
    };

    let original_matrix = Matrix::new(
        Vec::from([
            BgColor::Blue,
            BgColor::Red,
            BgColor::Green,
            BgColor::Yellow,
            BgColor::Blue,
            BgColor::Yellow,
            BgColor::Red,
            BgColor::White,
            BgColor::Cyan,
        ]),
        3,
    );

    let mut output = matrix.clone();
    output.recover_vertical_seam(&vertical_seam, &original_matrix);
    output.recover_horizontal_seam(&horizontal_seam, &original_matrix);

    assert_matrices_equal(matrix, output, original_matrix);
}

// in case when the seam is longer than the matrix' side
// we'll need to insert overflowing indices into perpendicular seams
// to ensure that their lengths match the new size of the matrix
#[test]
fn overflowing_seam_recovery() {
    let matrix = Matrix::from_memory_points(
        Vec::from([
            BgColor::Yellow,
            BgColor::Yellow,
            BgColor::Red,
            BgColor::Cyan,
        ])
        .iter()
        .zip(Vec::from([3, 5, 6, 8]))
        .map(|(color, original_index)| MemoryPoint {
            value: *color,
            original_index: original_index,
        })
        .collect(),
        2,
    );
    let mut seam_holder = SeamHolder::new();
    seam_holder.push_seam(Seam {
        indices: Vec::from([0, 4, 2]),
        is_vertical: false,
    });
    seam_holder.push_seam(Seam {
        indices: Vec::from([1, 7]),
        is_vertical: true,
    });

    let original_matrix = Matrix::new(
        Vec::from([
            BgColor::Blue,
            BgColor::Red,
            BgColor::Green,
            BgColor::Yellow,
            BgColor::Blue,
            BgColor::Yellow,
            BgColor::Red,
            BgColor::White,
            BgColor::Cyan,
        ]),
        3,
    );

    let mut output = matrix.clone();
    let overflowing_indices =
        output.recover_horizontal_seam(&seam_holder.pop_n_seams(1, false)[0], &original_matrix);
    seam_holder.insert_overflowing_indices(overflowing_indices, true);

    output.recover_vertical_seam(&seam_holder.pop_n_seams(1, true)[0], &original_matrix);

    assert_matrices_equal(matrix, output, original_matrix);
}
