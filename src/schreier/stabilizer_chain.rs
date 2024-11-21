use std::collections::HashMap;
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;
use crate::schreier::permutation_group::PermutationGroup;

pub struct StabilizerChain<G:Group>{

    generators: Vec<G::Element>,
    orbit: Vec<G::Target>,
    transversal: HashMap<G::Target, G::Element>,
    stabilizer: Option<Box<StabilizerChain<G>>>,
}

impl<G:Group> StabilizerChain<G>{

    fn new(generators: Vec<G::Element>,
           orbit: Vec<G::Target>,
           transversal: HashMap<G::Target, G::Element>,
           stabilizer: Option<Box<StabilizerChain<G>>>)->StabilizerChain<G>{
        StabilizerChain{
            generators,
            orbit,
            transversal,
            stabilizer
        }
    }
    fn factorize(&self,element: &G::Element) -> Option<Vec<&G::Element>> {
        let mut factorization:Vec<&G::Element> = vec![];
        let mut current_stab_chain = self;
        let mut element_placeholder = (*element).clone();
        while !current_stab_chain.generators.is_empty() {
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
}


#[test]
fn test_stabilizer_chain(){

    let sc = StabilizerChain::new(
        vec![
            Permutation::new(vec![2,3,1,4]),
            Permutation::new(vec![1,3,4,2]),
        ],
        vec![1,2,3,4],
        HashMap::from([
            (1,Permutation::new(vec![1,2,3,4])),
            (2,Permutation::new(vec![2,3,1,4])),
            (3,Permutation::new(vec![3,1,2,4])),
            (4,Permutation::new(vec![4,1,3,2])),
        ]),
        Some(Box::new(StabilizerChain::new(
            vec![
                Permutation::new(vec![1,3,4,2]),
            ],
            vec![2,3,4],
            HashMap::from([
                (2,Permutation::new(vec![1,2,3,4])),
                (3,Permutation::new(vec![1,3,4,2])),
                (4,Permutation::new(vec![1,4,2,3])),
            ]),
            Some(Box::new(StabilizerChain::<PermutationGroup>::new(
                vec![],
                vec![],
                HashMap::new(),
                None,
            )))
        )))
    );

    let permutation = Permutation::new(vec![2,3,1,4]);
    let permutation2 = Permutation::new(vec![1,3,4,2]);

    let product = PermutationGroup::operate(&permutation2,&permutation2);

    let fact = sc.factorize(&permutation);
    println!("{:?} factorizes into {:?}\n",permutation, fact);


    let fact = sc.factorize(&product);
    println!("{:?} factorizes into {:?}\n", product, fact);

    let fact = sc.factorize(&Permutation::new(vec![1,2,4,3]));
    println!("{:?} factorizes into {:?}\n", product, fact);




}
