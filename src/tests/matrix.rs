use ::rand::thread_rng;
use macroquad::color::Color;
use std::fmt::Debug;

use crate::structs::matrix::{HorizontalSeam, Matrix, VerticalSeam};

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
            result.push_str(&bg_color.to_string());
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
    output.carve_horizontal_seam(&HorizontalSeam {
        rows: vec![0, 1, 1, 2],
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
    output.carve_vertical_seam(&VerticalSeam {
        columns: vec![0, 1, 2, 3],
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
    let (seam, total_energy) = energy_matrix.extract_horizontal_seam(&mut rng);
    assert_eq!(seam.rows, [0, 1, 2]);
    assert_eq!(total_energy, 0.0);
}

#[test]
fn vertical_seam_extraction() {
    let mut rng = thread_rng();
    let energy_matrix: Matrix<f32> =
        Matrix::new(Vec::from([0.0, 1.0, 3.0, 2.0, 0.0, 1.0, 3.0, 2.0, 0.0]), 3);
    let (seam, total_energy) = energy_matrix.extract_vertical_seam(&mut rng);
    assert_eq!(seam.columns, [0, 1, 2]);
    assert_eq!(total_energy, 0.0);
}

#[test]
fn vertical_seam_insertion() {
    let first_color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 100.0,
    };
    let second_color = Color {
        r: 100.0,
        g: 100.0,
        b: 100.0,
        a: 100.0,
    };
    let mid_color = Color {
        r: 50.0,
        g: 50.0,
        b: 50.0,
        a: 100.0,
    };
    let mut matrix = Matrix::new(
        Vec::from([
            first_color,
            second_color,
            second_color,
            first_color,
            second_color,
            second_color,
            first_color,
            second_color,
            second_color,
        ]),
        3,
    );

    let seam = VerticalSeam {
        columns: Vec::from([0, 0, 0]),
    };
    matrix.insert_vertical_seam(seam);

    assert_eq!(
        matrix.vector,
        Vec::from([
            first_color,
            mid_color,
            second_color,
            second_color,
            first_color,
            mid_color,
            second_color,
            second_color,
            first_color,
            mid_color,
            second_color,
            second_color,
        ])
    );
}

#[test]
fn horizontal_seam_insertion() {
    let first_color = Color {
        r: 50.0,
        g: 50.0,
        b: 50.0,
        a: 100.0,
    };
    let second_color = Color {
        r: 100.0,
        g: 100.0,
        b: 100.0,
        a: 100.0,
    };
    let third_color = Color {
        r: 150.0,
        g: 150.0,
        b: 150.0,
        a: 100.0,
    };
    let mut matrix = Matrix::new(
        Vec::from([
            first_color,
            second_color,
            third_color,
            second_color,
            first_color,
            third_color,
            third_color,
            second_color,
            third_color,
        ]),
        3,
    );

    let seam = HorizontalSeam {
        rows: Vec::from([1, 0, 1]),
    };
    matrix.insert_horizontal_seam(seam);

    assert_eq!(
        matrix.vector,
        Vec::from([
            first_color,
            second_color,
            third_color,
            second_color,
            Color {
                r: 75.0,
                g: 75.0,
                b: 75.0,
                a: 100.0
            },
            third_color,
            second_color,
            first_color,
            third_color,
            third_color,
            second_color,
            third_color,
        ])
    );
}
