#[derive(Debug,Clone)]
pub struct Permutation {
    pub map: Vec<usize>,
    pub size: usize
}
impl Permutation{
    pub fn new(map: Vec<usize>) -> Permutation{
        Permutation{size: map.len(), map}
    }
}

