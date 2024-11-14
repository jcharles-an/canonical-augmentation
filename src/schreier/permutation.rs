use std::cmp;
use crate::schreier::group_element::GroupElement;
pub struct Permutation {
    map: Vec<usize>,
    size: usize
}

impl GroupElement for Permutation {
    type Element = Permutation;
    type Target = usize;

    fn operate(&self, element: &Self::Element) -> Self::Element {
        let mut result = vec![0; cmp::max(self.size,element.size)];
        for (index, value) in element.map.iter().enumerate(){
            result[index] = match self.apply_to(value){
                Some(transformed) => transformed,
                None => *value
            }
        }
        Permutation::new(result)
    }

    fn get_inverse(&self) -> Self::Element {
        let mut result = vec![0;self.size];
        for (index, value) in self.map.iter().enumerate(){
            result[*value - 1] = index + 1
        }
        Permutation::new(result)
    }

    fn apply_to(&self, target: &Self::Target) -> Option<Self::Target> {
        self.map.get(target - 1 ).map(|&x| x)
    }
}

impl Permutation{
    pub fn new(map: Vec<usize>) -> Permutation{
        Permutation{size: map.len(), map}
    }
}

#[test]
fn perm_test() {
    let mut permutation = Permutation::new(vec![1,3,4,6,5,2]);
    assert_eq!(permutation.get_inverse().map, vec![1,6,2,3,5,4]);
    assert_eq!(permutation.apply_to(&1),Some(1));
    assert_eq!(permutation.apply_to(&2),Some(3));
    assert_eq!(permutation.apply_to(&3),Some(4));

    permutation = Permutation::new(vec![3,1,5,4,2]);
    let permutation2 = Permutation::new(vec![6,2,4,1,5,3]);

    assert_eq!(permutation.operate(&permutation2).map, vec![6,1,4,3,2,5]);


}
