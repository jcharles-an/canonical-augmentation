use std::cmp;
use std::collections::{HashMap, HashSet};
use crate::data_structs::stabilizer_chain::StabChain;
use crate::groups::group::Group;
use crate::group_actions::groupAction::GroupAction;
use crate::data_structs::permutation::Permutation;
use crate::groups::permutation_group::PermutationGroup;
use crate::schreier::schreier::Schreier;

#[derive(Debug, Clone)]
pub struct PermuteEdges
{
    range: usize,
    vertices_range: usize,
    set: Vec<(usize, usize)>,
    inverse_set: HashMap<(usize, usize), usize>,
}
impl GroupAction for PermuteEdges{
    type Target = (usize,usize);
    type Group = PermutationGroup;
    fn apply_to(element: &Permutation, target: &Self::Target) -> Self::Target {
        let target = Self::get_canon_edge(target);
        (element.map.get( target.0 ).copied().unwrap_or(target.0),
        element.map.get( target.1 ).copied().unwrap_or(target.1))
    }
    fn get_index_from_target(&self, target: &Self::Target) -> usize {
        let ordered_target = Self::get_canon_edge(target);
       *self.inverse_set.get(&ordered_target).unwrap()
    }
    fn get_target_from_index(&self, index: usize) -> Self::Target{
        *self.set.get(index).unwrap()
    }

    fn get_range(&self) -> usize {
        self.range
    }
}


impl PermuteEdges {

    pub fn from_vertices_range<'a>(vertices_range: usize) -> PermuteEdges {
        let mut set = vec![(0, 0); num_integer::binomial(vertices_range,2)];
        let mut inverse_set = HashMap::new();

        let mut index = 0;
        for i in 0..vertices_range {
            for j in i+1..vertices_range {
                set[index] = (i, j);
                inverse_set.insert((i, j), index);
                index += 1;
            }
        }


        PermuteEdges {
            range: set.len(),
            vertices_range,
            set,
            inverse_set
        }
    }


    fn get_canon_edge(target: &<Self as GroupAction>::Target) -> <Self as GroupAction>::Target{
        if (target.0 > target.1) {(target.1, target.0)} else {target.clone()}
    }
    pub fn apply_to_vec<'a>(element: &Permutation, targets: &[<PermuteEdges as GroupAction>::Target]) -> Vec<<PermuteEdges as GroupAction>::Target> {
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
    // permute edge
    let permutation = Permutation::new(vec![1,3,0,2,4]);
    let permuted_edge = PermuteEdges::apply_to(&permutation, &(3,2));
    assert_eq!((0,2), permuted_edge);
    let permuted_edge = PermuteEdges::apply_to(&permutation, &(2,3));
    assert_eq!((0,2), permuted_edge);


    // getting indexes and targets
    let action = PermuteEdges::from_vertices_range(6);
    let edge = action.get_target_from_index(0);
    assert_eq!((0,1), edge);
    println!("first target: {:?}", edge);

    let index = action.get_index_from_target(&(3,1));
    assert_eq!(index, 6);
    let edge = action.get_target_from_index(6);
    assert_eq!(edge, (1, 3));

    let range = action.get_range();
    println!("index: {:?}", index);
    println!("range: {:?}", edge);
    println!("range: {:?}", range);
}