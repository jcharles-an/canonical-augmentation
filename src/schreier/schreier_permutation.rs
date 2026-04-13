use std::cmp;
use std::collections::HashSet;
use std::path::Prefix;
use std::str::from_utf8_unchecked;
use bitmaps::Bitmap;
use petgraph::visit::Walker;
use crate::data_structs::bitmaps::{BM64_as_set, BM64};
use crate::data_structs::stabilizer_chain::StabChain;
use crate::groups::group::Group;
use crate::group_actions::groupAction::GroupAction;
use crate::data_structs::permutation::Permutation;
use crate::groups::permutation_group::PermutationGroup;
use crate::group_actions::permute_points::PermutePoints;
use crate::schreier::stabilizer_chain::RecStabilizerChain;

struct SchreierVector<T,U>{
    list: Vec<Option<T>>,
    origin: U,
}

pub struct Data{
    pub(crate) orbit: Vec<usize>,
    pub(crate) stab_generators: Vec<Permutation>,
    transversal: Vec<Option<Permutation>>,
}

#[derive(Clone)]
pub(crate) struct Schreier{
    pub group: PermutationGroup,
    pub action: PermutePoints,
}

impl Schreier{

    pub fn new(group: PermutationGroup) -> Schreier{
        let n = group.get_identity_length();
        Schreier{
            group,
            action: PermutePoints::new(n),
        }
    }
    
