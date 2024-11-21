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
        let mut traversal = HashMap::new();
        traversal.insert(origin,self.group.get_identity());
        let mut schreier_vector = vec![self.group.get_identity_ref()];
        let mut stab_generators = vec![self.group.get_identity()];

        let mut orbit_index = 0;
        orbit_map.insert(origin, orbit_index);

         while orbit_index < orbit_vec.len(){
            for generator in self.group.get_generators(){
                if let Some(transformed) = T::apply_to(generator,&orbit_vec[orbit_index]){
                    if let None = orbit_map.insert(transformed,orbit_index) {
                        orbit_vec.push(transformed);
                        traversal.insert(transformed, T::operate(generator,&traversal[&orbit_vec[orbit_index]]));
                        schreier_vector.push(generator);
                    } else {
                        let transformed_traversal = traversal.get(&transformed).unwrap();
                        //TODO. This is not the most efficient way to do this
                        let next_generator = T::operate(&T::get_inverse(&transformed_traversal), &T::operate(generator,&traversal[&orbit_vec[orbit_index]]));
                        stab_generators.push(next_generator)
                    }
                }
            }
            orbit_index += 1
        }
        for trav in traversal.iter(){
            //println!("{:?}",trav);
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
    let mut orbit = schreir.get_orbit(4);
    assert_eq!(orbit, vec![4, 6, 2, 3]);

    schreir = Schreier::new(
        PermutationGroup::new(
        vec![
            Permutation::new(vec![1,3,4,6,5,2]),
            Permutation::new(vec![1,2,3,4,5,9,7,8,10,6]),
            Permutation::new(vec![10,2,3,4,5,6,7,8,1,9]),
        ]));


    assert_eq!(schreir.get_orbit(1), vec![1, 10, 6, 9, 2, 3, 4]);
    assert_eq!(schreir.get_orbit(2), vec![2, 3, 4, 6, 9, 10, 1]);
    assert_eq!(schreir.get_orbit(3), vec![3, 4, 6, 2, 9, 10, 1]);
    assert_eq!(schreir.get_orbit(4), vec![4, 6, 2, 9, 3, 10, 1]);
    assert_eq!(schreir.get_orbit(5), vec![5]);
    assert_eq!(schreir.get_orbit(6), vec![6, 2, 9, 3, 10, 1, 4]);
    assert_eq!(schreir.get_orbit(7), vec![7]);
    assert_eq!(schreir.get_orbit(8), vec![8]);
    assert_eq!(schreir.get_orbit(9), vec![9, 10, 1, 6, 2, 3, 4]);
    assert_eq!(schreir.get_orbit(10), vec![10, 6, 9, 2, 1, 3, 4]);

    schreir = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::new(vec![1,2,3]),
                Permutation::new(vec![1,3,4,2]),
            ]));

    assert_eq!(schreir.get_orbit(1), vec![1]);
    assert_eq!(schreir.get_orbit(2), vec![2, 3, 4]);
    assert_eq!(schreir.get_orbit(3), vec![3, 4, 2]);
    assert_eq!(schreir.get_orbit(4), vec![4, 2, 3]);

}

