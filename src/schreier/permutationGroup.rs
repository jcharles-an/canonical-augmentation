use std::collections::HashMap;
use crate::schreier::permutation::Permutation;

#[derive(Debug)]
pub struct PermutationGroup{
    generators: [[Permutation]]
}

impl PermutationGroup {
    pub fn from_generators(generators: &[&[Permutation]]) -> PermutationGroup {

    }
}