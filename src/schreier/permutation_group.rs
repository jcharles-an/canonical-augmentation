use std::cmp;
use std::collections::HashSet;
use crate::data_structs::stabilizer_chain::StabChain;
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;
use crate::schreier::schreier::Schreier;

#[derive(Debug, Clone)]
pub struct PermutationGroup
{
    generators: Vec<Permutation>,
    stab_chain: StabChain<Permutation>,
    identity: Permutation,
    v_3: Vec<usize>,
    max_n: usize,
    pub max_depth: usize,
}

impl Group for PermutationGroup{
    type Element = Permutation;
    type Target = usize;

    fn target_to_usize(target: Self::Target) -> usize {
        target
    }


    /// Operation of permutation composition, applying first element b, then element a. a ° b of a (b( x ))
    fn operate(element_a: &Permutation, element_b: &Permutation) -> Permutation {
        let mut result;
        if element_a.size > element_b.size{
            result = element_a.map.clone()
        } else {
            result = element_b.map.clone()
        }
        for (index, value) in element_b.map.iter().enumerate(){
            result[index] = Self::apply_to(element_a, value);
        }
        Permutation::new(result)
    }

    fn get_inverse(element: &Permutation) -> Permutation {
        let mut result = vec![0;element.size];
        for (index, value) in element.map.iter().enumerate(){
            result[*value] = index
        }
        Permutation::new(result)
    }

    fn apply_to(element: &Permutation, target: &Self::Target) -> Self::Target {
        element.map.get( *target ).map(|&x| x).unwrap_or(*target)
    }

    fn is_identity(element: &Permutation) -> bool {
        element.map.iter().enumerate().all(|(index, &value)| index == value)
    }

    fn get_identity(&self) -> Permutation {
        Permutation::new((0..self.max_n).collect())
    }

    fn get_identity_ref(&self) -> &Permutation {
        &self.identity
    }

    fn get_identity_length(&self) -> usize {
        self.max_n
    }

    fn get_generators(&self) -> &Vec<Permutation> {
        &self.generators
    }
    fn get_generators_as_mut(&mut self) -> &mut Vec<Permutation> {
        &mut self.generators
    }

    fn get_stab_chain(&self) -> &StabChain<Permutation> {
        &self.stab_chain
    }

    fn set_stab_chain(&mut self, stab_chain: StabChain<Permutation>) {
        self.stab_chain = stab_chain
    }

    fn get_range(&self) -> usize {
        self.max_n
    }

    fn add_generator(&mut self, new_generator: Permutation) {
        self.max_n = new_generator.size;
        self.generators.push(new_generator);
        self.identity = Permutation::new((0..self.max_n).collect());
    }


    fn new_empty() -> PermutationGroup {
        PermutationGroup::new( vec![])
    }

    /*
    fn generate_group(&self) {//ALGORITHM 6.6 RUN(n, G, USE())


    }

    fn generate_by_backtrack(){

    }

    fn process_group_element(&self){

    }

     */
}


impl PermutationGroup {
    pub fn new(generators: Vec<Permutation>) -> PermutationGroup{
        let max_n = generators.iter().map(|generator| generator.size).max().unwrap_or(0);
        let identity = Permutation::new((0..max_n).collect());
        PermutationGroup {
            generators,
            max_n,
            identity: identity.clone(),
            stab_chain: StabChain::new(max_n, identity.clone(),identity.clone()),
            v_3: vec![],
            max_depth: 0,
        }
    }

    pub fn new_with_stab_chain(generators: Vec<Permutation>, stab_chain: StabChain<Permutation>) -> PermutationGroup{
        let max_n = generators.iter().map(|generator| generator.size).max().unwrap_or(0);
        let identity = Permutation::new((0..max_n).collect());
        PermutationGroup {
            generators,
            max_n,
            identity: identity.clone(),
            stab_chain,
            v_3: vec![],
            max_depth: 0,
        }
    }

    //BOOK IMPLEMENTATIONS

    // Algorithm 6.5.1
    //Naïve algorithm to generate a full group from their generators
    pub fn simple_generate_group(&self){
        let mut counter = 0;
        let mut full_group: HashSet<Permutation> = HashSet::new();
        let mut new = vec![self.get_identity()];
        while !new.is_empty(){
            let last = new.clone();
            while let Some(element) = new.pop(){
                full_group.insert(element);
            }
            for g in self.get_generators(){
                for h in &last{
                    let f = Self::operate(g, h);
                    if !full_group.contains(&f){
                        new.push(f);
                        counter += 1;
                    }
                }
            }
        }
        println!("{} full_group:{:?}", full_group.len() ,full_group);
    }

