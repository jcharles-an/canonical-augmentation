use std::cmp;
use crate::schreier::group_element::GroupElement;
use crate::schreier::group_generators::GroupGenerators;
use crate::schreier::permutation::Permutation;

pub struct PermutationGenerators<T>
where T: GroupElement
{
    generators: Vec<T>,
    max_n: usize
}

impl GroupGenerators for PermutationGenerators<Permutation>{
    type Element = Permutation;

    fn get_identity(&self) -> Self::Element {
        Permutation::new((1..self.max_n).collect())
    }
}

impl PermutationGenerators<Permutation> {
    pub fn new(generators: Vec<Permutation>) -> PermutationGenerators<Permutation>{
        let max_n = generators.iter().map(|generator| generator.size).max().unwrap_or(0);
        PermutationGenerators{generators,max_n}
    }
}