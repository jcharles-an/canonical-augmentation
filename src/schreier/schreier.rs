use std::cmp;
use std::collections::HashSet;
use libc::sched_param;
use petgraph::data::Element;
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

pub struct Data<G, GA>
where
    G: Group,
    GA: GroupAction< Group = G >,
{
    pub(crate) orbit: Vec<<GA as GroupAction>::Target>,
    pub(crate) orbit_representative: <GA as GroupAction>::Target,
    pub(crate) stab_generators: Vec<<G as Group>::Element>,
    transversal: Vec<Option<<G as Group>::Element>>,
}

#[derive(Clone)]
pub(crate) struct Schreier<G, GA>
where
    G: Group,
    GA: GroupAction< Group = G>,
{
    pub group: G,
    pub action: GA,
}


impl<G, GA> Schreier<G, GA>
where
    G: Group,
    GA: GroupAction< Group = G>,
{

    pub fn new(group: G, action: GA) -> Schreier<G, GA>{
        Schreier{
            group,
            action,
        }
    }

    pub fn from_generators(generators: Vec<G::Element>, action: GA) -> Schreier<G, GA> {
        let group = G::new(generators);
        Schreier{
            group,
            action,
        }
    }

    pub fn get_orbit(&self, origin: <GA as GroupAction>::Target) -> Data<G, GA> {
        //let mut orbit_map = HashMap::new();
        let mut orbit_vec = vec![origin];
        let n = self.action.get_range();
        let mut orbit = vec![None::<<GA as GroupAction>::Target>; n];
        let origin_index = self.action.get_index_from_target(&origin);
        orbit[origin_index] = Some(origin);

        let mut traversal =  vec![None::<<G as Group>::Element>; n];
        traversal[origin_index] = Some(self.group.get_identity());

        let mut schreier_vector = SchreierVector{
            list: vec![None::<&<G as Group>::Element>; n],
            origin,
        };

        schreier_vector.list[origin_index] = Some(self.group.get_identity_ref());
        let mut stab_generators = vec![self.group.get_identity()];

        let mut orbit_index = 0;
        let mut orbit_canonical_representative = origin;
        while orbit_index < orbit_vec.len(){
            for generator in self.group.get_generators(){
                let previous_orbit_element = orbit_vec[orbit_index];
                let precious_orbit_element_index = self.action.get_index_from_target(&previous_orbit_element);
                let next_orbit_element = GA::apply_to(generator, &previous_orbit_element);
                let next_orbit_element_index = self.action.get_index_from_target(&next_orbit_element);

                let traversal_representative = traversal[precious_orbit_element_index].as_ref().unwrap();

                if orbit[next_orbit_element_index].is_none() {
                    orbit_vec.push(next_orbit_element);
                    if next_orbit_element < orbit_canonical_representative{
                        orbit_canonical_representative = next_orbit_element;
                    }
                    orbit[next_orbit_element_index] = Some(previous_orbit_element);
                    traversal[next_orbit_element_index] =  Some(G::operate(generator, traversal_representative));
                    schreier_vector.list[next_orbit_element_index] =  Some(generator);
                } else {

                    let transformed_traversal =  traversal[next_orbit_element_index].as_ref().unwrap();
                    //TODO. This is not the most efficient way to do this because is better to just save inverses in the traversal
                    let next_generator = G::operate(
                        &G::get_inverse(&transformed_traversal),
                        &G::operate(generator, traversal_representative));
                    if !stab_generators.contains(&next_generator){
                        stab_generators.push(next_generator)
                    }
                }
            }
            orbit_index += 1
        }

        if(false) {
            println!("%%%%%%%%%%%%%%%%%% Orbit for {origin:?} %%%%%%%%%%%%%%%%%%");
            println!("orbit");
            for (index, trav) in orbit.iter().enumerate() {
                if let Some(trav) = trav {
                    println!("{:?} <- {:?}", self.action.get_target_from_index(index) ,trav);
                };
            }
            println!(" ");

            println!("SchreierVector");
            for (index, trav) in schreier_vector.list.iter().enumerate() {
                if let Some(trav) = trav {
                    //println!("{index}: {:?}",trav);
                    println!("{index}: {:?}", self.get_representative(&schreier_vector, self.action.get_target_from_index(index)));
                }
            }
            println!(" ");

            println!("traversal");
            for (index, trav) in traversal.iter().enumerate() {
                if let Some(trav) = trav {
                    println!("{index}: {:?}", trav);
                }

                println!("stab_generators");
                for (index, trav) in stab_generators.iter().enumerate() {
                    println!("{index}: {:?}", trav);
                }

                println!("end\n\n");
            }
        }

        Data {
            orbit: orbit_vec,
            orbit_representative: orbit_canonical_representative,
            stab_generators,
            transversal: traversal
        }
    }

    fn get_representative(&self, schreier_vector: &SchreierVector<&G::Element,<GA as GroupAction>::Target>, orbit_element: <GA as GroupAction>::Target) -> <G as Group>::Element {
        let mut orbit_element_placeholder = orbit_element;
        let origin_index = self.action.get_index_from_target(&schreier_vector.origin);
        let mut r = schreier_vector.list[origin_index].unwrap().clone();
        while orbit_element_placeholder != schreier_vector.origin{
            let orbit_element_index = self.action.get_index_from_target(&orbit_element_placeholder);
            let current_sc = schreier_vector.list[orbit_element_index].unwrap();
            r = G::operate(current_sc, &r);
            orbit_element_placeholder = GA::apply_to(
                &G::get_inverse(current_sc),
                &orbit_element_placeholder);
        }
        r
    }


    fn get_cover_rec(schreier: &Schreier<G,GA>, M:  Vec<usize>, L:  Vec<GA::Target>, current_index: usize, free_M: usize){
        let l = L[current_index];
        let mut local_available_M = free_M;
        let mut m_index = 0;
        while local_available_M&1 == 1 && local_available_M != 0 {
            let data = schreier.get_orbit(l );
            let transporter =  data.transversal[m_index].as_ref();
            if transporter.is_none() {
                continue;
            }

            //transversal exists



            local_available_M = local_available_M >> 1
        }
    }

    pub fn transporter(mut schreier: Schreier<G,GA>, M: &[GA::Target], L: &[usize]) -> Option<G::Element>{
        let n = L.len();
        let mut g = schreier.group.get_identity();
        for i in 0..n{
            let data = schreier.get_orbit(GA::apply_to(&g,&M[i]));
            let g1 = data.transversal[L[i]].as_ref()?;
            g = G::operate(&g,g1);
            schreier = Schreier::from_generators(data.stab_generators,schreier.action);
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
    pub fn get_S_choose_k_in_G(mut schreier: Schreier<G,GA>,n:usize, S: &[usize], k: usize) -> Vec<Vec<usize>> {
        let mut vec_holder = vec![];
        let mut k_tuples = vec![];

        // express slice of ints as a bitmap
        let mut full_set = 0;
        for s in S {
            full_set |= 1usize << s
        }
        Schreier::get_S_choose_k_in_G_rec(schreier, n, full_set,  k,(n-1) as isize, &mut vec_holder, &mut k_tuples);
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
    pub fn get_S_choose_k_in_G_rec(mut schreier: Schreier<G,GA>, n:usize, S: usize, k: usize, mut current_index: isize, mut current_prefix: &mut Vec<usize>,  k_tuples: &mut Vec<Vec<usize>> ){
        if k == 0 {
            k_tuples.push(current_prefix.clone());
            return;
        }

        let mut local_available_S = S;

        for i in (0..current_index + 1).rev() {
            // if element i is not on local_available_S, continue
            if (local_available_S >> i)&1 == 0  {
                continue;
            }

            // get an orbit representative for ith element in G
            let mut data = schreier.get_orbit(schreier.action.get_target_from_index(i as usize));
            let orbit_rep = schreier.action.get_index_from_target( data.orbit.first().unwrap());
            current_prefix.push(orbit_rep);

            // make the elements inside the orbit unavailable
            let mut mask = S;
            for orb in data.orbit{
                mask ^= 1 << schreier.action.get_index_from_target(&orb)  // orbit union S / (orbit intersection S)
            }
            local_available_S &= mask; // local_available_S intersection S not in orbit


            Self::get_S_choose_k_in_G_rec(Schreier::from_generators(data.stab_generators, schreier.action.clone()), n, S, k - 1,i - 1, current_prefix,  k_tuples);
            current_prefix.pop();
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
    get_S_choose_k_rec(n, full_set, k, 0, &mut vec_holder, &mut prefixes_holder);
    prefixes_holder
}
pub fn get_S_choose_k_rec(n: usize, S: usize, k: usize, mut current_index: usize, mut current_prefix: &mut Vec<usize>, k_tuples: &mut Vec<Vec<usize>>) {
    if k == 0 {
        k_tuples.push(current_prefix.clone());
        return;
    }

    for i in current_index..n {
        if (S >> i)&1 == 1 {
            current_prefix.push(i);
            get_S_choose_k_rec(n, S, k - 1, i + 1, current_prefix, k_tuples);
            current_prefix.pop();
        }
    }
}

struct SChooseKInGState{

}
pub struct SChooseKInG<G,GA> 
where
    G: Group,
    GA: GroupAction <Group=G>,
{
    n:usize,
    S: usize,
    k: usize,
    current_prefix: Vec<usize>,
    available_S_per_level: Vec<usize>,
    schreier_per_level: Vec<Option<Schreier<G,GA>>>,
    index_per_level: Vec<isize>,
}

impl<G,GA> SChooseKInG<G,GA>
where
G: Group,
GA: GroupAction <Group=G>,
{
    pub fn new(schreier: Schreier<G,GA>, n: usize, S: &[usize], k: usize) -> SChooseKInG <G,GA> {
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
            index_per_level: ((n-k) as isize..n as isize).rev().collect(),
        }
    }
}

impl<G,GA> Iterator for SChooseKInG<G,GA> 
where
G: Group,
GA: GroupAction <Group=G>,
{
    type Item = Vec<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            //this while traverses the level
            while self.index_per_level[self.current_prefix.len()] > -1 {
                let level = self.current_prefix.len();
                let index = self.index_per_level[level];

                // if element i is not on
                if (self.available_S_per_level[level] >> index) & 1 == 0 {
                    // advance level iteration index
                    self.index_per_level[level] -= 1;
                    continue;
                }

                // get an orbit representative for ith element in G
                let schreier = self.schreier_per_level[level].as_ref().unwrap();
                let mut data = schreier.get_orbit(schreier.action.get_target_from_index(index as usize));


                // make the elements inside the orbit unavailable
                let mut mask = self.S;
                for orbit_element in &data.orbit {
                    mask ^= 1 << schreier.action.get_index_from_target(orbit_element)  // orbit union S / (orbit intersection S)
                }
                self.available_S_per_level[level] &= mask; // local_available_S intersection S not in orbit

                //Add the orbit representative to current_prefix (it is always current index)
                self.current_prefix.push(index as usize); // current prefix len has changed here

                if level + 1 == self.k {
                    let k_tuple = self.current_prefix.clone();
                    self.current_prefix.pop();
                    return Some(k_tuple);
                }

                self.schreier_per_level[level + 1] = Some(Schreier::from_generators(data.stab_generators, self.schreier_per_level[level].clone()?.action ));
                self.index_per_level[level + 1] = self.current_prefix[level] as isize - 1;
                self.available_S_per_level[level + 1] = self.S;
            }

            self.current_prefix.pop();

            // if tried all possibilities in first level
            if self.index_per_level[0] == -1 && self.current_prefix.is_empty() {
                return None;
            }
        }
    }
}


#[test]
fn orbit_test_1() {
    let  group = PermutationGroup::new(
            vec![Permutation::new(vec![0,1,2,3,4,5])]
    );
    let n = group.get_identity_length();

    let mut schreier = Schreier::new(
        group,
        PermutePoints::new(n),
    );

    let  group = PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![1,2]]),
                Permutation::from_cycles(vec![vec![0,1,2,3,4,5,6]]),
            ]);
    let n = group.get_identity_length();

    let mut schreier = Schreier::new(
        group,
        PermutePoints::new(n),
    );

    let  group = PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![0,2,4]]),
                Permutation::from_cycles(vec![vec![0,3,5]]),

            ]);
    let n = group.get_identity_length();

    let mut schreier = Schreier::new(
        group,
        PermutePoints::new(n),
    );

    schreier.group.simple_generate_group();

    let range = schreier.action.get_range();
    let S = (0..range).collect::<Vec<usize>>();
    let k = 3;
    let mut k_tuples_in_G = Schreier::get_S_choose_k_in_G(schreier.clone(), range, &S, k);
    println!("Subsets: {:?}", k_tuples_in_G);
    let a: Vec<_> = SChooseKInG::new(schreier.clone(), range, &S, k).collect();
    println!("Subsets: {:?}", a);


    let mut full_k_tuples = get_S_choose_k(range, &S,k);

    for full_k_tuple in &full_k_tuples {
        println!("{:?} has transporter: ", full_k_tuple);
        for k_tuple_in_G in &k_tuples_in_G {
            let transporter = Schreier::transporter(schreier.clone(), full_k_tuple, k_tuple_in_G);
            if let Some(trans) = transporter {
                println!("{:?} = {:?} ^ {:?} = {:?}", k_tuple_in_G, full_k_tuple, trans, PermutePoints::apply_to_vec(&trans, full_k_tuple,));
            }
        }
        println!();
    }

}

