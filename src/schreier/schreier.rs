use std::collections::HashSet;
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;
use crate::schreier::permutation_group::PermutationGroup;

struct SchreierVector<T,U>{
    list: Vec<Option<T>>,
    origin: U,
}

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
        let mut orbit = vec![None::<<G as Group>::Target>;self.group.get_range()];
        let mut traversal =  vec![None::<<G as Group>::Element>;self.group.get_range()];
        let mut schreier_vector = SchreierVector{
            list: vec![None::<&<G as Group>::Element>;self.group.get_range()],
            origin,
        };

        traversal[origin] = Some(self.group.get_identity());
        schreier_vector.list[origin] = Some(self.group.get_identity_ref());
        let mut stab_generators = vec![self.group.get_identity()];

        let mut orbit_index = 0;
        orbit[origin] = Some(origin);
        //orbit_map.insert(origin, orbit_index);

         while orbit_index < orbit_vec.len(){
            for generator in self.group.get_generators(){
                let orbit_element = orbit_vec[orbit_index];
                let next_orbit_element = G::apply_to(generator, &orbit_element);
                let traversal_representative = traversal[orbit_element].as_ref().unwrap();

                if orbit[next_orbit_element] == None {
                    orbit_vec.push(next_orbit_element);
                    orbit[next_orbit_element] = Some(orbit_element);
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



    //BOOK IMPLEMENTATIONS

    fn test_pertenece(&self, n: usize, g: &mut <G as Group>::Element, stab_chain: &Vec<Vec<<G as Group>::Element>>) -> usize{
        for i in 0..n{
            let x = G::apply_to(g, &i);
            let mut found_h = None;
            // is there any h in subgroups of stab_chain that goes to x?
            for h in &stab_chain[i]{
                if G::apply_to(h, &i) == x {
                    found_h = Some(h);
                    break;
                }
            }
            if let Some(h) = found_h {
                let h_inv = G::get_inverse(h);
                let product =  G::operate(&h_inv, g);
                for j in 0..n{
                    g.map[j] = G::apply_to(&product, &j);
                }

            } else {
                return i;
            }
        }
        n
    }


    fn generate_stab_chain(&self) -> Vec<Vec<Permutation>> {
        let n = self.group.get_range();
        let mut stab_chain = vec![vec![self.group.get_identity()];n];
        for generator in self.group.get_generators() {
            let mut generator = generator.clone();
            self.enter(n, &mut generator, &mut stab_chain,0);
        }
        for generator in self.group.get_generators() {
            let mut generator = generator.clone();
            self.enter(n, &mut generator, &mut stab_chain,0);
        }

        println!("Stab_chain is:");
        for (index, stab) in stab_chain.iter().enumerate() {
            println!("{index} : {:?}", stab)
        }

        stab_chain
    }

    fn print_format(tier:usize){
        for i in 0..tier{
            print!("\t");
        }
    }

    fn enter(&self, n: usize, generator: &mut <G as Group>::Element, stab_chain: &mut Vec<Vec<<G as Group>::Element>>, tier:usize){
        Self::print_format(tier);println!("testing: {generator:?}");
        let i = self.test_pertenece(n, generator, stab_chain);
        Self::print_format(tier);println!("got: {generator:?} with index {i}");
        if i == n {
            println!();
            return;
        }
        stab_chain[i].push(generator.clone());
        Self::print_format(tier);println!("Introducing in stabchain: {{");
        for (index, stab) in stab_chain.iter().enumerate(){
            Self::print_format(tier);println!("\t{index}: {:?}",stab);
        }
        Self::print_format(tier);println!("}}");

        let generator = &(*generator);
        for j in 0..n{
            let mut k = 0;
            let len = stab_chain[j].len();
            while k < len{
                let h = &stab_chain[j][k];
                if G::is_identity(h) {
                    k+=1;
                    continue;
                }
                let mut product = G::operate(generator,h);
                Self::print_format(tier);println!("{generator:?} * {h:?} = {product:?}");
                self.enter(n, &mut product, stab_chain, tier + 1);
                k += 1;
            }
        }
        println!();

    }


    fn simple_generate_group(&self, n: usize, generators: &Vec<<G as Group>::Element>){
        let mut counter = 0;
        let mut full_group: HashSet<<G as Group>::Element> = HashSet::new();
        let mut new = vec![self.group.get_identity()];
        while !new.is_empty(){
            let last = new.clone();
            while let Some(element) = new.pop(){
                full_group.insert(element);
            }
            for g in generators{
                for h in &last{
                    let f = G::operate(g,h);
                    if !full_group.contains(&f){
                        new.push(f);
                        counter += 1;
                    }
                }
            }
        }
        println!("{} full_group:{:?}", full_group.len() ,full_group);
    }

    fn simple_gen(&self){
        let n = self.group.get_range();
        self.simple_generate_group(n,self.group.get_generators())
    }

    fn run(&self, stab_chain: & Vec<Vec<<G as Group>::Element>>){
        Self::run_backtrack(self.group.get_range(),0,stab_chain,self.group.get_identity())
    }

    fn run_backtrack(n: usize, l: usize, stab_chain: & Vec<Vec<<G as Group>::Element>>, g:<G as Group>::Element ){
        if l == n{
            println!("{g:?}");
        } else {
            for h in &stab_chain[l]{
                let f = G::operate(&g, h);
                Self::run_backtrack(n, l + 1, stab_chain, f)
            }
        }
    }

}


#[test]
fn orbit_test() {
    /*
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


     */

    let mut schreir = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![0 ,1 ,3 ,7 ,6,4], vec![2,5]]),
                Permutation::from_cycles(vec![vec![0 ,1 ,3 ,2], vec![4,5,7,6]]),
            ]));

    // schreir.simple_gen();

    let stab_chain = schreir.generate_stab_chain();

    println!("Generated group with the stab chain:");
    schreir.run(&stab_chain);
}

