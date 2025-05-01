use std::cmp;
use std::collections::HashSet;
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;
use crate::schreier::permutation_group::PermutationGroup;

struct SchreierVector<T,U>{
    list: Vec<Option<T>>,
    origin: U,
}

pub(crate) struct Schreier<T>
where
    T: Group
{
    pub group: T,
    v_3: Vec<usize>
}

type G = PermutationGroup;

impl Schreier<G>{

    pub fn new(generators: G) -> Schreier<G>{
        Schreier{
            group: generators,
            v_3: Vec::new()
        }
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

    // Algorithm 6.5
    fn simple_gen(&self){
        let n = self.group.get_range();
        self.simple_generate_group(n,self.group.get_generators())
    }

    // Algorithm 6.5.1
    //Naïve algorithm to generate a full group from their generators
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

    // Algorithm 6.6
    fn run(&mut self, stab_chain: & Vec<Vec<<G as Group>::Element>>){
        self.run_backtrack(self.group.get_range(),0,stab_chain,self.group.get_identity(),Self::print_result)
    }

    // Algorithm 6.6.1
    //process-leaf might be an attribute in the struct
    fn run_backtrack(&mut self, n: usize, l: usize, stab_chain: & Vec<Vec<<G as Group>::Element>>, g:<G as Group>::Element, process_leaf: fn(&mut Schreier<G>, n: usize, g:<G as Group>::Element) ){
        if l == n{
            process_leaf(self, n, g)
        } else {
            for h in &stab_chain[l]{
                let f = G::operate(&g, h);
                self.run_backtrack(n, l + 1, stab_chain, f, process_leaf)
            }
        }
    }

    // Algorithm 6.6.2
    // Simple callback for when finding leave
    fn print_result(&mut self, n: usize, g:<G as Group>::Element ){
        println!("{g:?}");
    }



    //algorithm 6.8
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


    // Algorithm 6.9
    fn generate_stab_chain(&self) -> Vec<Vec<Permutation>> {
        let n = self.group.get_range();
        let mut stab_chain = vec![vec![self.group.get_identity()];n];
        let mut max_depth = 0;
        for generator in self.group.get_generators() {
            let mut generator = generator.clone();
            self.enter(n, &mut max_depth,&mut generator, &mut stab_chain,0);
        }

        println!("Stab_chain is:");
        for (index, stab) in stab_chain.iter().enumerate() {
            println!("{index} : {:?}", stab)
        }

        stab_chain
    }

    // Algorithm 6.9.1
    fn enter(&self, n: usize, max_depth: &mut usize, generator: &mut <G as Group>::Element, stab_chain: &mut Vec<Vec<<G as Group>::Element>>, tier:usize){
        //Self::print_format(tier);println!("testing: {generator:?}");
        let i = self.test_pertenece(n, generator, stab_chain);
        //Self::print_format(tier);println!("got: {generator:?} with index {i}");
        if i == n {
            //println!();
            return;
        }
        //update max depth if needed
        *max_depth = cmp::max(*max_depth, i);
        //push multiplied permutation into group i
        stab_chain[i].push(generator.clone());
        //Self::print_format(tier);println!("Introducing in stabchain: {{");
        for (index, stab) in stab_chain.iter().enumerate(){
            //Self::print_format(tier);println!("\t{index}: {:?}",stab);
        }
        //Self::print_format(tier);println!("}}");

        let generator = &(*generator);
        let current_max_depth = *max_depth;
        for j in 0..current_max_depth{
            let mut k = 0;
            let len = stab_chain[j].len();
            while k < len{
                let h = &stab_chain[j][k];
                if G::is_identity(h) {
                    k+=1;
                    continue;
                }
                let mut product = G::operate(generator,h);
                //Self::print_format(tier);println!("{generator:?} * {h:?} = {product:?}");
                self.enter(n, max_depth,&mut product, stab_chain, tier + 1);
                k += 1;
            }
        }
        //println!();

    }

    fn get_group_order(stab_chain: &Vec<Vec<<G as Group>::Element>>) -> usize{
        stab_chain.iter().map(|x| x.len()).product()
    }

    fn print_format(tier:usize){
        for i in 0..tier{
            print!("\t");
        }
    }


    //Algorithm 6.10
    fn test_pertenece_2(&self, n: usize, g: &mut <G as Group>::Element, stab_chain: &Vec<Vec<<G as Group>::Element>>, base: &<G as Group>::Element) -> usize{
        for i in 0..n{
            let permuted_i = (G::apply_to(base, &i));
            let x = G::apply_to(g, &permuted_i);
            let mut found_h = None;
            // is there any h in subgroups of stab_chain that goes to x?
            for h in &stab_chain[i]{
                if G::apply_to(h, &permuted_i) == x {
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

    // Algorithm 6.11
    pub fn enter_2(&self, n: usize, max_depth: &mut usize, generator: &mut <G as Group>::Element, stab_chain: &mut Vec<Vec<<G as Group>::Element>>, base: &<G as Group>::Element){

        let i = self.test_pertenece_2(n, generator, stab_chain, base);
        if i == n {
            return;
        }
        //update max depth if needed
        *max_depth = cmp::max(*max_depth, i);
        //push multiplied permutation into group i
        stab_chain[i].push(generator.clone());

        let generator = &(*generator);
        let current_max_depth = *max_depth;
        for j in 0..current_max_depth{
            let mut k = 0;
            let len = stab_chain[j].len();
            while k < len{
                let h = &stab_chain[j][k];
                if G::is_identity(h) {
                    k+=1;
                    continue;
                }
                let mut product = G::operate(generator,h);
                //Self::print_format(tier);println!("{generator:?} * {h:?} = {product:?}");
                self.enter_2(n, max_depth,&mut product, stab_chain, base);
                k += 1;
            }
        }
        //println!();

    }

    //Algorithm 6.12
    pub fn change_base(&self, n: usize,
                   stab_chain: &mut Vec<Vec<<G as Group>::Element>>,
                   new_base: &<G as Group>::Element) -> Vec<Vec<<G as Group>::Element>>{

        let mut new_stab_chain : Vec<Vec<<G as Group>::Element>> = vec![vec![ self.group.get_identity()]; n];
        let mut max_depth = 0;
        for j in 0..n{
            for g in stab_chain[j].iter_mut() {
                self.enter_2(n,&mut max_depth,g,&mut new_stab_chain,new_base);
            }
        }
        new_stab_chain
    }




    // Algorithm 6.13.0
    fn norb(&mut self, stab_chain: & Vec<Vec<<G as Group>::Element>>){
        let n = self.group.get_range();

        self.v_3 = vec![0; n];
        self.run_backtrack(n, 0, stab_chain,self.group.get_identity(), Self::norb_use);

        let mut N = vec![0; n];
        let order = Self::get_group_order(stab_chain);
        for k in 0..n {
            N[k] = self.v_3[k]/order;
        }

        println!("N:{:?}",N);
        println!("V_3:{:?}",self.v_3);
    }

    // Algorithm 6.13.1
    fn rec_partition(&mut self, n:usize, k:usize, m: usize, B: usize, N: usize, v_2: &mut[usize], T: &[usize]){
        if m==0 {
            let mut c = vec![0; n];
            for i in 1..N {
                c[v_2[i]] += 1;
            }
            let mut prod = 1;
            for i in 1..n {
                prod *= num_integer::binomial(T[i], c[i])
            }
            self.v_3[k] += prod;
        } else {
            for i in 1..cmp::min(B, m){
                v_2[N+1] = i;
                self.rec_partition(n,k,m-i,i,N+1, v_2,&T)
            }
        }
    }

    // Algorithm 6.13.2
    fn norb_use(&mut self, n: usize, g: <G as Group>::Element ){
        let mut g = g;
        let T = g.get_type();
        let mut v_2 = vec![0;n];
        for k in 0..n+1{
            v_2[1] = k;
            self.rec_partition(n,k,k,k,0,&mut v_2, &T)
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

    let mut schreier = Schreier::new(
        PermutationGroup::new(
            vec![
                Permutation::from_cycles(vec![vec![0 ,1 ,3 ,7 ,6,4], vec![2,5]]),
                Permutation::from_cycles(vec![vec![0 ,1 ,3 ,2], vec![4,5,7,6]]),
            ]));

    schreier.simple_gen();

    let stab_chain = schreier.generate_stab_chain();

    println!("Generated group with order {} with the stab chain:", Schreier::get_group_order(&stab_chain));
    schreier.run(&stab_chain);
    schreier.norb(&stab_chain);
}
