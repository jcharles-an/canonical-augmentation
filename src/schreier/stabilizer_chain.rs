use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Mul;
use num_bigint::{ BigUint};
use rand::rngs::ThreadRng;
use crate::group_actions::groupAction::GroupAction;
use crate::data_structs::permutation::Permutation;
use crate::groups::permutation_group::PermutationGroup;
use rand::seq::SliceRandom;
use crate::groups::group::Group;
use crate::group_actions::permute_points::PermutePoints;

#[derive(Eq, PartialEq)]
pub struct RecStabilizerChain<G:Group ,GA: GroupAction>{
    group: G,
    orbit: Vec<GA::Target>,
    transversal: HashMap<GA::Target, G::Element>,
    stabilizer: Option<Box<RecStabilizerChain<G, GA>>>,
    full_chain: bool
}

type G = PermutationGroup;
type GA = PermutePoints;

impl Debug for RecStabilizerChain<G, GA> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "transversal: {:?}\nstabilizer: {:?}",self.transversal, self.stabilizer)
    }
}

impl RecStabilizerChain<G, GA>{

    fn new(group: G,
           orbit: Vec<<GA as GroupAction>::Target>,
           transversal: HashMap<<GA as GroupAction>::Target, <G as Group>::Element>,
           stabilizer: Option<Box<RecStabilizerChain<G, GA>>>,
           full_chain: bool
    ) -> RecStabilizerChain<G,GA>{
        RecStabilizerChain {
            group,
            orbit,
            transversal,
            stabilizer,
            full_chain
        }
    }

    fn empty()-> RecStabilizerChain<G,GA>{
        RecStabilizerChain::<G,GA>::new(
            G::new_empty(),
            vec![],
            HashMap::new(),
            None,
            false
        )
    }

    fn init(group: G)-> RecStabilizerChain<G,GA>{
        RecStabilizerChain::<G,GA>::new(
            group,
            vec![],
            HashMap::new(),
            None,
            false
        )
    }

    fn factorize(&self, element: &<G as Group>::Element) -> Option<Vec<&<G as Group>::Element>> {
        let mut factorization:Vec<&<G as Group>::Element> = vec![];
        let mut current_stab_chain = self;
        let mut element_placeholder = (*element).clone();
        while !current_stab_chain.group.get_generators().is_empty() {
            let origin = current_stab_chain.orbit.first()?;
            let transformed = GA::apply_to(&element_placeholder, origin);
            let b = current_stab_chain.transversal.get(&transformed)?;
            element_placeholder = G::operate(&G::get_inverse(b),&element_placeholder,);
            factorization.push(b);
            current_stab_chain = current_stab_chain.stabilizer.as_ref()?.as_ref();
            //println!("origin: {origin:?}\ntransformed: {transformed:?}\nreresentative: {b:?}\nafter inverse: {:?}\ncurrent factors: {:?}\n",element_placeholder,factorization);
        }
        if !G::is_identity(&element_placeholder){
            return None
        }
        Some(factorization)
    }

    pub fn test_belonging(&self, element: &<G as Group>::Element) -> bool {
        self.factorize(element).is_some()
    }

    pub(crate) fn get_stabilizer_chain(generators: Vec<<G as Group>::Element>) -> RecStabilizerChain<G,GA>{
        let mut chain = Self::empty();
        let mut rng = rand::thread_rng();
        for generator in generators{
            Self::extend_chain(&mut chain,&generator,&mut rng);
        }
        chain.full_chain = true;
        chain
    }


    pub(crate) fn get_order(&self) -> BigUint{
        if !self.full_chain {
            //TODO: finish full chain
        }
        let mut sc = self;
        let mut order = BigUint::default();
        let mut simple_order = 1usize;
        let mut is_big = false;
        while !sc.group.get_generators().is_empty() {
            let orbit_len = sc.orbit.len();
            println!("Orbit len: {}", orbit_len);
            if is_big {
                order = order.mul(BigUint::from(orbit_len));
            } else{
                if let Some(result) = simple_order.checked_mul(orbit_len) {
                    simple_order = result;
                } else{
                    is_big = true;
                    order = BigUint::from(simple_order);
                    order = order.mul(BigUint::from(orbit_len));
                };
            }
            sc = sc.stabilizer.as_ref().unwrap().as_ref();
        }
        if !is_big {
            order = BigUint::from(simple_order);
        }
        order
    }

