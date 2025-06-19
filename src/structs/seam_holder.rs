use crate::structs::matrix::Seam;

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
                        self.vertical_seams.len()
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
                        self.horizontal_seams.len()
                    })..self.horizontal_seams.len(),
                    [],
                )
                .collect();
        }
    }
}

#[cfg(test)]
mod tests {
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
                .map(|seam| seam.indices.iter().cloned().collect())
                .collect::<Vec<Vec<usize>>>(),
            (8..11)
                .map(|index| (index..100).step_by(10).collect())
                .collect::<Vec<Vec<usize>>>()
        );

        assert_eq!(
            seam_holder
                .vertical_seams
                .iter()
                .map(|seam| seam.indices.iter().cloned().collect())
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
                .map(|seam| seam.indices.iter().cloned().collect())
                .collect::<Vec<Vec<usize>>>(),
            (8..11)
                .map(|index| ((index * 10)..(index * 10 + 10)).collect())
                .collect::<Vec<Vec<usize>>>()
        );

        assert_eq!(
            seam_holder
                .horizontal_seams
                .iter()
                .map(|seam| seam.indices.iter().cloned().collect())
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
                .map(|seam| seam.indices.iter().cloned().collect())
                .collect::<Vec<Vec<usize>>>(),
            expected
        );

        let popped_seams = seam_holder.pop_n_seams(3, false);
        assert_eq!(
            popped_seams
                .iter()
                .map(|seam| seam.indices.iter().cloned().collect())
                .collect::<Vec<Vec<usize>>>(),
            expected
        );
    }
}
