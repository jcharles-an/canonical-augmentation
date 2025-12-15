use std::cmp;
use std::collections::HashSet;
use crate::data_structs::stabilizer_chain::StabChain;
use crate::groups::group::Group;
use crate::group_actions::groupAction::GroupAction;
use crate::data_structs::permutation::Permutation;
use crate::groups::permutation_group::PermutationGroup;
use crate::schreier::schreier::Schreier;

#[derive(Debug, Clone)]
pub struct PermutePoints
{
    range: usize,
}
impl GroupAction for PermutePoints{
    type Target = usize;
    type Group = PermutationGroup;
    fn apply_to(element: &Permutation, target: &usize) -> usize {
        element.map.get( *target ).map(|&x| x).unwrap_or(*target)
    }
    fn get_index_from_target(&self, target: &Self::Target) -> usize {
        *target
    }
    fn get_target_from_index(&self, index: usize) -> Self::Target{
        index
    }

    fn get_range(&self) -> usize {
        self.range
    }
}


impl PermutePoints {
    
    pub fn new(range: usize) -> PermutePoints {
        PermutePoints {
            range
        }
    }
    pub fn apply_to_vec(element: &Permutation, targets: &[<PermutePoints as GroupAction>::Target]) -> Vec<usize> {
        targets.iter().map( |target| Self::apply_to(element,target) ).collect()
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
    let permutation_group = PermutationGroup::new(
        vec![Permutation::new(vec![0,2,3,5,4,1])]
    );

    let permutation = Permutation::new(vec![1,3,0,2,4]);
    let permutation2 = Permutation::new(vec![4,3,2,1,0]);
    assert_eq!(PermutationGroup::operate(&permutation2, &permutation).map, vec![3,1,4,2,0]);



    assert_eq!(permutation_group.get_identity().map, vec![0, 1, 2, 3, 4, 5]);



}