    fn extend_chain(
        chain: & mut RecStabilizerChain<G,GA>,
        g: &<G as Group>::Element,
        rng: &mut ThreadRng
    ){
        if let Some(_) = chain.factorize(g){
            return;
        }
        if chain.group.get_generators().len() == 0 {
            Self::calculate_single_generator_chain(chain, g,rng);
        } else { //an orbit already exists
            let orbit_len = chain.orbit.len();
            let mut orbit_index = 0;
            while orbit_index < orbit_len {
                Self::extend_orbit_or_chain(chain, g, orbit_index, rng);
                orbit_index = orbit_index + 1;
            }
            //TODO: Get rid of this clone
            chain.group.add_generator(g.clone());
            let generators = chain.group.get_generators().clone();

            let mut current_length = chain.orbit.len();
            while orbit_index < current_length {
                for generator in &generators{
                    Self::extend_orbit_or_chain(chain, generator, orbit_index, rng);
                }
                orbit_index = orbit_index + 1;
                current_length = chain.orbit.len();
            }
            chain.full_chain = true;
        }

    }

    fn calculate_single_generator_chain(
        chain: & mut RecStabilizerChain<G,GA>,
        g: &<G as Group>::Element,
        rng: &mut ThreadRng
    ){
        let orbit_origin = Self::get_orbit_origin(g,rng);
        chain.group.add_generator(g.clone());
        chain.orbit.push(orbit_origin);
        chain.transversal.insert(orbit_origin, chain.group.get_identity());
        //init new layer
        chain.stabilizer = Some(Box::new(Self::empty()));

        let mut next_orbit_element = GA::apply_to(g, &orbit_origin);

        let mut traversal_representative = (*g).clone();
        while orbit_origin != next_orbit_element {
            chain.orbit.push(next_orbit_element);
            chain.transversal.insert(next_orbit_element, traversal_representative.clone());
            next_orbit_element = GA::apply_to(g, &next_orbit_element);
            traversal_representative = G::operate(g, &traversal_representative)
        }
        let stab = chain.stabilizer.as_mut().unwrap().as_mut();
        Self::extend_chain(stab,&traversal_representative,rng);
    }

    fn extend_orbit_or_chain(
        chain: & mut RecStabilizerChain<G,GA>,
        g: &<G as Group>::Element,
        orbit_index: usize,
        rng: &mut ThreadRng
    ){
        let mut orbit = &mut chain.orbit;
        let mut traversal = &mut chain.transversal;
        let orbit_element = &orbit[orbit_index];
        let current_representative = &traversal[orbit_element];

        let next_orbit_element = GA::apply_to(g, orbit_element) ;
        if !orbit.contains(&next_orbit_element){ //TODO: is this contains efficient? Says its O(n)
            orbit.push(next_orbit_element);
            traversal.insert(next_orbit_element, G::operate(g, current_representative));
        } else {
            let next_representative = &traversal[&next_orbit_element];
            //TODO. This is not the most efficient way to do this, we should be saving inverse in the transversal
            let next_generator = G::operate(&G::get_inverse(&next_representative), &G::operate(g, current_representative));
            Self::extend_chain(chain.stabilizer.as_mut().unwrap().as_mut(), &next_generator,rng)
        }
    }

    fn get_orbit_origin(g: &<G as Group>::Element, rng: &mut ThreadRng) -> <GA as GroupAction>::Target {
        let mut moved = Vec::new();
        for (index, value) in g.map.iter().enumerate(){
            if &index != value{
                moved.push(*value);
            }
        }
        *moved.choose(rng).unwrap()
    }
}


#[test]
fn test_schreir_stab() {
    let sc = RecStabilizerChain::get_stabilizer_chain(vec![
        Permutation::from_cycles(vec![vec![0 ,1 ,3 ,2], vec![4,5,7,6]]),
        Permutation::from_cycles(vec![vec![0 ,1 ,3 ,7 ,6,4], vec![2,5]]),
    ]);

    println!("{:?}", sc);
}

