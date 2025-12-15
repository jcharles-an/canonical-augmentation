use std::cmp;
use std::collections::{HashMap, HashSet};
use num_integer::binomial;
use crate::data_structs::bitmaps::{slice_into_BM64, BM64_as_set, BM64};
use crate::data_structs::consts::N;
use crate::data_structs::stabilizer_chain::StabChain;
use crate::groups::group::Group;
use crate::group_actions::groupAction::GroupAction;
use crate::data_structs::permutation::Permutation;
use crate::groups::permutation_group::PermutationGroup;
use crate::schreier::schreier::Schreier;

#[derive(Debug, Clone)]
pub struct PermuteSubset
{
    range: usize,
    n: usize,
    k: usize,
}
impl GroupAction for PermuteSubset{
    type Target = BM64;
    type Group = PermutationGroup;
    fn apply_to(element: &Permutation, target: &Self::Target) -> Self::Target {
        let mut permuted = BM64::new();
        for index in target{
            permuted.set(*element.map.get(index).unwrap(), true);
        }
        permuted
    }
    fn get_index_from_target(&self, target: &Self::Target) -> usize {
        //colex rank for k subsets
        let mut rank = 0;
        for (i, val) in target.into_iter().rev().enumerate() {
            rank +=  num_integer::binomial(val, self.k - i);
        }
        rank
    }
    fn get_target_from_index(&self, index: usize) -> Self::Target{
        //colex un rank for k subsets
        let mut target = BM64::new();
        let mut rank = index;

        let mut x = self.range;
        for i in 0..self.k{
            let mut binomial = num_integer::binomial(x, self.k - i);
            while binomial > rank{
                x -= 1;
                binomial = num_integer::binomial(x, self.k - i)
            }
            target.set(x, true);
            rank -=  binomial;
        }

        target
    }

    fn get_range(&self) -> usize {
        self.range
    }
}


impl PermuteSubset {

    pub fn new(n:usize, k:usize) -> PermuteSubset {
        PermuteSubset{
            range: binomial(n, k),
            n,
            k,
        }
    }
}

#[test]
fn test_schreir() {
    /*
    let mut schreier = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![0 ,1 ,3 ,7 ,6,4], vec![2,5]]),
                Permutation::from_cycles(vec![vec![0 ,1 ,3 ,2], vec![4,5,7,6]]),
            ]));

    schreier.simple_gen();

    let stab_chain = schreier.generate_stab_chain();

    println!("Generated group with order {} with the stab chain:", Schreier::get_group_order(&stab_chain));
    schreier.run(&stab_chain);
    schreier.norb(&stab_chain);

     */
}


#[test]
fn perm_group_test() {
    // permute edge
    let permutation = Permutation::new(vec![1,3,0,2,4]);
    let set = slice_into_BM64(vec![2,3].as_slice());
    let permuted_set = PermuteSubset::apply_to(&permutation, &set);

    assert_eq!(slice_into_BM64(vec![0,2].as_slice()), permuted_set);


    // getting indexes and targets
    let action = PermuteSubset::new(5,3);
    let range = action.get_range();



    (0..range).for_each(|i|{
        let set = action.get_target_from_index(i);
        println!("index {i} goes to target: {:?}", BM64_as_set(&set));
    });


    let set = action.get_target_from_index(0);
    assert_eq!(slice_into_BM64(vec![2,1,0].as_slice()), set);

    let index = 6;
    let set = action.get_target_from_index(index);
    assert_eq!(slice_into_BM64(vec![4,2,1].as_slice()), set);

    let new_index = action.get_index_from_target(&set);
    assert_eq!(new_index, index);

    println!("index: {:?}", index);
    println!("set: {:?}", BM64_as_set(&set));
    println!("range: {:?}", range);
}