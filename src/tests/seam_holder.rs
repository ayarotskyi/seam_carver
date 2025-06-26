use super::*;

#[test]
pub fn vertical_seam_popping() {
    let mut seam_holder = SeamHolder::new();

    for index in 0..11 {
        seam_holder.push_seam(Seam {
            indices: (index..100).step_by(10).collect(),
            is_vertical: true,
        });
    }

    let popped_seams = seam_holder.pop_n_seams(3, true);

    assert_eq!(
        popped_seams
            .iter()
            .map(|seam| seam.indices.clone())
            .collect::<Vec<Vec<usize>>>(),
        (8..11)
            .map(|index| (index..100).step_by(10).collect())
            .collect::<Vec<Vec<usize>>>()
    );

    assert_eq!(
        seam_holder
            .vertical_seams
            .iter()
            .map(|seam| seam.indices.clone())
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
            indices: ((index * 10)..(index * 10 + 10)).collect(),
            is_vertical: false,
        });
    }

    let popped_seams = seam_holder.pop_n_seams(3, false);

    assert_eq!(
        popped_seams
            .iter()
            .map(|seam| seam.indices.clone())
            .collect::<Vec<Vec<usize>>>(),
        (8..11)
            .map(|index| ((index * 10)..(index * 10 + 10)).collect())
            .collect::<Vec<Vec<usize>>>()
    );

    assert_eq!(
        seam_holder
            .horizontal_seams
            .iter()
            .map(|seam| seam.indices.clone())
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
            .map(|seam| seam.indices.clone())
            .collect::<Vec<Vec<usize>>>(),
        expected
    );

    let popped_seams = seam_holder.pop_n_seams(3, false);
    assert_eq!(
        popped_seams
            .iter()
            .map(|seam| seam.indices.clone())
            .collect::<Vec<Vec<usize>>>(),
        expected
    );
}

#[test]
pub fn vertical_overflowing_indices() {
    let mut seam_holder = SeamHolder::new();
    seam_holder.push_seam(Seam {
        indices: [
            (0..5).collect::<Vec<usize>>(),
            (6..10).collect::<Vec<usize>>(),
        ]
        .concat(),
        is_vertical: true,
    });

    let overflowing_indices: Vec<usize> = Vec::from([5]);

    seam_holder.insert_overflowing_indices(overflowing_indices, true);

    assert_eq!(
        seam_holder.vertical_seams[0].indices,
        (0..10).collect::<Vec<usize>>()
    );
}

#[test]
fn horizontal_overflowing_indices() {
    let mut seam_holder = SeamHolder::new();
    seam_holder.push_seam(Seam {
        indices: [
            (0..5).collect::<Vec<usize>>(),
            (6..10).collect::<Vec<usize>>(),
        ]
        .concat(),
        is_vertical: false,
    });
    seam_holder.push_seam(Seam {
        indices: [
            (10..17).collect::<Vec<usize>>(),
            (18..20).collect::<Vec<usize>>(),
        ]
        .concat(),
        is_vertical: false,
    });

    let overflowing_indices: Vec<usize> = Vec::from([5, 17]);

    seam_holder.insert_overflowing_indices(overflowing_indices, false);

    assert_eq!(
        seam_holder.pop_n_seams(1, false)[0].indices,
        (10..20).collect::<Vec<usize>>()
    );
    assert_eq!(
        seam_holder.pop_n_seams(1, false)[0].indices,
        (0..10).collect::<Vec<usize>>()
    );
}