    // Algorithm 6.6 This algorithm creates the group from the shcreier sims representation
    pub fn run( &mut self ){
        self.run_backtrack(0,self.get_identity(),Self::print_result)
    }

    // Algorithm 6.6.1
    //process-leaf might be an attribute in the struct
    fn run_backtrack(&mut self, l: usize, g:Permutation, process_leaf: fn(&mut Self, g:Permutation) ){
        let mut stab_chain = self.get_stab_chain();
        let n = self.get_range();
        if l == n{
            process_leaf(self, g)
        } else {
            for h in &stab_chain.chain[l].clone(){
                let f = Self::operate(&g, h);
                self.run_backtrack(l + 1, f, process_leaf)
            }
        }
    }

    // Algorithm 6.6.2
    // Simple callback for when finding leave
    fn print_result(&mut self, g:Permutation ){
        println!("{g:?}");
    }


    //algorithm 6.8
    fn test_pertenece(&self, g: &mut Permutation) -> usize{
        let n = self.get_range();
        let stab_chain = self.get_stab_chain();
        for i in 0..n{
            let x = Self::apply_to(g, &i);
            let mut found_h = None;
            // is there any h in subgroups of stab_chain that goes to x?
            for h in &stab_chain.chain[i]{
                if Self::apply_to(h, &i) == x {
                    found_h = Some(h);
                    break;
                }
            }
            if let Some(h) = found_h {
                let h_inv = Self::get_inverse(h);
                let product =  Self::operate(&h_inv, g);
                *g = product;
            } else {
                return i;
            }
        }
        n
    }

    // Algorithm 6.9
    fn generate_stab_chain(&mut self) -> StabChain<Permutation> {
        let n = self.get_range();
        let mut stab_chain = StabChain::new(n,self.get_identity(),self.get_identity());
        self.max_depth = 0;
        for generator in self.generators.clone().into_iter() {
            let mut generator = generator;
            self.enter(&mut generator,0);
        }

        println!("Stab_chain is:");
        for (index, stab) in stab_chain.chain.iter().enumerate() {
            println!("{index} : {:?}", stab)
        }

        stab_chain
    }

    // Algorithm 6.9.1
    fn enter(&mut self, generator: &mut Permutation, tier:usize){
        let mut stab_chain = &mut self.generate_stab_chain();
        //Self::print_format(tier);println!("testing: {generator:?}");
        let i = self.test_pertenece( generator );
        //Self::print_format(tier);println!("got: {generator:?} with index {i}");
        if i == self.max_n {
            //println!();
            return;
        }
        //update max depth if needed
        self.max_depth = cmp::max(self.max_depth, i);
        //push multiplied permutation into group i
        stab_chain.chain[i].push(generator.clone());
        //Self::print_format(tier);println!("Introducing in stabchain: {{");
        for (index, stab) in stab_chain.chain.iter().enumerate(){
            //Self::print_format(tier);println!("\t{index}: {:?}",stab);
        }
        //Self::print_format(tier);println!("}}");

        let generator = &(*generator);
        let current_max_depth = self.max_depth;
        for j in 0..current_max_depth{
            let mut k = 0;
            let len = stab_chain.chain[j].len();
            while k < len{
                let h = &stab_chain.chain[j][k];
                if Self::is_identity(h) {
                    k+=1;
                    continue;
                }
                let mut product = Self::operate(generator, h);
                //Self::print_format(tier);println!("{generator:?} * {h:?} = {product:?}");
                self.enter(&mut product, tier + 1);
                k += 1;
            }
        }
        //println!();

    }

    //Algorithm 6.10
    pub fn test_pertenece_2(&self, g: &mut Permutation) -> usize{
        let n = self.max_n;
        let stab_chain = self.get_stab_chain();
        for i in 0..n{
            let permuted_i = (Self::apply_to(&stab_chain.base, &i));
            let x = Self::apply_to(g, &permuted_i);
            let mut found_h = None;
            // is there any h in subgroups of stab_chain that goes to x?
            for h in &stab_chain.chain[i]{
                if Self::apply_to(h, &permuted_i) == x {
                    found_h = Some(h);
                    break;
                }
            }
            if let Some(h) = found_h {
                let h_inv = Self::get_inverse(h);
                let product =  Self::operate(&h_inv, g);
                *g = product;
            } else {
                return i;
            }
        }
        n
    }

