#[derive(Clone)]
pub struct WindowSize {
    pub height: usize,
    pub width: usize,
}

#[derive(Clone)]
pub struct Matrix<T> {
    pub width: usize,
    pub vector: Vec<T>,
}

pub struct Seam {
    pub indices: Vec<usize>,
    pub total_energy: f32,
}
impl Eq for Seam {}
impl PartialEq for Seam {
    fn eq(&self, other: &Self) -> bool {
        self.total_energy == other.total_energy
    }
}
impl PartialOrd for Seam {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.total_energy > other.total_energy {
            Some(std::cmp::Ordering::Greater)
        } else if self.total_energy < other.total_energy {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}
impl Ord for Seam {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.total_energy > other.total_energy {
            std::cmp::Ordering::Greater
        } else if self.total_energy < other.total_energy {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

pub struct SeamHolder {
    pub vertical_seams: Vec<Seam>,
    pub horizontal_seams: Vec<Seam>,
}
