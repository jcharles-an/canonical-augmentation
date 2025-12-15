use std::cmp;
use std::collections::HashSet;
use crate::data_structs::bitmaps::{BMC};
use crate::data_structs::consts::{N};
use crate::group_actions::groupAction::GroupAction;
use crate::data_structs::permutation::Permutation;
use crate::groups::permutation_group::PermutationGroup;
use crate::schreier::schreier::Schreier;

#[derive(Clone, Debug)]
pub struct StabChain<T> {
    pub chain: Vec<Vec<T>>,
    pub base: T,

    v_3: Vec<usize>
}

impl <T: Clone> StabChain<T> {
    pub fn new(n:usize, identity: T, base: T) -> StabChain<T> {
        let chain = vec![vec![identity]; n];
        StabChain{
            chain,
            base,
            v_3: vec![0; n]
        }
    }

}