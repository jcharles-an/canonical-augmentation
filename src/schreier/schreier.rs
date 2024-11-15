use std::collections::HashMap;
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;
use crate::schreier::permutation_group::PermutationGroup;

struct Schreier<T>
where
    T: Group
{
    group: T
}



impl<T: Group> Schreier<T>{

    pub fn new(generators: T) -> Schreier<T>{
        Schreier{ group: generators }
    }
    pub fn get_orbit(&self, origin: T::Target) -> Vec<T::Target> {
        let mut orbit_map = HashMap::new();
        let mut orbit_vec = vec![origin];
        let mut traversal = vec![self.group.get_identity()];
        orbit_map.insert(origin, true);

        let mut orbit_index = 0;

         while orbit_index < orbit_vec.len(){
            for generator in self.group.get_generators(){
                if let Some(transformed) = T::apply_to(generator,&orbit_vec[orbit_index]){
                    if let None = orbit_map.insert(transformed,true) {
                        orbit_vec.push(transformed);
                        traversal.push(T::operate(generator,&traversal[orbit_index]));
                    }
                }
            }
            orbit_index += 1
        }
        for trav in traversal.iter(){
            println!("{:?}",trav);
        }

        orbit_vec
    }
}


#[test]
fn orbit_test() {
    let mut schreir = Schreier::new(
        PermutationGroup::new(
            vec![Permutation::new(vec![1,3,4,6,5,2])])
    );
    //let mut orbit = schreir.get_orbit(4);
    //assert_eq!(orbit, vec![4, 6, 2, 3]);


    schreir = Schreier::new(
        PermutationGroup::new(
        vec![
            Permutation::new(vec![1,3,4,6,5,2]),
            Permutation::new(vec![1,2,3,4,5,9,7,8,10,6]),
            Permutation::new(vec![10,2,3,4,5,6,7,8,1,9]),
        ]));
    for i in 1..11{
        let orbit = schreir.get_orbit(i);
        println!("i:{i} => {:?}", orbit);
    }

}

