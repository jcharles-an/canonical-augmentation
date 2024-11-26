use std::collections::HashMap;
use std::io::Chain;
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;
use crate::schreier::permutation_group::PermutationGroup;

pub struct StabilizerChain<G:Group>{
    group: G,
    orbit: Vec<G::Target>,
    transversal: HashMap<G::Target, G::Element>,
    stabilizer: Option<Box<StabilizerChain<G>>>,
}

type G = PermutationGroup;

impl StabilizerChain<G>{

    fn new(group: G,
           orbit: Vec<<G as Group>::Target>,
           transversal: HashMap<<G as Group>::Target, <G as Group>::Element>,
           stabilizer: Option<Box<StabilizerChain<G>>>)->StabilizerChain<G>{
        StabilizerChain{
            group,
            orbit,
            transversal,
            stabilizer
        }
    }

    fn empty()->StabilizerChain<G>{
        StabilizerChain::<G>::new(
            G::new_empty(),
            vec![],
            HashMap::new(),
            None,
        )
    }
    fn factorize(&self,element: &<G as Group>::Element) -> Option<Vec<&<G as Group>::Element>> {
        let mut factorization:Vec<&<G as Group>::Element> = vec![];
        let mut current_stab_chain = self;
        let mut element_placeholder = (*element).clone();
        while !current_stab_chain.group.get_generators().is_empty() {
            let origin = current_stab_chain.orbit.first()?;
            let transformed = G::apply_to(&element_placeholder, origin)?;
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

    fn get_stabilizer_chain(&self, generators: Vec<<G as Group>::Element>) -> StabilizerChain<G>{
        let mut chain = Self::empty();
        for generator in generators{
            self.extend_chain(&mut chain,&generator,generator.map[0]);
        }
        chain
    }

    fn extend_chain(
        &self,
        chain: & mut StabilizerChain<G>,
        g: &<G as Group>::Element,
        origin: <G as Group>::Target ){
        if let Some(_) = chain.factorize(g){
            return;
        }
        if chain.group.get_generators().len() == 0 {
            chain.stabilizer = Some(Box::new(Self::empty()));
            //TODO: Get rid of this clone
            chain.group.add_generator(g.clone());
            let mut transformed = None;
            while transformed == None {
                transformed = G::apply_to(g,&origin)
            }
            let mut transformed = transformed.unwrap();

            chain.orbit.push(origin);
            chain.transversal.insert(origin,self.group.get_identity());

            let mut traversal_representative = (*g).clone();
            while origin != transformed{
                chain.orbit.push(transformed);
                chain.transversal.insert(transformed, traversal_representative.clone());
                transformed = G::apply_to(g,&transformed).unwrap();
                traversal_representative = G::operate(&traversal_representative,g)
            }
            self.extend_chain(chain,&traversal_representative,origin);

        } else {
            let orbit_len = chain.orbit.len();
            let mut orbit_index = 0;
            while orbit_index < orbit_len {
                self.extend_orbit_or_chain(chain, g,chain.orbit[orbit_index],origin);
                orbit_index = orbit_index + 1;
            }
            //TODO: Get rid of this clone
            chain.group.add_generator(g.clone());
            let generators = chain.group.get_generators().clone();

            while orbit_index < chain.orbit.len() {
                for generator in &generators{
                    self.extend_orbit_or_chain(chain, generator, chain.orbit[orbit_index],origin);
                    orbit_index = orbit_index + 1;
                }
            }
        }

    }

    fn extend_orbit_or_chain(&self, chain: & mut StabilizerChain<G>, g: &<G as Group>::Element, orbit_index: usize, origin: <G as Group>::Target){
        let mut orbit = &mut chain.orbit;
        let mut traversal = &mut chain.transversal;

        let Some(transformed) = G::apply_to(g,&orbit[orbit_index]) else { todo!() };
        if !orbit.contains(&transformed){ //TODO: is this contains efficient? Says its O(n)
            orbit.push(transformed);
            traversal.insert(transformed, G::operate(g,&traversal[&orbit[orbit_index]]));
        } else {
            let transformed_traversal = traversal.get(&transformed).unwrap();
            //TODO. This is not the most efficient way to do this
            let next_generator = G::operate(&G::get_inverse(&transformed_traversal), &G::operate(g,&traversal[&orbit[orbit_index]]));
            self.extend_chain(chain, &next_generator,origin)
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
            (1,Permutation::new(vec![0,1,2,3])),
            (2,Permutation::new(vec![1,2,0,3])),
            (3,Permutation::new(vec![2,0,1,3])),
            (4,Permutation::new(vec![3,0,2,1])),
        ]),
        Some(Box::new(StabilizerChain::new(
            PermutationGroup::new(vec![
                Permutation::new(vec![0,2,3,1]),
            ]),
            vec![1,2,3],
            HashMap::from([
                (2,Permutation::new(vec![0,1,2,3])),
                (3,Permutation::new(vec![0,2,3,1])),
                (4,Permutation::new(vec![0,3,1,2])),
            ]),
            Some(Box::new(StabilizerChain::<PermutationGroup>::empty()))
        )))
    );

    let permutation = Permutation::new(vec![1,2,0,3]);
    let permutation2 = Permutation::new(vec![0,2,3,1]);

    let product = PermutationGroup::operate(&permutation2,&permutation2);

    let fact = sc.factorize(&permutation);
    println!("{:?} factorizes into {:?}\n",permutation, fact);


    let fact = sc.factorize(&product);
    println!("{:?} factorizes into {:?}\n", product, fact);

    let fact = sc.factorize(&Permutation::new(vec![0,1,3,4]));
    println!("{:?} factorizes into {:?}\n", product, fact);




}
