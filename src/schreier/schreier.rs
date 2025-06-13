use std::cmp;
use std::collections::HashSet;
use crate::data_structs::stabilizer_chain::StabChain;
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;
use crate::schreier::permutation_group::PermutationGroup;
use crate::schreier::stabilizer_chain::RecStabilizerChain;

struct SchreierVector<T,U>{
    list: Vec<Option<T>>,
    origin: U,
}

pub(crate) struct Schreier<T>
where
    T: Group
{
    pub group: T,
}

type G = PermutationGroup;

impl Schreier<G>{

    pub fn new(generators: G) -> Schreier<G>{
        Schreier{
            group: generators,
        }
    }
    
    pub fn get_orbit(&self, origin: <G as Group>::Target) -> Vec<<G as Group>::Target> {
        //let mut orbit_map = HashMap::new();
        let mut orbit_vec = vec![origin];
        let mut orbit = vec![None::<<G as Group>::Target>;self.group.get_range()];
        orbit[origin] = Some(origin);
        
        let mut traversal =  vec![None::<<G as Group>::Element>;self.group.get_range()];
        let mut schreier_vector = SchreierVector{
            list: vec![None::<&<G as Group>::Element>;self.group.get_range()],
            origin,
        };

        traversal[origin] = Some(self.group.get_identity());
        schreier_vector.list[origin] = Some(self.group.get_identity_ref());
        let mut stab_generators = vec![self.group.get_identity()];

        let mut orbit_index = 0;
         while orbit_index < orbit_vec.len(){
            for generator in self.group.get_generators(){
                let previous_orbit_element = orbit_vec[orbit_index];
                let next_orbit_element = G::apply_to(generator, &previous_orbit_element);
                let traversal_representative = traversal[previous_orbit_element].as_ref().unwrap();

                if orbit[next_orbit_element].is_none() {
                    orbit_vec.push(next_orbit_element);
                    orbit[next_orbit_element] = Some(previous_orbit_element);
                    traversal[next_orbit_element] =  Some(G::operate(generator, traversal_representative));
                    schreier_vector.list[next_orbit_element] =  Some(generator);
                } else {
                    
                    let transformed_traversal =  traversal[next_orbit_element].as_ref().unwrap();
                    //TODO. This is not the most efficient way to do this because is better to just save inverses in the traversal
                    let next_generator = G::operate(&G::get_inverse(&transformed_traversal), &G::operate(generator, traversal_representative));
                    stab_generators.push(next_generator)
                }
            }
            orbit_index += 1
        }

        println!("orbit");
        for (index, trav) in orbit.iter().enumerate(){
            if let Some(trav) = trav{
                println!("{index}: {:?}",trav);
            };
        }
        println!(" ");


        println!("SchreierVector");
        for (index, trav) in schreier_vector.list.iter().enumerate(){
            if let Some(trav) = trav{
                //println!("{index}: {:?}",trav);
                println!("{index}: {:?}",Self::get_representative(&schreier_vector,index));
            }
        }
        println!(" ");

        println!("traversal");
        for (index, trav) in traversal.iter().enumerate(){
            if let Some(trav) = trav{
                println!("{index}: {:?}",trav);
            }
        }

        println!("end\n\n");

        orbit_vec
    }

    fn get_representative(schreier_vector: &SchreierVector<&<G as Group>::Element,<G as Group>::Target>, orbit_element: <G as Group>::Target) -> <G as Group>::Element {
        let mut orbit_element_placeholder = orbit_element;
        let mut r = schreier_vector.list[schreier_vector.origin].unwrap().clone();
        while orbit_element_placeholder != schreier_vector.origin{
            let current_sc = schreier_vector.list[orbit_element_placeholder].unwrap();
            r = G::operate(current_sc,&r);
            orbit_element_placeholder = G::apply_to(&G::get_inverse(current_sc),&orbit_element_placeholder);
        }
        r
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
