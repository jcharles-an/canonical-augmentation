use std::cmp;
use crate::schreier::group_element::GroupElement;
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;

pub struct PermutationGroup<T>
{
    generators: Vec<T>,
    identity: Permutation,
    max_n: usize
}

impl Group for PermutationGroup<Permutation>{
    type Element = Permutation;
    type Target = usize;

    fn operate(element_a: &Self::Element, element_b: &Self::Element) -> Self::Element {
        let mut result;
        if element_a.size > element_b.size{
            result = element_a.map.clone()
        } else {
            result = element_b.map.clone()
        }
        for (index, value) in element_b.map.iter().enumerate(){
            result[index] = match Self::apply_to(element_a, value){
                Some(transformed) => transformed,
                None => *value
            }
        }
        Permutation::new(result)
    }

    fn get_inverse(element: &Self::Element) -> Self::Element {
        let mut result = vec![0;element.size];
        for (index, value) in element.map.iter().enumerate(){
            result[*value - 1] = index + 1
        }
        Permutation::new(result)
    }

    fn apply_to(element: &Self::Element, target: &Self::Target) -> Option<Self::Target> {
        element.map.get(target - 1 ).map(|&x| x)
    }

    fn get_identity(&self) -> Self::Element {
        self.identity.clone()
    }

    fn get_identity_ref(&self) -> &Self::Element {
        &self.identity
    }

    fn get_generators(&self) -> &Vec<Self::Element> {
        &self.generators
    }
}
impl PermutationGroup<Permutation> {
    pub fn new(generators: Vec<Permutation>) -> PermutationGroup<Permutation>{
        let max_n = generators.iter().map(|generator| generator.size).max().unwrap_or(0);
        PermutationGroup {
            generators,
            max_n,
            identity: Permutation::new((1..max_n+1).collect())
        }
    }
}

#[test]
fn perm_group_test() {
    let mut permutation_group = PermutationGroup::new(
        vec![Permutation::new(vec![1,3,4,6,5,2])]
    );

    let mut permutation = Permutation::new(vec![1,3,4,6,5,2]);
    assert_eq!(PermutationGroup::get_inverse(&permutation).map, vec![1,6,2,3,5,4]);
    assert_eq!(PermutationGroup::apply_to(&permutation, &1),Some(1));
    assert_eq!(PermutationGroup::apply_to(&permutation,&2),Some(3));
    assert_eq!(PermutationGroup::apply_to(&permutation,&3),Some(4));

    permutation = Permutation::new(vec![3,1,5,4,2]);
    let permutation2 = Permutation::new(vec![6,2,4,1,5,3]);
    assert_eq!(PermutationGroup::operate(&permutation, &permutation2).map, vec![6,1,4,3,2,5]);

    let permutation2 = Permutation::new(vec![1,3,2]);
    assert_eq!(PermutationGroup::operate(&permutation, &permutation2).map, vec![3,5,1,4,2]);
    assert_eq!(PermutationGroup::operate(&permutation2, &permutation).map, vec![2,1,5,4,3]);


    assert_eq!(permutation_group.get_identity().map, vec![1, 2, 3, 4, 5, 6]);


}