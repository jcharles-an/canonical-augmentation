use std::cmp;
use std::collections::HashSet;
use std::str::from_utf8_unchecked;
use crate::data_structs::stabilizer_chain::StabChain;
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;
use crate::schreier::permutation_group::PermutationGroup;
use crate::schreier::stabilizer_chain::RecStabilizerChain;

struct SchreierVector<T,U>{
    list: Vec<Option<T>>,
    origin: U,
}

pub struct Data{
    orbit: Vec<usize>,
    pub(crate) stab_generators: Vec<Permutation>,
    transversal: Vec<Option<Permutation>>,
}

#[derive(Clone)]
pub(crate) struct Schreier{
    pub group: PermutationGroup,
}

impl Schreier{

    pub fn new(group: PermutationGroup) -> Schreier{
        Schreier{
            group
        }
    }
    
    pub fn from_generators(generators: Vec<Permutation>) -> Schreier {
        Schreier{
            group: PermutationGroup::new(generators)
        }
    }

    pub fn get_orbit(&self, origin: usize, n: usize) -> Data {
        //let mut orbit_map = HashMap::new();
        let mut orbit_vec = vec![origin];
        let mut orbit = vec![None::<usize>;n];
        orbit[origin] = Some(origin);

        let mut traversal =  vec![None;n];
        traversal[origin] = Some(self.group.get_identity());
        
        let mut schreier_vector = SchreierVector{
            list: vec![None::<&Permutation>;n],
            origin,
        };
        
        schreier_vector.list[origin] = Some(self.group.get_identity_ref());
        let mut stab_generators = vec![self.group.get_identity()];

        let mut orbit_index = 0;
        while orbit_index < orbit_vec.len(){
            for generator in self.group.get_generators(){
                let previous_orbit_element = orbit_vec[orbit_index];
                let next_orbit_element = PermutationGroup::apply_to(generator, &previous_orbit_element);
                let traversal_representative = traversal[previous_orbit_element].as_ref().unwrap();

                if orbit[next_orbit_element].is_none() {
                    orbit_vec.push(next_orbit_element);
                    orbit[next_orbit_element] = Some(previous_orbit_element);
                    traversal[next_orbit_element] =  Some(PermutationGroup::operate(generator, traversal_representative));
                    schreier_vector.list[next_orbit_element] =  Some(generator);
                } else {

                    let transformed_traversal =  traversal[next_orbit_element].as_ref().unwrap();
                    //TODO. This is not the most efficient way to do this because is better to just save inverses in the traversal
                    let next_generator = PermutationGroup::operate(
                        &PermutationGroup::get_inverse(transformed_traversal), 
                        &PermutationGroup::operate(generator, traversal_representative));
                    if !stab_generators.contains(&next_generator){
                        stab_generators.push(next_generator)
                    }
                }
            }
            orbit_index += 1
        }
        
        if(false) {
            println!("%%%%%%%%%%%%%%%%%% Orbit for {origin} %%%%%%%%%%%%%%%%%%");
            println!("orbit");
            for (index, trav) in orbit.iter().enumerate() {
                if let Some(trav) = trav {
                    println!("{index} <- {:?}", trav);
                };
            }
            println!(" ");

            println!("SchreierVector");
            for (index, trav) in schreier_vector.list.iter().enumerate() {
                if let Some(trav) = trav {
                    //println!("{index}: {:?}",trav);
                    println!("{index}: {:?}", Self::get_representative(&schreier_vector, index));
                }
            }
            println!(" ");

            println!("traversal");
            for (index, trav) in traversal.iter().enumerate() {
                println!("{index}: {:?}", trav);
            }

            println!("stab_generators");
            for (index, trav) in stab_generators.iter().enumerate() {
                println!("{index}: {:?}",trav);
            }

            println!("end\n\n");
        }

        Data{
            orbit: orbit_vec,
            stab_generators,
            transversal: traversal
        }
    }

    fn get_representative(schreier_vector: &SchreierVector<&Permutation,usize>, orbit_element: usize) -> Permutation {
        let mut orbit_element_placeholder = orbit_element;
        let mut r = schreier_vector.list[schreier_vector.origin].unwrap().clone();
        while orbit_element_placeholder != schreier_vector.origin{
            let current_sc = schreier_vector.list[orbit_element_placeholder].unwrap();
            r = PermutationGroup::operate(current_sc,&r);
            orbit_element_placeholder = PermutationGroup::apply_to(
                &PermutationGroup::get_inverse(current_sc),
                &orbit_element_placeholder);
        }
        r
    }
    
    
    fn get_cover_rec(schreier: &Schreier, M:  Vec<usize>, L:  Vec<usize>, current_index: usize, free_M: usize){
        let l = L[current_index];
        let mut local_available_M = free_M;
        let mut m_index = 0;
        while local_available_M&1 == 1 && local_available_M != 0 {
            let data = schreier.get_orbit(l, schreier.group.get_range());
            let transporter =  data.transversal[m_index].as_ref();
            if transporter.is_none() {
                continue;
            }
            
            //transversal exists
            
            
            
            local_available_M = local_available_M >> 1
        }
    }

