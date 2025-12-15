use std::cmp;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use crate::data_structs::consts::N;
use crate::data_structs::stabilizer_chain::StabChain;
use crate::groups::group::Group;
use crate::data_structs::permutation::Permutation;
use crate::groups::permutation_group::PermutationGroup;

pub trait GroupAction : Clone {
    type Target:Debug + Ord + Hash + Clone + Copy;
    type Group: Group;

    fn apply_to(element: &<Self::Group as Group>::Element, target: &Self::Target) -> Self::Target;
    fn get_index_from_target(&self, target: &Self::Target) -> usize;
    fn get_target_from_index(&self, index: usize) -> Self::Target;

    fn get_range(&self) -> usize;


}