#[test]
fn orbit_test_3() {

    let  group = PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![1,4],vec![5,7]]),
                Permutation::from_cycles(vec![vec![1,4,5,7]]),

            ]);
    let n = group.get_identity_length();
    let mut schreier = Schreier::new(
        group,
        PermutePoints::new(n),
    );

    schreier.group.simple_generate_group();

    let range = schreier.action.get_range();
    let S = vec![1,4,5,7];
    let k = 3;
    let mut prefixes_holder = Schreier::get_S_choose_k_in_G(schreier.clone(),7, &S,k,);
    println!("Subsets: {:?}\n", prefixes_holder);

    let a: Vec<_> = SChooseKInG::new(schreier.clone(), range, &S, k).collect();
    println!("Subsets: {:?}\n", a);

    let mut full_k_tuples = get_S_choose_k(range, &S,k);

    for full_prefix in &full_k_tuples {
        println!("{:?} has transporter: ", full_prefix);
        for prefix in &prefixes_holder {
            let transporter = Schreier::transporter(schreier.clone(), full_prefix, prefix);
            if let Some(trans) = transporter {
                println!("{:?} = {:?} ^ {:?} = {:?}", prefix, full_prefix ,trans, PermutePoints::apply_to_vec( &trans, full_prefix,));
            }
        }
        println!();
    }

}
#[test]
fn orbit_test_2() {

    let  group = PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![0,2,4]]),
                Permutation::from_cycles(vec![vec![0,3,5]]),

            ]);
    let n = group.get_identity_length();

    let mut schreier = Schreier::new(
        group,
        PermutePoints::new(n),
    );

    // [0, 3, 5] has transporter: to [0, 2, 3]
    // [[2, 3, 4]] * [[3, 5, 4]] = [[2, 5, 4]] inv [[2, 4, 5]] it goes to [0,5,3]
    // [[2, 5, 4]] * [[2, 4], [3, 5]] = [[2, 3, 5]]
    schreier.group.simple_generate_group();

    let range = schreier.action.get_range();
    let k = 3;
    let origin = 0;
    let data = schreier.get_orbit(origin);
    println!("{origin} has orbit: {:?}", data.orbit);

    let mut schreier = Schreier::from_generators(
        data.stab_generators, schreier.action.clone());
    schreier.group.simple_generate_group();
    let origin = 2;
    let data = schreier.get_orbit(origin);
    println!("{origin} has orbit: {:?}", data.orbit);

    let mut schreier = Schreier::from_generators(
        data.stab_generators, schreier.action.clone());
    schreier.group.simple_generate_group();
    let origin = 3;
    let data = schreier.get_orbit(origin);
    println!("{origin} has orbit: {:?}", data.orbit);

}