    pub fn from_generators(generators: Vec<Permutation>) -> Schreier {
        let group = PermutationGroup::new(generators);
        let n = group.get_identity_length();
        Schreier{
            group,
            action: PermutePoints::new(n),
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
                let next_orbit_element = PermutePoints::apply_to(generator, &previous_orbit_element);
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
            orbit_element_placeholder = PermutePoints::apply_to(
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
            let data = schreier.get_orbit(l, schreier.action.get_range());
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
            let data = schreier.get_orbit(PermutePoints::apply_to(&g,&M[i]), n);
            let g1 = data.transversal[L[i]].as_ref()?;
            g = PermutationGroup::operate(&g,g1);
            schreier = Schreier::from_generators(data.stab_generators)
        }
        Some(g)
    }

    /// Calculates all possible k-tuples in S, non-isomorph in G
    ///
    /// # Arguments
    /// * `schreier` - schreier-sims struct to interact with group generated by generators
    /// * `n` - the max possible value in the set.
    /// * `S` - slice, subset of ints from which to choose
    /// * `k` - int, number of elements to choose
    ///
    /// # Returns
    /// Vec holding all possible k-tuples in S, non-isomorph in G
    pub fn get_S_choose_k_in_G(mut schreier: Schreier,n:usize, S: &[usize], k: usize) -> Vec<Vec<usize>> {
        let mut vec_holder = vec![];
        let mut k_tuples = vec![];

        // express slice of ints as a bitmap
        let mut full_set = 0;
        for s in S {
            full_set |= 1usize << s
        }
        Schreier::get_S_choose_k_in_G_rec(schreier, n, full_set,  k,0, &mut vec_holder,0, &mut k_tuples);
        k_tuples
    }

    /// Calculates the sum of two 32-bit signed integers.
    ///
    /// # Arguments
    /// * `schreier` - schreier-sims struct to interact with group generated by generators
    /// * `n` - the max possible value in the set.
    /// * `S` - int-bitmap, subset of ints from which to choose
    /// * `current_index` - int, index from which to choose up to n
    /// * `prefix` - current partial solution prefix holder
    /// * `k` - int, number of elements to choose
    /// * `k_tuples` - Vec holding all possible k-tuples in S, non-isomorph in G
    pub fn get_S_choose_k_in_G_rec(mut schreier: Schreier, n:usize, S: usize, k: usize, mut current_index: usize, mut current_prefix: &mut Vec<usize>, last_orbit: usize,  k_tuples: &mut Vec<Vec<usize>> ){
        if k == 0 {
            k_tuples.push(current_prefix.clone());
            return;
        }
        
        let mut local_available_S = S;

        for i in current_index..n {
            // if element i is not on local_available_S, continue
            if (local_available_S >> i)&1 == 0  {
                continue;
            }
            println!("available: {:8b}", local_available_S);


            // get an orbit representative for ith element in G
            let mut data = schreier.get_orbit(i as usize, n);
            let orbit_rep = data.orbit.first().unwrap();
            current_prefix.push(*orbit_rep);
            // when chosing outside of the previous orbit, no more elements in that orbit are allowed


            println!("orbit: {:?}", data.orbit);
            let mut orbit_bitmap = 0;
            for orb in data.orbit.iter() {
                orbit_bitmap |= 1usize << orb;
            }
            println!("prefix: {:?}\n", current_prefix);
            Self::get_S_choose_k_in_G_rec(Schreier::from_generators(data.stab_generators), n, local_available_S, k - 1,i + 1, current_prefix, orbit_bitmap, k_tuples);
            current_prefix.pop();


            // make the elements inside the orbit unavailable
            for orb in data.orbit{
                local_available_S &= !orbit_bitmap
            }

            // [[(10, 0), (10, 5), (10, 6)], [(10, 2), (10, 3), (10, 6)], [(10, 2), (10, 4), (10, 5)], [(10, 2), (10, 4), (10, 6)]]
        }
    }



    pub fn get_S_choose_k_in_G_2(mut schreier: Schreier,n:usize, S: &[usize], k: usize) -> Vec<Vec<usize>> {
        let mut s_set = BM64::new();
        for s in S{
            s_set.set(*s,true);
        }

        let mut orbits = vec![];
        for s in S{
            if !s_set.get(*s) {
                continue;
            }
            let data = schreier.get_orbit(*s,n);
            let mut orbit_set = BM64::new();
            println!("{:?}", data.orbit);
            for orb in data.orbit{
                orbit_set.set(orb,true);
                s_set.set(orb,false);
            }
            orbits.push(orbit_set);
        }

        let mut k_tuples = vec![];
        Schreier::get_S_choose_k_in_G_rec_2(orbits, BM64::new(), s_set, k, &mut k_tuples);
        k_tuples.iter().map(|k_tuple| k_tuple.into_iter().collect() ).collect()
    }

    pub fn get_S_choose_k_in_G_rec_2(mut orbits: Vec<BM64>, mut prefix: BM64, S: BM64, k: usize, k_tuples: &mut Vec<BM64>){

        if prefix.len() == k {
            k_tuples.push(prefix);
            //println!("found tuple {prefix:?}");
            return;
        }

        let first_orbit = orbits.get(0);
        if first_orbit.is_none() {
            //println!("No more orbits, return");
            return;
        }

        //discard orbit
        Self::get_S_choose_k_in_G_rec_2(orbits[1..orbits.len()].to_owned(), prefix, S, k, k_tuples);

        let current_orbit = orbits.get_mut(0).unwrap();
        let element = current_orbit.first_index();
        if element.is_none() {
            return;
        }
        let element = element.unwrap();

        current_orbit.set(element, false);
        prefix.set(element, true);

        // Keep orbit
        // Self::get_S_choose_k_in_G_rec_2(orbits.clone(), prefix, k, k_tuples);

    }


    pub fn get_S_choose_k(n:usize, S: &[usize], k: usize) -> Vec<BM64> {
        let mut vec_holder = BM64::new();
        let mut prefixes_holder = vec![];
        let mut full_set = 0;
        for s in S {
            full_set |= 1usize << s
        }
        Schreier::get_S_choose_k_rec(n, full_set, k, 0, &mut vec_holder, &mut prefixes_holder);
        prefixes_holder
    }
    pub fn get_S_choose_k_rec(n: usize, S: usize, k: usize, mut current_index: usize, mut current_prefix: &mut BM64, k_tuples: &mut Vec<BM64>) {
        if k == 0 {
            k_tuples.push(current_prefix.clone());
            return;
        }

        for i in current_index..n {
            if (S >> i)&1 == 1 {
                current_prefix.set(i, true);
                Self::get_S_choose_k_rec(n, S, k - 1, i + 1, current_prefix, k_tuples);
                current_prefix.set(i, false);
            }
        }
    }
}

struct SChooseKInGState{

}
pub struct SChooseKInG {
    n:usize,
    S: usize,
    k: usize,
    current_prefix: Vec<usize>,
    available_S_per_level: Vec<usize>,
    schreier_per_level: Vec<Option<Schreier>>,
    index_per_level: Vec<usize>,
}

impl SChooseKInG {
    pub fn new(schreier: Schreier, n: usize, S: &[usize], k: usize) -> SChooseKInG {
        // express slice of ints as a bitmap
        let mut S_bitmap = 0;
        for s in S {
            S_bitmap |= 1usize << s
        }
        let mut schreier_per_index = vec![None;k];
        schreier_per_index[0] = Some(schreier);
        SChooseKInG{
            n,
            S: S_bitmap,
            k,
            current_prefix: Vec::with_capacity(k),
            available_S_per_level: vec![S_bitmap; k],
            schreier_per_level: schreier_per_index,
            index_per_level: (0..k).collect(),
        }
    }
}

impl Iterator for SChooseKInG {
    type Item = Vec<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            //this while traverses the level
            while self.index_per_level[self.current_prefix.len()] < self.n {
                let level = self.current_prefix.len();
                let index = self.index_per_level[level];

                // if element i is not on
                if (self.available_S_per_level[level] >> index) & 1 == 0 {
                    // advance level iteration index
                    self.index_per_level[level] += 1;
                    continue;
                }

                // get an orbit representative for ith element in G
                let mut data = self.schreier_per_level[level].as_ref().unwrap()
                    .get_orbit(index, self.n);

                //Add the orbit representative to current_prefix (it is always current index)
                self.current_prefix.push(index); // current prefix len has changed here

                if level + 1 == self.k {
                    let k_tuple = self.current_prefix.clone();

                    for orbit_element in &data.orbit {
                        self.available_S_per_level[level]  &= !(1 << *orbit_element)
                    }

                    self.current_prefix.pop();

                    return Some(k_tuple);
                }

                //simulate recursive call
                self.schreier_per_level[level + 1] = Some(Schreier::from_generators(data.stab_generators));
                self.available_S_per_level[level + 1] = self.available_S_per_level[level];
                self.index_per_level[level + 1] = self.current_prefix[level] + 1;


                // make the elements inside the orbit unavailable
                for orbit_element in &data.orbit {
                    self.available_S_per_level[level]  &= !(1 << *orbit_element)
                }
            }

