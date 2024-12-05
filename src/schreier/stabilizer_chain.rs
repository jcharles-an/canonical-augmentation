use std::collections::HashMap;
use std::io::Chain;
use std::ops::Mul;
use num_bigint::{BigInt, BigUint};
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;
use crate::schreier::permutation_group::PermutationGroup;

#[derive(Debug, Eq, PartialEq)]
pub struct StabilizerChain<G:Group>{
    group: G,
    orbit: Vec<G::Target>,
    transversal: HashMap<G::Target, G::Element>,
    stabilizer: Option<Box<StabilizerChain<G>>>,
    full_chain: bool
}

type G = PermutationGroup;

impl StabilizerChain<G>{

    fn new(group: G,
           orbit: Vec<<G as Group>::Target>,
           transversal: HashMap<<G as Group>::Target, <G as Group>::Element>,
           stabilizer: Option<Box<StabilizerChain<G>>>,
           full_chain: bool
    )->StabilizerChain<G>{
        StabilizerChain{
            group,
            orbit,
            transversal,
            stabilizer,
            full_chain
        }
    }

    fn empty()->StabilizerChain<G>{
        StabilizerChain::<G>::new(
            G::new_empty(),
            vec![],
            HashMap::new(),
            None,
            false
        )
    }

    fn init(group: G)->StabilizerChain<G>{
        StabilizerChain::<G>::new(
            group,
            vec![],
            HashMap::new(),
            None,
            false
        )
    }

    fn factorize(&self,element: &<G as Group>::Element) -> Option<Vec<&<G as Group>::Element>> {
        let mut factorization:Vec<&<G as Group>::Element> = vec![];
        let mut current_stab_chain = self;
        let mut element_placeholder = (*element).clone();
        while !current_stab_chain.group.get_generators().is_empty() {
            let origin = current_stab_chain.orbit.first()?;
            let transformed = G::apply_to(&element_placeholder, origin);
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

    fn test_belonging(&self,element: &<G as Group>::Element) -> bool {
        self.factorize(element).is_some()
    }

    pub(crate) fn get_stabilizer_chain(generators: Vec<<G as Group>::Element>) -> StabilizerChain<G>{
        let mut chain = Self::empty();
        let first_base_element = 0;
        for generator in generators{
            Self::extend_chain(&mut chain,&generator,first_base_element);
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
        chain: & mut StabilizerChain<G>,
        g: &<G as Group>::Element,
        mut origin: <G as Group>::Target ){
        if let Some(_) = chain.factorize(g){
            return;
        }
        if chain.group.get_generators().len() == 0 {
            chain.stabilizer = Some(Box::new(Self::empty()));
            //TODO: Get rid of this clone
            chain.group.add_generator(g.clone());
            let mut transformed = G::apply_to(g,&origin);
            while transformed == origin {
                origin += 1;
                transformed = G::apply_to(g,&origin);
            }

            chain.orbit.push(origin);
            chain.transversal.insert(origin,chain.group.get_identity());

            let mut traversal_representative = (*g).clone();
            while origin != transformed{
                chain.orbit.push(transformed);
                chain.transversal.insert(transformed, traversal_representative.clone());
                transformed = G::apply_to(g,&transformed);
                traversal_representative = G::operate(&traversal_representative,g)
            }
            //let stab = chain.stabilizer.as_mut().unwrap().as_mut();
            //Self::extend_chain(stab,&traversal_representative,origin + 1);

        } else {
            let orbit_len = chain.orbit.len();
            let mut orbit_index = 0;
            while orbit_index < orbit_len {
                Self::extend_orbit_or_chain(chain, g,orbit_index,origin + 1);
                orbit_index = orbit_index + 1;
            }
            //TODO: Get rid of this clone
            chain.group.add_generator(g.clone());
            let generators = chain.group.get_generators().clone();

            let mut current_length = chain.orbit.len();
            while orbit_index < current_length {
                for generator in &generators{
                    Self::extend_orbit_or_chain(chain, generator, orbit_index,origin + 1);
                }
                orbit_index = orbit_index + 1;
                current_length = chain.orbit.len();
            }
            chain.full_chain = true;
        }

    }

    fn extend_orbit_or_chain( chain: & mut StabilizerChain<G>, g: &<G as Group>::Element, orbit_index: usize, origin: <G as Group>::Target){
        let mut orbit = &mut chain.orbit;
        let mut traversal = &mut chain.transversal;

        let transformed = G::apply_to(g,&orbit[orbit_index]) ;
        if !orbit.contains(&transformed){ //TODO: is this contains efficient? Says its O(n)
            orbit.push(transformed);
            traversal.insert(transformed, G::operate(g,&traversal[&orbit[orbit_index]]));
        } else {
            let transformed_traversal = traversal.get(&transformed).unwrap();
            //TODO. This is not the most efficient way to do this, we should be saving inverse in the transversal
            let next_generator = G::operate(&G::get_inverse(&transformed_traversal), &G::operate(g,&traversal[&orbit[orbit_index]]));
            Self::extend_chain(chain.stabilizer.as_mut().unwrap().as_mut(), &next_generator,origin)
        }
    }

}


#[test]
fn test_stabilizer_chain(){

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

    let permutation = Permutation::new(vec![1,2,0,3]);
    let permutation2 = Permutation::new(vec![0,2,3,1]);
    let product = PermutationGroup::operate(&permutation,&permutation2);


    let fact = sc.factorize(&product);
    println!("{:?} factorizes into {:?}\n", product, fact);


    let c = StabilizerChain::get_stabilizer_chain(vec![
        Permutation::new(vec![1,2,0,3]),
        Permutation::new(vec![0,2,3,1]),
    ]);
    assert_eq!(sc,c);
    //println!("{c:?}");
    assert_eq!(c.get_order(),BigUint::from(12usize));

    let target = Permutation::new(vec![0,3,1,2]);
    assert!(c.test_belonging(&target));
    let target = Permutation::new(vec![1,0,3,2]);
    assert!(c.test_belonging(&target));
    let target = Permutation::new(vec![1,3,2,0]);
    assert!(c.test_belonging(&target));
    let target = Permutation::new(vec![2,0,1,3]);
    assert!(c.test_belonging(&target));


    let non_belonging = Permutation::new(vec![0,1,3,2]);
    assert!(!c.test_belonging(&non_belonging));
    let non_belonging = Permutation::new(vec![0,2,1,3]);
    assert!(!c.test_belonging(&non_belonging));
    let non_belonging = Permutation::new(vec![0,3,2,1]);
    assert!(!c.test_belonging(&non_belonging));


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




    //let sc = StabilizerChain::get_stabilizer_chain(vec![u,l,f,r,b,d]);
    let perm = PermutationGroup::operate(&u,&r);
    println!("perm: {perm:?}\n");
    let sc = StabilizerChain::get_stabilizer_chain(vec![perm]);
    let order = sc.get_order();
    println!("order: {order}");
    assert_eq!(order,"43252003274489856000".parse().unwrap());
    //println!("{sc:?}");




}