    pub fn transporter(mut schreier: Schreier, M: &[usize], L: &[usize]) -> Option<Permutation>{
        let n = L.len();
        let mut g = schreier.group.get_identity();
        for i in 0..n{
            let data = schreier.get_orbit(PermutationGroup::apply_to(&g,&M[i]), n);
            let g1 = data.transversal[L[i]].as_ref()?;
            g = PermutationGroup::operate(&g,g1);
            schreier = Schreier::from_generators(data.stab_generators)
        }
        Some(g)
    }
    
    pub fn get_S_choose_k_in_G(mut schreier: Schreier,n:usize, S: &[usize], k: usize) -> Vec<Vec<usize>> {
        let mut vec_holder = vec![];
        let mut prefixes_holder = vec![];
        let mut full_set = 0;
        for s in S {
            full_set |= 1usize << s
        }
        Schreier::get_S_choose_k_in_G_rec(schreier, n, full_set, 0, &mut vec_holder, k, &mut prefixes_holder);
        prefixes_holder
    }
    pub fn get_S_choose_k_in_G_rec(mut schreier: Schreier,n:usize, S: usize, mut current_index: usize, mut prefix: &mut Vec<usize>, k: usize, prefixes : &mut Vec<Vec<usize>> ){
        if k == 0 {
            prefixes.push(prefix.clone());
            return;
        }
        
        let mut local_available_M = S;

        for i in current_index..n {
            if (local_available_M >> i)&1 == 1  {
                let mut data = schreier.get_orbit(i, n);
                // println!("Orbit: {:?}", data.orbit);
                let orbit_rep = data.orbit.first().unwrap();
                prefix.push(*orbit_rep);
                //println!("orbit_rep: {}", orbit_rep);
                let mut mask = S;
                for orb in data.orbit{
                    mask ^= 1 << orb
                }
                local_available_M &= mask;
                Self::get_S_choose_k_in_G_rec(Schreier::from_generators(data.stab_generators), n, S, i + 1, prefix, k - 1, prefixes);
                prefix.pop();
            }
        }
    }


    pub fn get_S_choose_k(n:usize, S: &[usize], k: usize) -> Vec<Vec<usize>> {
        let mut vec_holder = vec![];
        let mut prefixes_holder = vec![];
        let mut full_set = 0;
        for s in S {
            full_set |= 1usize << s
        }
        Schreier::get_S_choose_k_rec(n, full_set, 0, &mut vec_holder, k, &mut prefixes_holder);
        prefixes_holder
    }
    pub fn get_S_choose_k_rec(n: usize, S: usize, mut current_index: usize, mut prefix: &mut Vec<usize>, k: usize, prefixes : &mut Vec<Vec<usize>>) {
        if k == 0 {
            prefixes.push(prefix.clone());
            return;
        }

        let mut local_available_M = S;
        
        for i in current_index..n {
            if (local_available_M >> i)&1 == 1 {
                prefix.push(i);
                Self::get_S_choose_k_rec(n, S,i + 1, prefix, k - 1, prefixes);
                prefix.pop();
            }
        }
    }


}



#[test]
fn orbit_test_1() {
  
    let mut schreier = Schreier::new(
        PermutationGroup::new(
            vec![Permutation::new(vec![0,1,2,3,4,5])])
    );

    let mut schreier = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![1,2]]),
                Permutation::from_cycles(vec![vec![0,1,2,3,4,5,6]]),
            ])
    );
    let mut schreier = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![0,2,4]]),
                Permutation::from_cycles(vec![vec![0,3,5]]),

            ])
    );
    
    schreier.group.simple_generate_group();
    
    let range = schreier.group.get_range();
    let S = (0..range).collect::<Vec<usize>>();
    let k = 3;
    let mut prefixes_holder = Schreier::get_S_choose_k_in_G(schreier.clone(), range, &S, k);
    println!("Subsets: {:?}", prefixes_holder);
    
    
    let mut full_prefixes_holder = Schreier::get_S_choose_k(range, &S,k);
    
    for full_prefix in &full_prefixes_holder {
        println!("{:?} has transporter: ", full_prefix);
        for prefix in &prefixes_holder {
            let transporter = Schreier::transporter(schreier.clone(), full_prefix, prefix);
            if let Some(trans) = transporter {
                println!("{:?} = {:?} ^ {:?} = {:?}", prefix, full_prefix ,trans, PermutationGroup::apply_to_vec( &trans, full_prefix,));
            }
        }
        println!();
    }
  
}

