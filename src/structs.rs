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
                for row in 0..height {
                    let index = self.original_indices[row * self.width + column];
                    if current_remove_index < sorted_indices_to_remove.len()
                        && sorted_indices_to_remove[current_remove_index] == index
                    {
                        current_remove_index = current_remove_index + 1;
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
    pub fn carve_vertical_seams(&mut self, seams: Vec<Seam>) {
        let mut sorted_indices_to_remove = seams
            .iter()
            .cloned()
            .map(|seam| seam.indices)
            .collect::<Vec<Vec<usize>>>()
            .concat();
        sorted_indices_to_remove.sort();
        let mut sorted_indices_to_remove_iter = sorted_indices_to_remove.iter();

        let mut resulting_vector: Vec<T> =
            Vec::with_capacity(self.vector.len() - seams.len() * self.height());
        let mut resulting_original_indices: Vec<usize> =
            Vec::with_capacity(self.vector.len() - seams.len() * self.height());

        let mut index_to_remove = match sorted_indices_to_remove_iter.next() {
            None => {
                return;
            }
            Some(index_to_remove) => *index_to_remove,
        };

        for (index, value) in self.vector.iter().enumerate() {
            let original_index = self.original_indices[index];
            if index_to_remove == original_index {
                index_to_remove = match sorted_indices_to_remove_iter.next() {
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
            width: self.width - seams.len(),
            vector: resulting_vector,
            original_indices: resulting_original_indices,
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
