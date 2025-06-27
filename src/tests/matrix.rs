use ::rand::thread_rng;
use std::fmt::Debug;

use crate::structs::matrix::{Matrix, MemoryPoint, Seam};

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
