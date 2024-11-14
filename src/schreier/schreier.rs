use std::collections::HashMap;
use crate::schreier::group_element::GroupElement;
use crate::schreier::group_generators::GroupGenerators;
use crate::schreier::permutation::Permutation;
use crate::schreier::permutation_generators::PermutationGenerators;

struct Schreier<T>
where
    T: GroupGenerators
{
    generators: T
}



impl<T:GroupGenerators> Schreier<T>{

    pub fn new(generators: T) -> Schreier<T>{
        Schreier{generators}
    }
    pub fn get_orbit(&self, origin: T::Element::Target) -> Vec<T::Element::Target> {
        let mut orbit_map = HashMap::new();
        let mut orbit_vec = vec![origin];
        let mut traversal = vec![1];
        orbit_map.insert(origin, true);

        let mut orbit_index = 0;
        while orbit_index < orbit_vec.len(){
            for generator in self.generators.iter(){
                if let Some(transformed) = generator.apply_to(&orbit_vec[orbit_index]){
                    if let None = orbit_map.insert(transformed,true) {
                        orbit_vec.push(transformed);
                    }
                }
            }
            orbit_index += 1
        }
        orbit_vec
    }
}


#[test]
fn orbit_test() {
    let mut schreir = Schreier::new(
        PermutationGenerators::new(
            vec![Permutation::new(vec![1,3,4,6,5,2])])
    );
    let mut orbit = schreir.get_orbit(4);
    assert_eq!(orbit, vec![4, 6, 2, 3]);
    orbit = schreir.get_orbit(1);
    println!("{:?}", orbit);

    schreir = Schreier::new(
        PermutationGenerators::new(
        vec![
            Permutation::new(vec![1,3,4,6,5,2]),
            Permutation::new(vec![1,2,3,4,5,9,7,8,10,6]),
            Permutation::new(vec![10,2,3,4,5,6,7,8,1,9]),
        ]));
    for i in 1..11{
        orbit = schreir.get_orbit(i);
        println!("i:{i} => {:?}", orbit);
    }

}