            self.current_prefix.pop();

            // if tried all possibilities in first level
            if self.index_per_level[0] == self.n && self.current_prefix.is_empty() {
                return None;
            }
        }
    }
}


#[test]
fn orbit_test_1() {


    let mut schreier = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![0,2,4]]),
                Permutation::from_cycles(vec![vec![0,3,5]]),

            ])
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
            vec![Permutation::new(vec![0,1,2,3,4,5])])
    );


    schreier.group.simple_generate_group();
    
    let range = schreier.action.get_range();
    let S = (0..range).collect::<Vec<usize>>();
    let k = 3;
    let mut k_tuples_in_G = Schreier::get_S_choose_k_in_G(schreier.clone(), range, &S, k);
    println!("Subsets: {:?}", k_tuples_in_G);
    let mut k_tuples_in_G = Schreier::get_S_choose_k_in_G_2(schreier.clone(), range, &S, k);
    println!("Subsets: {:?}", k_tuples_in_G);

    let a: Vec<_> = SChooseKInG::new(schreier.clone(), range, &S, k).collect();
    println!("Subsets: {:?}", a);


    let mut full_k_tuples = Schreier::get_S_choose_k(range, &S,k);
    
    for full_k_tuple in &full_k_tuples {
        let full_k_tuple = full_k_tuple.into_iter().collect::<Vec<usize>>();
        println!("{:?} has transporter: ", full_k_tuple);
        for k_tuple_in_G in &k_tuples_in_G {
            let transporter = Schreier::transporter(schreier.clone(), full_k_tuple.as_slice(), k_tuple_in_G);
            if let Some(trans) = transporter {
                println!("{:?} = {:?} ^ {:?} = {:?}", k_tuple_in_G, full_k_tuple.as_slice(), trans, PermutePoints::apply_to_vec(&trans, full_k_tuple.as_slice(),));
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
    
    let range = schreier.action.get_range();
    let S = vec![1,4,5,7];
    let k = 3;
    let mut prefixes_holder = Schreier::get_S_choose_k_in_G(schreier.clone(),7, &S,k,);
    println!("Subsets: {:?}\n", prefixes_holder);

    let a: Vec<_> = SChooseKInG::new(schreier.clone(), range, &S, k).collect();
    println!("Subsets: {:?}\n", a);

    let mut full_k_tuples = Schreier::get_S_choose_k(range, &S,k);
    
    for full_prefix in &full_k_tuples {
        let full_prefix = full_prefix.into_iter().collect::<Vec<usize>>();
        println!("{:?} has transporter: ", full_prefix);
        for prefix in &prefixes_holder {
            let transporter = Schreier::transporter(schreier.clone(), full_prefix.as_slice(), prefix);
            if let Some(trans) = transporter {
                println!("{:?} = {:?} ^ {:?} = {:?}", prefix, full_prefix ,trans, PermutePoints::apply_to_vec( &trans, full_prefix.as_slice(),));
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
    
    let range = schreier.action.get_range();
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
    let n = schreier.action.get_range();
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

    let n = schreier.action.get_range();
    let data = schreier.get_orbit(0, n);
    assert_eq!(data.orbit, vec![0]);
    
    let data = schreier.get_orbit(1, n);
    assert_eq!(data.orbit, vec![1, 2, 3]);
    
    let data = schreier.get_orbit(2, n);
    assert_eq!(data.orbit, vec![2, 3, 1]);
    
    let data = schreier.get_orbit(3, n);
    assert_eq!(data.orbit, vec![3, 1, 2]);
    





}

#[test]
fn test_S_choose_k() {

    let mut schreier = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![1,4],vec![5,7]]),
                Permutation::from_cycles(vec![vec![1,4,5,7]]),

            ])
    );

    let range = schreier.action.get_range();
    let S = vec![1,2,4,5,7];
    let k = 3;
    let mut full_k_tuples = Schreier::get_S_choose_k(range, &S,k);

   println!("S_choose_{k} with range {range} over S:{S:?}:");
    for k_tuple in full_k_tuples {
        println!("{:?}", BM64_as_set(&k_tuple));
    }

}