#[test]
fn orbit_test() {

    let group = PermutationGroup::new(
        vec![Permutation::new(vec![0,2,3,5,4,1])]);
    let n = group.get_identity_length();

    let mut schreier = Schreier::new(
        group,
        PermutePoints::new(n),
    );
    let mut data = schreier.get_orbit(3);
    assert_eq!(data.orbit, vec![3, 5, 1, 2]);


    let group = PermutationGroup::new(
        vec![
            Permutation::new(vec![0,2,3,5,4,1]),
            Permutation::new(vec![0,1,2,3,4,8,6,7,9,5]),
            Permutation::new(vec![9,1,2,3,4,5,6,7,0,8]),
        ]);
    let n = group.get_identity_length();

    schreier = Schreier::new(
        group,
        PermutePoints::new(n)
    );


    let data = schreier.get_orbit(0);
    assert_eq!(data.orbit, vec![0, 9, 5, 8, 1, 2, 3]);

    let data = schreier.get_orbit(1);
    assert_eq!(data.orbit, vec![1, 2, 3, 5, 8, 9, 0]);

    let data = schreier.get_orbit(2);
    assert_eq!(data.orbit, vec![2, 3, 5, 1, 8, 9, 0]);

    let data = schreier.get_orbit(3);
    assert_eq!(data.orbit, vec![3, 5, 1, 8, 2, 9, 0]);

    let data = schreier.get_orbit(4);
    assert_eq!(data.orbit, vec![4]);

    let data = schreier.get_orbit(5);
    assert_eq!(data.orbit, vec![5, 1, 8, 2, 9, 0, 3]);

    let data = schreier.get_orbit(6);
    assert_eq!(data.orbit, vec![6]);

    let data = schreier.get_orbit(7);
    assert_eq!(data.orbit, vec![7]);

    let data = schreier.get_orbit(8);
    assert_eq!(data.orbit, vec![8, 9, 0, 5, 1, 2, 3]);

    let data = schreier.get_orbit(9);
    assert_eq!(data.orbit, vec![9, 5, 8, 1, 0, 2, 3]);


    let group = PermutationGroup::new(
        vec![
            Permutation::new(vec![0,1,2]),
            Permutation::new(vec![0,2,3,1]),
        ]);
    let n = group.get_identity_length();

    schreier = Schreier::new(
        group,
        PermutePoints::new(n)
    );

    let data = schreier.get_orbit(0);
    assert_eq!(data.orbit, vec![0]);

    let data = schreier.get_orbit(1);
    assert_eq!(data.orbit, vec![1, 2, 3]);

    let data = schreier.get_orbit(2);
    assert_eq!(data.orbit, vec![2, 3, 1]);

    let data = schreier.get_orbit(3);
    assert_eq!(data.orbit, vec![3, 1, 2]);






}
