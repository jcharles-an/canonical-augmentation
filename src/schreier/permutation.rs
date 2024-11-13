use std::collections::HashMap;
use crate::schreier::group::Group;

#[derive(Debug)]
pub struct Permutation{
    map: HashMap<usize,usize>
}

impl Group for Permutation {
    type Element = Permutation;

    fn operate(element_a: &Permutation, element_b: &Permutation) -> Permutation {
        todo!()
    }

    fn get_inverse(&self) -> Permutation{
        Permutation{
            map: self.map.iter().map(|(k,v)| (*v,*k)).collect()
        }
    }

    fn apply_to(&self, element: usize) -> Option<usize>{
        match self.map.get(&element){
            None => {None}
            Some(&value) => {Some(value)}
        }
    }
}

impl Permutation{
    pub fn from_slice_cicle(input: &[usize]) -> Permutation{
        let mut map = HashMap::new();
        for window in input.windows(2) {
            map.insert(window[0],window[1]);
        }
        map.insert(*input.last().unwrap(),*input.first().unwrap());
        Permutation{ map }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let permutation = Permutation::from_slice_cicle(&[1,2,3,5]);
        assert_eq!(permutation.map, HashMap::from([
            (1, 2),
            (2, 3),
            (3, 5),
            (5, 1),
        ]));
        assert_eq!(permutation.get_inverse().map, HashMap::from([
            (2, 1),
            (3, 2),
            (5, 3),
            (1, 5),
        ]));
        assert_eq!(permutation.apply_to(3), Some(5));
    }
}
