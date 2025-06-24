use crate::structs::matrix::MemoryPoint;

use super::*;

#[test]
pub fn vertical_seam_popping() {
    let mut seam_holder = SeamHolder::new();

    for index in 0..11 {
        seam_holder.push_seam(Seam {
            indices: (index..100)
                .step_by(10)
                .map(|index| MemoryPoint {
                    value: index,
                    original_index: index,
                })
                .collect(),
            is_vertical: true,
        });
    }

    let popped_seams = seam_holder.pop_n_seams(3, true);

    assert_eq!(
        popped_seams
            .iter()
            .map(|seam| seam
                .indices
                .iter()
                .map(|memory_point| memory_point.original_index)
                .collect())
            .collect::<Vec<Vec<usize>>>(),
        (8..11)
            .map(|index| (index..100).step_by(10).collect())
            .collect::<Vec<Vec<usize>>>()
    );

    assert_eq!(
        seam_holder
            .vertical_seams
            .iter()
            .map(|seam| seam
                .indices
                .iter()
                .map(|memory_point| memory_point.original_index)
                .collect())
            .collect::<Vec<Vec<usize>>>(),
        (0..8)
            .map(|index| (index..100).step_by(10).collect())
            .collect::<Vec<Vec<usize>>>()
    );
}
#[test]
pub fn horizontal_seam_popping() {
    let mut seam_holder = SeamHolder::new();

    for index in 0..11 {
        seam_holder.push_seam(Seam {
            indices: ((index * 10)..(index * 10 + 10))
                .map(|index| MemoryPoint {
                    value: index,
                    original_index: index,
                })
                .collect(),
            is_vertical: false,
        });
    }

    let popped_seams = seam_holder.pop_n_seams(3, false);

    assert_eq!(
        popped_seams
            .iter()
            .map(|seam| seam
                .indices
                .iter()
                .map(|memory_point| memory_point.original_index)
                .collect())
            .collect::<Vec<Vec<usize>>>(),
        (8..11)
            .map(|index| ((index * 10)..(index * 10 + 10)).collect())
            .collect::<Vec<Vec<usize>>>()
    );

    assert_eq!(
        seam_holder
            .horizontal_seams
            .iter()
            .map(|seam| seam
                .indices
                .iter()
                .map(|memory_point| memory_point.original_index)
                .collect())
            .collect::<Vec<Vec<usize>>>(),
        (0..8)
            .map(|index| ((index * 10)..(index * 10 + 10)).collect())
            .collect::<Vec<Vec<usize>>>()
    );
}

#[test]
pub fn seam_popping_overflow() {
    let mut seam_holder = SeamHolder::new();

    let expected: Vec<Vec<usize>> = vec![];

    let popped_seams = seam_holder.pop_n_seams(3, true);
    assert_eq!(
        popped_seams
            .iter()
            .map(|seam| seam
                .indices
                .iter()
                .map(|memory_point| memory_point.original_index)
                .collect())
            .collect::<Vec<Vec<usize>>>(),
        expected
    );

    let popped_seams = seam_holder.pop_n_seams(3, false);
    assert_eq!(
        popped_seams
            .iter()
            .map(|seam| seam
                .indices
                .iter()
                .map(|memory_point| memory_point.original_index)
                .collect())
            .collect::<Vec<Vec<usize>>>(),
        expected
    );
}
