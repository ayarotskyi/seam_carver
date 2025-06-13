use crate::*;

#[derive(Clone)]
pub struct WindowSize {
    pub height: usize,
    pub width: usize,
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
    pub fn carve_horizontal_seams(&mut self, seams: Vec<Seam>) {
        let height = self.height();
        let resulting_height = height - seams.len();

        let column_vectors: Vec<(Vec<T>, Vec<usize>)> = (0..self.width)
            .into_par_iter()
            .map(|column| {
                let mut sorted_indices_to_remove = seams
                    .iter()
                    .map(|seam| seam.indices[column])
                    .collect::<Vec<usize>>();
                sorted_indices_to_remove.sort();

                let mut current_remove_index = 0;

                let mut vector_result = Vec::with_capacity(resulting_height);
                let mut original_indices_result = Vec::with_capacity(resulting_height);
                let mut count = 0;
                for row in 0..height {
                    let index = self.original_indices[row * self.width + column];
                    if current_remove_index < sorted_indices_to_remove.len()
                        && sorted_indices_to_remove[current_remove_index] == index
                    {
                        current_remove_index = current_remove_index + 1;
                        count = count + 1;
                    } else {
                        vector_result.push(self.vector[row * self.width + column]);
                        original_indices_result.push(index);
                    }
                }

                return (vector_result, original_indices_result);
            })
            .collect::<Vec<(Vec<T>, Vec<usize>)>>();

        let result = (0..resulting_height)
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

#[derive(Clone)]
pub struct Seam {
    pub indices: Vec<usize>,
}

pub struct SeamHolder {
    pub vertical_seams: Vec<Seam>,
    pub horizontal_seams: Vec<Seam>,
}