    // Algorithm 6.11
    pub fn enter_2(&mut self, generator: &mut Permutation) {

        let i = self.test_pertenece_2(generator);
        if i == self.max_n {
            return;
        }
        //update max depth if needed
        self.max_depth = cmp::max(self.max_depth, i);
        //push multiplied permutation into group i
        self.stab_chain.chain[i].push(generator.clone());

        let generator = &(*generator);
        let current_max_depth = self.max_depth;
        for j in 0..current_max_depth{
            let mut k = 0;
            let len = self.stab_chain.chain[j].len();
            while k < len{
                let h = &self.stab_chain.chain[j][k];
                if Self::is_identity(h) {
                    k+=1;
                    continue;
                }
                let mut product = Self::operate(generator, h);
                //Self::print_format(tier);println!("{generator:?} * {h:?} = {product:?}");
                self.enter_2(&mut product);
                k += 1;
            }
        }
        //println!();

    }

    pub fn get_group_order(&self) -> usize{
        self.get_stab_chain().chain.iter().map(|x| x.len()).product()
    }

    fn print_format(tier:usize){
        for i in 0..tier{
            print!("\t");
        }
    }

    //Algorithm 6.12
    pub fn change_base(&mut self, new_base: Permutation) {
        let n = self.max_n;
        let mut stab_chain = self.get_stab_chain().clone();
        let mut new_stab_chain = StabChain::new(n, self.get_identity(), new_base );
        self.set_stab_chain(new_stab_chain);
        for j in 0..n{
            for g in stab_chain.chain[j].iter_mut() {
                self.enter_2( g );
            }
        }
    }



    // Algorithm 6.13.0
    fn norb(&mut self){
        let n = self.get_range();
        self.v_3 = vec![0; n];
        self.run_backtrack(0,self.get_identity(), Self::norb_use);

        let mut V = vec![0; n];
        let order = self.get_group_order();
        for k in 0..n {
            V[k] = self.v_3[k]/order;
        }

        println!("N:{:?}", V);
        println!("V_3:{:?}",self.v_3);
    }


    // Algorithm 6.13.1
    fn rec_partition(&mut self, n:usize, k:usize, m: usize, B: usize, N_var: usize, v_2: &mut[usize], T: &[usize]){
        if m==0 {
            let mut c = vec![0; n];
            for i in 1..N_var {
                c[v_2[i]] += 1;
            }
            let mut prod = 1;
            for i in 1..n {
                prod *= num_integer::binomial(T[i], c[i])
            }
            self.v_3[k] += prod;
        } else {
            for i in 1..cmp::min(B, m){
                v_2[N_var +1] = i;
                self.rec_partition(n, k, m-i, i, N_var +1, v_2, &T)
            }
        }
    }

    // Algorithm 6.13.2
    fn norb_use(&mut self, g: Permutation ){
        let n = self.get_range();
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
fn test_schreir() {
    /*
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

     */
}


#[test]
fn perm_group_test() {
    let permutation_group = PermutationGroup::new(
        vec![Permutation::new(vec![0,2,3,5,4,1])]
    );

    let mut permutation = Permutation::new(vec![0,2,3,5,4,1]);
    assert_eq!(PermutationGroup::get_inverse(&permutation).map, vec![0,5,1,2,4,3]);
    assert_eq!(PermutationGroup::apply_to(&permutation, &0),0);
    assert_eq!(PermutationGroup::apply_to(&permutation,&1),2);
    assert_eq!(PermutationGroup::apply_to(&permutation,&2), 3);

    permutation = Permutation::new(vec![2,0,4,3,1]);
    let permutation2 = Permutation::new(vec![5,1,3,0,4,2]);
    assert_eq!(PermutationGroup::operate(&permutation, &permutation2).map, vec![5,0,3,2,1,4]);

    let permutation2 = Permutation::new(vec![0,2,1]);
    assert_eq!(PermutationGroup::operate(&permutation, &permutation2).map, vec![2,4,0,3,1]);
    assert_eq!(PermutationGroup::operate(&permutation2, &permutation).map, vec![1,0,4,3,2]);

    let permutation = Permutation::new(vec![1,3,0,2,4]);
    let permutation2 = Permutation::new(vec![4,3,2,1,0]);
    assert_eq!(PermutationGroup::operate(&permutation2, &permutation).map, vec![3,1,4,2,0]);



    assert_eq!(permutation_group.get_identity().map, vec![0, 1, 2, 3, 4, 5]);



}