#[test]
fn orbit_test_3() {
    
    let mut schreier = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![1,4],vec![5,7]]),
                Permutation::from_cycles(vec![vec![1,4,5,7]]),

            ])
    );
    
    schreier.group.simple_generate_group();
    
    let range = schreier.group.get_range();
    let S = vec![1,4,5,7];
    let k = 3;
    let mut prefixes_holder = Schreier::get_S_choose_k_in_G(schreier.clone(),7, &S,k,);
    println!("Subsets: {:?}\n", prefixes_holder);
    
    let mut full_prefixes_holder = Schreier::get_S_choose_k(range, &S,k);
    
    for full_prefix in &full_prefixes_holder {
        println!("{:?} has transporter: ", full_prefix);
        for prefix in &prefixes_holder {
            let transporter = Schreier::transporter(schreier.clone(), full_prefix, prefix);
            if let Some(trans) = transporter {
                println!("{:?} = {:?} ^ {:?} = {:?}", prefix, full_prefix ,trans, PermutationGroup::apply_to_vec( &trans, full_prefix,));
            }
        }
        println!();
    }
  
}
#[test]
fn orbit_test_2() {
    
    let mut schreier = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![0,2,4]]),
                Permutation::from_cycles(vec![vec![0,3,5]]),

            ])
    );
    
    // [0, 3, 5] has transporter: to [0, 2, 3]
    // [[2, 3, 4]] * [[3, 5, 4]] = [[2, 5, 4]] inv [[2, 4, 5]] it goes to [0,5,3]
    // [[2, 5, 4]] * [[2, 4], [3, 5]] = [[2, 3, 5]]
    schreier.group.simple_generate_group();
    
    let range = schreier.group.get_range();
    let k = 3;
    let origin = 0;
    let data = schreier.get_orbit(origin, range);
    println!("{origin} has orbit: {:?}", data.orbit);
    
    let mut schreier = Schreier::from_generators(data.stab_generators);
    schreier.group.simple_generate_group();
    let origin = 2;
    let data = schreier.get_orbit(origin, range);
    println!("{origin} has orbit: {:?}", data.orbit);

    let mut schreier = Schreier::from_generators(data.stab_generators);
    schreier.group.simple_generate_group();
    let origin = 3;
    let data = schreier.get_orbit(origin, range);
    println!("{origin} has orbit: {:?}", data.orbit);
  
}

#[test]
fn orbit_test() {

    let mut schreier = Schreier::new(
        PermutationGroup::new(
            vec![Permutation::new(vec![0,2,3,5,4,1])])
    );
    let n = schreier.group.get_range();
    let mut data = schreier.get_orbit(3, n);
    assert_eq!(data.orbit, vec![3, 5, 1, 2]);

    schreier = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::new(vec![0,2,3,5,4,1]),
                Permutation::new(vec![0,1,2,3,4,8,6,7,9,5]),
                Permutation::new(vec![9,1,2,3,4,5,6,7,0,8]),
            ]));
    
    let data = schreier.get_orbit(0, n);
    assert_eq!(data.orbit, vec![0, 9, 5, 8, 1, 2, 3]);
    
    let data = schreier.get_orbit(1, n);
    assert_eq!(data.orbit, vec![1, 2, 3, 5, 8, 9, 0]);
    
    let data = schreier.get_orbit(2, n);
    assert_eq!(data.orbit, vec![2, 3, 5, 1, 8, 9, 0]);
    
    let data = schreier.get_orbit(3, n);
    assert_eq!(data.orbit, vec![3, 5, 1, 8, 2, 9, 0]);
    
    let data = schreier.get_orbit(4, n);
    assert_eq!(data.orbit, vec![4]);
    
    let data = schreier.get_orbit(5, n);
    assert_eq!(data.orbit, vec![5, 1, 8, 2, 9, 0, 3]);
    
    let data = schreier.get_orbit(6, n);
    assert_eq!(data.orbit, vec![6]);
    
    let data = schreier.get_orbit(7, n);
    assert_eq!(data.orbit, vec![7]);
    
    let data = schreier.get_orbit(8, n);
    assert_eq!(data.orbit, vec![8, 9, 0, 5, 1, 2, 3]);
    
    let data = schreier.get_orbit(9, n);
    assert_eq!(data.orbit, vec![9, 5, 8, 1, 0, 2, 3]);
    

    schreier = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::new(vec![0,1,2]),
                Permutation::new(vec![0,2,3,1]),
            ]));

    let n = schreier.group.get_range();
    let data = schreier.get_orbit(0, n);
    assert_eq!(data.orbit, vec![0]);
    
    let data = schreier.get_orbit(1, n);
    assert_eq!(data.orbit, vec![1, 2, 3]);
    
    let data = schreier.get_orbit(2, n);
    assert_eq!(data.orbit, vec![2, 3, 1]);
    
    let data = schreier.get_orbit(3, n);
    assert_eq!(data.orbit, vec![3, 1, 2]);
    





}