#[test]
fn test_stabilizer_chain(){
    /*
    let sc = StabilizerChain::new(
        PermutationGroup::new(vec![
            Permutation::new(vec![1,2,0,3]),
            Permutation::new(vec![0,2,3,1]),
        ]),
        vec![0,1,2,3],
        HashMap::from([
            (0,Permutation::new(vec![0,1,2,3])),
            (1,Permutation::new(vec![1,2,0,3])),
            (2,Permutation::new(vec![2,0,1,3])),
            (3,Permutation::new(vec![3,0,2,1])),
        ]),
        Some(Box::new(StabilizerChain::new(
            PermutationGroup::new(vec![
                Permutation::new(vec![0,2,3,1]),
            ]),
            vec![1,2,3],
            HashMap::from([
                (1,Permutation::new(vec![0,1,2,3])),
                (2,Permutation::new(vec![0,2,3,1])),
                (3,Permutation::new(vec![0,3,1,2])),
            ]),
            Some(Box::new(StabilizerChain::<PermutationGroup>::empty())),
            false
        ))),
        true
    );
    */



    let permutation = Permutation::new(vec![1,2,0,3]);
    let permutation2 = Permutation::new(vec![0,2,3,1]);

    let product = PermutationGroup::operate(&permutation,&permutation2);

    let sc = RecStabilizerChain::get_stabilizer_chain(vec![
        permutation, permutation2,
    ]);

    let fact = sc.factorize(&product);
    println!("{:?} factorizes into {:?}\n", product, fact);

    //println!("{c:?}");
    assert_eq!(sc.get_order(),BigUint::from(12usize));

    let mut target :Permutation;
    target = Permutation::new(vec![0,3,1,2]);
    assert!(sc.test_belonging(&target));
    target = Permutation::new(vec![1,0,3,2]);
    assert!(sc.test_belonging(&target));
    target = Permutation::new(vec![1,3,2,0]);
    assert!(sc.test_belonging(&target));
    target = Permutation::new(vec![2,0,1,3]);
    assert!(sc.test_belonging(&target));


    let mut non_belonging : Permutation;
    non_belonging = Permutation::new(vec![0,1,3,2]);
    assert!(!sc.test_belonging(&non_belonging));
    non_belonging = Permutation::new(vec![0,2,1,3]);
    assert!(!sc.test_belonging(&non_belonging));
    non_belonging = Permutation::new(vec![0,3,2,1]);
    assert!(!sc.test_belonging(&non_belonging));




    let u =  Permutation::from_cycles(vec![
        vec![0, 2, 7, 5],
        vec![1, 4, 6, 3],
        vec![8,32,24,16],
        vec![9,33,25,17],
        vec![10,34,26,18],
    ]);

    let l = Permutation::from_cycles(vec![
        vec![8,10,15,13],
        vec![9,12,14,11],
        vec![0,16,40,39],
        vec![3,19,43,36],
        vec![5,21,45,34],
    ]);
    let f = Permutation::from_cycles(vec![
        vec![16,18,23,21],
        vec![17,20,22,19],
        vec![5,24,42,15],
        vec![6,27,41,12],
        vec![7,29,40,10],
    ]);
    let r = Permutation::from_cycles(vec![
        vec![24,26,31,29],
        vec![25,28,30,27],
        vec![2,37,42,18],
        vec![4,35,44,20],
        vec![7,32,47,23],
    ]);
    let b = Permutation::from_cycles(vec![
        vec![32,34,39,37],
        vec![33,36,38,35],
        vec![2, 8,45,31],
        vec![1,11,46,28],
        vec![0,13,47,26],
    ]);
    let d = Permutation::from_cycles(vec![
        vec![40,42,47,45],
        vec![41,44,46,43],
        vec![13,21,29,37],
        vec![14,22,30,38],
        vec![15,23,31,39],
    ]);




    let mut perm = PermutationGroup::operate
        (&u,&r);
    //perm = PermutationGroup::operate(&l,&perm);
    //perm = PermutationGroup::operate(&d,&perm);

    println!("perm: {perm:?}\n");

    let sc = RecStabilizerChain::get_stabilizer_chain(vec![perm]);
    println!("sc: {sc:?}\n");
    let order = sc.get_order();
    println!("order: {order}\n\n");



    let sc = RecStabilizerChain::get_stabilizer_chain(vec![u, l, f, r, b]);
    let order = sc.get_order();
    println!("order: {order}");
    assert_eq!(order,"43252003274489856000".parse().unwrap());
    //println!("{sc:?}");






}
