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

type G = PermutationGroup;

impl Schreier<G>{

    pub fn new(generators: G) -> Schreier<G>{
        Schreier{ group: generators }
    }
    pub fn get_orbit(&self, origin: <G as Group>::Target) -> Vec<<G as Group>::Target> {
        //let mut orbit_map = HashMap::new();
        let mut orbit_vec = vec![origin];
        let mut orbit = vec![None::<usize>;self.group.get_range()];
        let mut traversal = HashMap::new();
        traversal.insert(origin,self.group.get_identity());
        let mut schreier_vector = vec![self.group.get_identity_ref()];
        let mut stab_generators = vec![self.group.get_identity()];

        let mut orbit_index = 0;
        orbit[origin] = Some(origin);
        //orbit_map.insert(origin, orbit_index);

         while orbit_index < orbit_vec.len(){
            for generator in self.group.get_generators(){
                if let Some(transformed) = G::apply_to(generator, &orbit_vec[orbit_index]){


                    if orbit[transformed] == None {
                    //if let None = orbit_map.insert(transformed,orbit_index) {
                        orbit_vec.push(transformed);
                        orbit[transformed] = Some(orbit_vec[orbit_index]);
                        traversal.insert(transformed, G::operate(generator, &traversal[&orbit_vec[orbit_index]]));
                        schreier_vector.push(generator);
                    } else {
                        let transformed_traversal = traversal.get(&transformed).unwrap();
                        //TODO. This is not the most efficient way to do this
                        let next_generator = G::operate(&G::get_inverse(&transformed_traversal), &G::operate(generator, &traversal[&orbit_vec[orbit_index]]));
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
            vec![Permutation::new(vec![0,2,3,5,4,1])])
    );
    let mut orbit = schreir.get_orbit(3);
    assert_eq!(orbit, vec![3, 5, 1, 2]);

    schreir = Schreier::new(
        PermutationGroup::new(
        vec![
            Permutation::new(vec![0,2,3,5,4,1]),
            Permutation::new(vec![0,1,2,3,4,8,6,7,9,5]),
            Permutation::new(vec![9,1,2,3,4,5,6,7,0,8]),
        ]));


    assert_eq!(schreir.get_orbit(0), vec![0, 9, 5, 8, 1, 2, 3]);
    assert_eq!(schreir.get_orbit(1), vec![1, 2, 3, 5, 8, 9, 0]);
    assert_eq!(schreir.get_orbit(2), vec![2, 3, 5, 1, 8, 9, 0]);
    assert_eq!(schreir.get_orbit(3), vec![3, 5, 1, 8, 2, 9, 0]);
    assert_eq!(schreir.get_orbit(4), vec![4]);
    assert_eq!(schreir.get_orbit(5), vec![5, 1, 8, 2, 9, 0, 3]);
    assert_eq!(schreir.get_orbit(6), vec![6]);
    assert_eq!(schreir.get_orbit(7), vec![7]);
    assert_eq!(schreir.get_orbit(8), vec![8, 9, 0, 5, 1, 2, 3]);
    assert_eq!(schreir.get_orbit(9), vec![9, 5, 8, 1, 0, 2, 3]);

    schreir = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::new(vec![0,1,2]),
                Permutation::new(vec![0,2,3,1]),
            ]));

    assert_eq!(schreir.get_orbit(0), vec![0]);
    assert_eq!(schreir.get_orbit(1), vec![1, 2, 3]);
    assert_eq!(schreir.get_orbit(2), vec![2, 3, 1]);
    assert_eq!(schreir.get_orbit(3), vec![3, 1, 2]);

}

