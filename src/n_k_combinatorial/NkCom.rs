use crate::data_structs::bitmaps::BM;
use crate::data_structs::{self, consts::*};
use crate::data_structs::graph::Graph;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::schreier::schreier_permutation::{SChooseKInG, Schreier};
use crate::schreier::schreier::{get_S_choose_k_rec, Schreier as GenericSChreier};
use memory_stats::memory_stats;
use nauty_pet::graph::CanonGraph;
use petgraph::visit::Walker;
use std::cmp;
use std::collections::{BTreeMap, HashMap};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::ops::BitAnd;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use base64::Engine;
use base64::engine::general_purpose;
use log::debug;
use crate::data_structs::cert_graph::CertGraph;
use crate::data_structs::invariant_chain::InvariantChain;
use crate::n_k_combinatorial::NkCom;
use crate::data_structs::permutation::Permutation;
use crate::group_actions::permute_edges::PermuteEdges;



#[derive(Clone, Default)]
pub struct SolutionProperties {
    new_vertices: Vec<usize>,
    canon_certificate: String,
    solution_set: Vec<Vec<usize>>,
    solution_certificate: u128,
    ady_certificate: String,
}

pub struct Config {
    pub(crate) leaf_level: usize,
    pub(crate) save_leaves: bool,
}

pub(crate) struct NkComb {
    graph: Graph,
    n: usize,
    k: usize,
    pub seen_vertices: usize,
    pub seen_leaves:usize,
    pub sols: HMap,
    pub sols_lock: Arc<RwLock<HMap>>,
    pub solution_path: Vec<SolutionProperties>,
    pub memory: Vec<BTree>,
    pub trace: Vec<(usize, Vec<usize>)>,
    pub max_memory: f64,
    pub tree: Vec<(usize, usize)>,
    // paralellize
    pub(crate) early_leaves: Arc<RwLock<HashMap<u128,Graph>>>,
    pub(crate) config: Config
}

impl NkComb {
    pub fn new(k: usize) -> NkComb {
        let n = N;
        let ady_mat = vec![BM::new();n];
        let g = Graph {
            ady_matrix: ady_mat,
        };
        Self::from_graph(k, g,
                         Config { leaf_level: n, save_leaves: false },
                         Arc::new(RwLock::new(HMap::new())),
                         Arc::new(RwLock::new(HashMap::new())),
        )
    }

    pub fn from_graph_simple(k: usize, graph: Graph) -> NkComb {
        let n = N;
        Self::from_graph(k, graph,
                         Config { leaf_level: n, save_leaves: false },
                         Arc::new(RwLock::new(HMap::new())),
                         Arc::new(RwLock::new(HashMap::new())),
        )
    }
    pub fn from_graph(k: usize, graph: Graph, config: Config, sols_lock: Arc<RwLock<HMap>>, early_leaves:Arc<RwLock<HashMap<u128, Graph>>>) -> NkComb {
        let n = graph.ady_matrix.len();
        NkComb {
            graph,
            n,
            k,
            seen_vertices: 0,
            seen_leaves: 0,
            sols: HashMap::new(),
            sols_lock,
            solution_path: vec![SolutionProperties::default(); n/2],
            memory: vec![BTreeMap::new(); n + 1],
            trace: vec![],
            tree: vec![(0,0); n + 1],
            max_memory: 0.0,
            config,
            early_leaves: early_leaves,
        }
    }
    
    
    pub fn rec( &mut self, current_line: usize){
        self.seen_vertices += 1;

        // Early stop or leaf test
        let S = self.get_available_points();
        if S.is_empty() || current_line > self.config.leaf_level {
            self.process_leaf(current_line);
            return;
        }

        // Gets the current graph (or solution prefix) automorphism group generators -> G
        let (gen, canon)= NautyGraph::get_aut_generators_and_cert(&self.graph).unwrap();
        let schreier = Schreier::from_generators(gen);

        // Gets current chosen solution stabilizer in G
        let data = schreier.get_orbit(current_line, self.n);
        let schreier = Schreier::from_generators(data.stab_generators);

        // Get all possible solutions to extend graph, non-isomorphic in G
        let mut iter = SChooseKInG::new(schreier.clone(), self.n, &S, self.k);
        let mut prefixes_holder = iter.collect::<Vec<_>>();

        self.tree[current_line].0 += prefixes_holder.len();

        for prefix in prefixes_holder {
            // add solution to graph
            self.add_edges(current_line, &prefix);
            
            if self.is_feasible(current_line, &prefix) {
                //self.gather_info(current_line, &prefix);
                self.rec(current_line + 1);
            }

            // reset inplace graph
            self.remove_edges(current_line, &prefix);
        }
    }

    pub fn rec_over_full_set( &mut self, current_line: usize, remember: bool){
        self.seen_vertices += 1;

        // Early stop or leaf test
        let available_points = self.get_available_points();
        let available_lines = self.get_available_lines();
        let mut S = available_points.clone();
        S.extend(available_lines.clone());

        if S.is_empty() {
            self.process_leaf(current_line);
            return;
        }

        // Gets the current graph (or solution prefix) automorphism group generators -> G
        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(&self.graph).unwrap();
        if remember && self.memory[S.len()].insert(certificate, (gen.clone(),self.graph.clone())).is_some() {
            return;
        }
        let schreier = Schreier::from_generators(gen);

        // Get all the equivalence class representatives in S, according to G
        let non_iso_S = self.get_non_isomorfic_available_points(&schreier,S.as_slice());


        // Set all the iterators for the non-isomorphic point candidates
        let mut s_choose_k_in_g_iterator = vec![];
        for s in non_iso_S.as_slice(){
            // get subgroup of G with s fixed
            let data = schreier.get_orbit(*s, self.n);
            let schreier_for_stab = Schreier::from_generators(data.stab_generators);

            // Get k - |s.neighbors| non-isomorph subset
            let potential_s_neighbors = if s < &(self.n / 2) {
                available_points.as_slice()
            } else {
                available_lines.as_slice()
            };
            s_choose_k_in_g_iterator.push(SChooseKInG::new(schreier_for_stab, self.n, potential_s_neighbors, self.k - self.graph.ady_matrix[*s].len()));
        }

        // Evaluate each iterator once, to know which has the least solutions
        let mut feasible_extenders = vec![vec![]; s_choose_k_in_g_iterator.len()];
        let mut index = 0;
        while let Some(extender_representative) = s_choose_k_in_g_iterator[index].next() {
            let current_vertex =  non_iso_S[index];

            // extend prefix, advance only if it is valid
            self.add_edges(current_vertex, extender_representative.as_slice());
            if !self.is_feasible(current_vertex, extender_representative.as_slice()){
                self.remove_edges(current_vertex, extender_representative.as_slice());
                continue;
            }
            self.remove_edges(current_vertex, extender_representative.as_slice());

            // push feasible extender and advance index
            feasible_extenders[index].push(extender_representative);
            index = (index + 1) % s_choose_k_in_g_iterator.len();
        }

        // Get all possible solutions to extend graph, non-isomorphic in G
        let mut chosen_feasible_extenders = &feasible_extenders[index];
        let current_vertex = non_iso_S[index];

       // self.tree[current_line-1].0 += chosen_feasible_extenders.len();

        for extender in chosen_feasible_extenders {
            // add solution to graph
            self.add_edges(current_vertex, extender);
            self.trace.push((current_vertex, extender.clone()));
            self.rec_over_full_set(current_line + 1, remember);
            // reset inplace graph
            self.remove_edges(current_vertex, extender);
            self.trace.pop();

        }
    }

    fn get_non_isomorfic_available_points(&self, schreier: &Schreier, S_vector: &[usize]) -> Vec<usize> {
        let mut non_isomorphic_S = vec![];
        let mut S = 0;
        for s in S_vector {
            S |= 1usize << s
        }
        for index in 0..self.n {
            // if element i is not on local_available_S, continue
            if (S >> index) & 1 == 0 {
                continue;
            }
            let mut data = schreier.get_orbit(index, self.n);


            // make the elements inside the orbit unavailable
            let mut mask = S;
            for orbit_element in &data.orbit {
                mask ^= 1 << *orbit_element  // orbit union S / (orbit intersection S)
            }
            S &= mask; // local_available_S intersection S not in orbit

            //Add the orbit representative to current_prefix (it is always current index)
            non_isomorphic_S.push(index); // current p
        }
        non_isomorphic_S
    }


    pub fn rec_with_memory( &mut self, current_line: usize){
        self.seen_vertices += 1;

        let S = self.get_available_points();
        if S.is_empty() || current_line > self.config.leaf_level {
            self.process_leaf(current_line);
            return;
        }

        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(&self.graph).unwrap();
        if self.memory[current_line].insert(certificate, (gen.clone(),self.graph.clone())).is_some() {
            return;
        }
        
        let schreier = Schreier::from_generators(gen);
        let mut prefixes_holder = Schreier::get_S_choose_k_in_G(schreier.clone(), self.n, &S, self.k);

        // println!("Subsets: {:?}", prefixes_holder);

        // debug!("prefixes for line {current_line}: \n{:?}", prefixes_holder);

        self.tree[current_line].0 += prefixes_holder.len();

        for prefix in prefixes_holder {
            self.add_edges(current_line, &prefix);

            if self.is_feasible(current_line, &prefix) {
                self.rec_with_memory(current_line + 1);
            }

            self.remove_edges(current_line, &prefix);
        }
    }


    pub fn rec_over_full_set_with_memory( &mut self, current_line: usize){
        self.seen_vertices += 1;

        // Early stop or leaf test
        let available_points = self.get_available_points();
        let available_lines = self.get_available_lines();
        let mut S = available_points.clone();
        S.extend(available_lines.clone());

        if S.is_empty()  || current_line > self.config.leaf_level {
            self.process_leaf(current_line);
            return;
        }



        // Gets the current graph (or solution prefix) automorphism group generators -> G
        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(&self.graph).unwrap();
        if self.memory[S.len()].insert(certificate, (gen.clone(),self.graph.clone())).is_some() {
            return;
        }

        let schreier = Schreier::from_generators(gen);

        // Set all the iterators for the non-isomorphic point candidates
        let non_iso_S = self.get_non_isomorfic_available_points(&schreier,S.as_slice());
        let mut iters = vec![];
        for s in non_iso_S.as_slice(){
            let data = schreier.get_orbit(*s, self.n);
            let schreier_for_stab = Schreier::from_generators(data.stab_generators);
            if s < &(self.n / 2) {
                iters.push(SChooseKInG::new(schreier_for_stab, self.n, available_points.as_slice(), self.k - self.graph.ady_matrix[*s].len()));
            } else {
                iters.push(SChooseKInG::new(schreier_for_stab, self.n, available_lines.as_slice(), self.k - self.graph.ady_matrix[*s].len()));
            }
        }

        let mut prefixes_holders = vec![vec![]; iters.len()];
        let mut index = 0;


        while let Some(prefix) = iters[index].next() {
            let current_vertex =  non_iso_S[index];
            self.add_edges(current_vertex, prefix.as_slice());

            if !self.is_feasible(current_vertex, prefix.as_slice()){
                self.remove_edges(current_vertex, prefix.as_slice());
                continue;
            }
            self.remove_edges(current_vertex, prefix.as_slice());

            prefixes_holders[index].push(prefix);
            index = (index + 1) % iters.len();
        }


        // Get all possible solutions to extend graph, non-isomorphic in G
        let mut prefixes_holder = &prefixes_holders[index];
        let current_vertex = non_iso_S[index];

        // println!("prefixes for vertex {}: {}", current_vertex, prefixes_holder.len());

        self.tree[current_line].0 += prefixes_holder.len();


        for prefix in prefixes_holder {
            // add solution to graph
            self.add_edges(current_vertex, prefix);

            if self.is_feasible(current_vertex, prefix) {
                //self.gather_info(current_line, &prefix);
                self.rec_over_full_set_with_memory(current_line + 1);
            }

            // reset inplace graph
            self.remove_edges(current_vertex, prefix);
        }
    }


    pub fn add_vertex_to_prefix(&mut self, mut cert: u128, mut gen: Vec<Permutation>, graph: Graph, next_vertex: usize, current_list: &mut BTree, solutions: &mut BTree ) {
        // Early stop or leaf test
        self.graph = graph;
        let available_points = self.get_available_points();
        let available_lines = self.get_available_lines();
        let mut S = available_points.clone();
        S.extend(available_lines.clone());

        if S.is_empty(){
            solutions.insert(cert,(vec![], self.graph.clone()));
            return;
        }

        // Gets the current graph (or solution prefix) automorphism group generators -> G
        if gen.is_empty(){
            (gen, cert)= NautyGraph::get_aut_generators_and_cert(&self.graph).unwrap();
        }

        let schreier = Schreier::from_generators(gen);

        // Set all the iterators for the non-isomorphic point candidates
        let non_iso_S = self.get_non_isomorfic_available_points(&schreier,S.as_slice());
        let mut iters = vec![];
        for s in non_iso_S.as_slice(){
            let data = schreier.get_orbit(*s, self.n);
            let schreier_for_stab = Schreier::from_generators(data.stab_generators);
            if s < &(self.n / 2) {
                iters.push(SChooseKInG::new(schreier_for_stab, self.n, available_points.as_slice(), self.k - self.graph.ady_matrix[*s].len()));
            } else {
                iters.push(SChooseKInG::new(schreier_for_stab, self.n, available_lines.as_slice(), self.k - self.graph.ady_matrix[*s].len()));
            }
        }

        let mut prefixes_holders = vec![vec![]; iters.len()];
        let mut index = 0;


        while let Some(prefix) = iters[index].next() {
            let current_vertex =  non_iso_S[index];
            self.add_edges(current_vertex, prefix.as_slice());

            if !self.is_feasible(current_vertex, prefix.as_slice()){
                self.remove_edges(current_vertex, prefix.as_slice());
                continue;
            }
            self.remove_edges(current_vertex, prefix.as_slice());

            prefixes_holders[index].push(prefix);
            index = (index + 1) % iters.len();
        }


        // Get all possible solutions to extend graph, non-isomorphic in G
        let mut prefixes_holder = &prefixes_holders[index];
        let current_vertex = non_iso_S[index];

        // Calculate solution and certs, save them in memory
        for prefix in prefixes_holder {
            // add solution to graph
            self.add_edges(current_vertex, prefix);

            let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(&self.graph).unwrap();
            current_list.insert(certificate, (gen,self.graph.clone()));

            // reset inplace graph
            self.remove_edges(current_vertex, prefix);
        }
    }


    // gets all the points which still have room for neighbors (points are second half)
    fn get_available_points(&self) -> Vec<usize> {
        Self::get_available_points_stat(&self.graph, self.k)
    }
    // gets all the points which still have room for neighbors
    fn get_available_points_stat(graph: &Graph, k: usize) -> Vec<usize> {
        let mut available_points = vec![];
        let n = graph.ady_matrix.len();
         for (i,neighbors) in graph.ady_matrix.iter().enumerate().skip(n/2) {
             if neighbors.len() < k {
                 available_points.push(i);
             }
         }
        available_points
    }

    // gets all the points which still have room for neighbors (lines are first half)
    fn get_available_lines(&self) -> Vec<usize> {
        Self::get_available_lines_stat(&self.graph, self.k)
    }

    // gets all the points which still have room for neighbors
    fn get_available_lines_stat(graph: &Graph, k: usize) -> Vec<usize> {
        let mut available_points_and_lines = vec![];
        let n = graph.ady_matrix.len();
        for (i,neighbors) in graph.ady_matrix.iter().enumerate().take(n/2) {
            if neighbors.len() < k {
                available_points_and_lines.push(i);
            }
        }
        available_points_and_lines
    }

    fn add_edges(&mut self, origin:usize, ends: &[usize]) {
        for end in ends {
            self.graph.add_edge(origin, *end);
        }
    }
    fn remove_edges(&mut self, origin:usize, ends: &[usize]){
        for end in ends {
            self.graph.remove_edge(origin, *end);
        }
    }
    fn is_feasible(&self, origin:usize, ends:&[usize]) -> bool {
        let ends = self.graph.ady_matrix[origin].clone().into_iter().collect::<Vec<usize>>();
        for i in (0..ends.len()-1){
            let mut neighborhood_1 = self.graph.ady_matrix[ends[i]] ;
            neighborhood_1.set(origin,false);
            for j in i+1..ends.len(){
                if !neighborhood_1.bitand(self.graph.ady_matrix[ends[j]]).is_empty() {
                    return false;
                }
            }
        }
        true
    }

    fn is_feasible_stat(graph: &Graph, origin:usize, ends:&[usize]) -> bool {
        let ends = graph.ady_matrix[origin].clone().into_iter().collect::<Vec<usize>>();
        for i in (0..ends.len()-1){
            let mut neighborhood_1 = graph.ady_matrix[ends[i]] ;
            neighborhood_1.set(origin,false);
            for j in i+1..ends.len(){
                if !neighborhood_1.bitand(graph.ady_matrix[ends[j]]).is_empty() {
                    return false;
                }
            }
        }
        true
    }

    fn get_possible_children(&mut self) -> Vec<Graph>{
        let available_points = self.get_available_points();
        let available_lines = self.get_available_lines();
        let mut possible_children = vec![];
        for point in &available_points {
            for line in &available_lines {
                if self.graph.has_edge(*point,*line){
                    continue;
                }
                let ends =[*line];
                self.add_edges(*point, ends.as_slice());
                if self.is_feasible(*point, ends.as_slice()){
                    possible_children.push(self.graph.clone());
                }
                self.remove_edges(*point, ends.as_slice());
            }
        }
        possible_children
    }

    fn get_possible_edges(graph: &mut Graph, k: usize ) -> Vec<(usize, usize)>{
        let available_points = Self::get_available_points_stat(graph,k);
        let available_lines = Self::get_available_lines_stat(graph,k);
        let mut possible_edges = vec![];
        for point in &available_points {
            for line in &available_lines {
                if graph.has_edge(*point,*line){
                    continue;
                }
                graph.add_edge(*point, *line);
                if Self::is_feasible_stat(&graph, *point, [*line].as_slice()){
                    possible_edges.push(( *line, *point ));
                }
                graph.remove_edge(*point, *line);
            }
        }
        possible_edges
    }

    fn get_subgraphs(graph: &Graph) -> Vec<Graph> {
        let n  = graph.ady_matrix.len();
        let mut subobjects = Vec::with_capacity(n);
        for i in 0..n - 1 {
            for j in i + 1 ..n {
                if graph.has_edge(i, j){
                    let mut subobject = graph.clone();
                    subobject.remove_edge(i, j);
                    subobjects.push(subobject);
                }
            }
        }
        subobjects
    }

    fn is_isomorph_to_canonical(graph: &Graph, subgraphs:  &[Graph] ) -> (bool, bool) {
        let chain = InvariantChain::new(0);
        let (used_cert, canonical_index) = chain.get_graphs_isomorph_to_canonical(subgraphs);

        let mut graph_index= None;
        for (index, subgraph) in subgraphs.iter().enumerate() {
            if subgraph == graph {
                graph_index = Some(index);
                break
            }
        }
        debug!("Canonical idexes: {:?}", canonical_index);
        let result = canonical_index.contains(&graph_index.unwrap());
        if result{
            debug!("Found that p(X) is isomorph to m(X) with index {}",graph_index.unwrap())
        } else {
            debug!("p(X) is not isomorph to m(X) index {} NOT FOUND",graph_index.unwrap());
        }
        (result, used_cert)
    }



    fn process_leaf(&mut self, current_line: usize){
        self.seen_leaves += 1;
        //self.tree[current_line-2].1 += 1;
        if let Some(usage) = memory_stats() {
            let memory = usage.physical_mem as f64 / (1024.0 * 1024.0);
            if self.max_memory < memory {
                self.max_memory = memory;
            }
        }

        let (gen, canon)= NautyGraph::get_aut_generators_and_cert(&self.graph).unwrap();

        println!("\ncanon:{canon}:");
        /*
        println!("[");
        for element in &self.trace {
            println!("\t{:?}",element);
        }
        println!("]");
         */

        println!("Graph:\n[");
        for i in 0..self.graph.ady_matrix.len() {
            print!("\t{i:02},\t[");
            for j in &self.graph.ady_matrix[i]{
                print!("{j:02}, ");
            }
            println!("]");
        }
        println!("]");

        if self.sols_lock.write().unwrap().insert( canon, true).is_none() {
            if self.config.save_leaves{
                self.early_leaves.write().unwrap().insert(canon.clone(), self.graph.clone());
            }
            return;
            println!("{} {canon} => bytes:{} graph: {}",
                     self.early_leaves.write().unwrap().len(),
                     general_purpose::STANDARD.encode( self.graph.to_bytes()),
                     self.graph
            );
        };
        
        
        /*
        if self.print > 0 {
            self.print-=1;
            println!("{:?}", Dot::new(&self.graph.to_pet_graph()));
        }
        
         */


        
        return;
        /*
        println!("Solution path : ");
        for i in &self.solution_path{
            println!("{:?} {:?} {:?} ", i.solution_set, i.canon_certificate, i.ady_certificate);
        }
        println!("\n");


         */
        
    }
    fn gather_info(&mut self, current_line:usize, prefix: &[usize]) {
        let mut current_sol_number = 0u128;
        let mut current_sol = vec![vec![]; self.k+1];
        for (i,v) in self.graph.ady_matrix.iter().enumerate(){
            current_sol[self.k - v.len()].push(i);
            current_sol_number |= 1 << (i + v.len()*N);
        }

        self.solution_path[current_line]= SolutionProperties {
            new_vertices: prefix.to_vec(),
            canon_certificate: self.get_canon_certificate(),
            solution_set: current_sol,
            solution_certificate: current_sol_number,
            ady_certificate: self.get_ady_certificate(),
        };
    }

    pub fn get_canon_certificate(&self) -> String{
        let canon = CanonGraph::from(self.graph.to_pet_levy_graph());
        let mut hasher = DefaultHasher::new();
        canon.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn get_ady_certificate(&self) -> String{
        let mut certificate = 0;
        for (count, i) in (N_CONF..N).enumerate() {
            certificate |= (*self.graph.ady_matrix[i].as_value() as u128 ) << (N_CONF*count);
        }
        certificate.to_string()
    }
}




#[test]
fn test_n_comb() {

    /*
    let k = 3;
    let mut nkComb = NkComb::new(k);
    nkComb.config = Config{leaf_level: N, save_leaves: true};
    let now = Instant::now();
    nkComb.rec(0);
    let duration = now.elapsed();
    println!("\n{N_CONF}_{k}  Without memory:");
    println!("vertices: {}\nleaves: {}\nsols: {}", nkComb.seen_vertices, nkComb.seen_leaves, nkComb.early_leaves.read().unwrap().len());
    println!("Time elapsed: {:?}", duration);
    println!("TREE: \n");
    for (i, (extensions, sols)) in nkComb.tree.iter().enumerate() {
        println!("\t{}\t{}\t{}", i, extensions,sols);
    }

     */


    let k = 3;
    let mut nkComb = NkComb::new(k);
    nkComb.config = Config{leaf_level: N, save_leaves: true};
    let now = Instant::now();
    nkComb.rec_over_full_set(N_CONF, true);
    let duration = now.elapsed();
    println!("\n{N_CONF}_{k}  Without memory:");
    println!("vertices: {}\nleaves: {}\nsols: {}", nkComb.seen_vertices, nkComb.seen_leaves, nkComb.early_leaves.read().unwrap().len());
    println!("Time elapsed: {:?}", duration);
    /*
    println!("TREE: \n");
    for (i, (extensions, sols)) in nkComb.tree.iter().enumerate() {
        println!("\t{}\t{}\t{}", i, extensions,sols);
    }
    
     */

    return;

    // 18 446 741 950 994 212 328
    let mut nkComb = NkComb::new(k);
    nkComb.config = Config{leaf_level: N, save_leaves: true};
    let now = Instant::now();
    nkComb.rec_over_full_set(N_CONF, true);
    let duration = now.elapsed();
    println!("\n{N_CONF}_{k}  With memory:");
    println!("vertices: {}\nleaves: {}\nsols: {}", nkComb.seen_vertices, nkComb.seen_leaves, nkComb.early_leaves.read().unwrap().len());
    println!("Time elapsed: {:?}", duration);
    println!("TREE: \n");
    for (i, (extensions, sols)) in nkComb.tree.iter().enumerate() {
        println!("\t{}\t{}\t{}", i, extensions,sols);
    }
}

#[test]
fn test_get_children(){
    let k = 3;
    let mut nkComb = NkComb::new(k);
    let graphs = nkComb.get_possible_children();

    let mut children_certs = HashMap::new();
    for graph in graphs {
        println!("{}", graph.custom_print());
        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(&graph).unwrap();
        children_certs.insert(certificate,graph);
    }

    println!("{:?}", children_certs);

}



fn print_nested_optional_vec(data: &Vec<Vec<Option<(usize, usize)>>>) {
    println!("--- Nested Optional Vector Contents (Aligned) ---");

    // Define a fixed width for each data cell (10 characters)
    const CELL_WIDTH: usize = 10;

    // 1. Determine maximum number of columns to align the header
    let max_cols = data.iter().map(|row| row.len()).max().unwrap_or(0);

    // --- Print Header Row ---
    print!("{:<10}", "Index:");

    // Print the column indices (0, 1, 2, ...) centered within the fixed width
    for j in 0..max_cols {
        print!("{:^width$}", j, width = CELL_WIDTH);
    }
    println!();

    // Print a separator line based on the total width
    println!("{}", "-".repeat(10 + max_cols * CELL_WIDTH));

    // --- Print Data Rows ---
    for (i, row) in data.iter().enumerate() {
        // Print the row index label
        print!("{:<10}", format!("Row {}:", i));

        // Print row elements with fixed width
        for item in row.iter() {
            let cell_content = match item {
                // If the option is Some((r, c)), format the tuple
                Some((r, c)) => format!("({},{})", r, c),
                // If the option is None, use a placeholder
                None => " -- ".to_string(),
            };

            // Print the content centered within the fixed width
            print!("{:^width$}", cell_content, width = CELL_WIDTH);
        }

        // Pad the line if this row is shorter than max_cols
        if row.len() < max_cols {
            for _ in 0..(max_cols - row.len()) {
                print!("{:^width$}", " ", width = CELL_WIDTH);
            }
        }

        println!();
    }

    println!("----------------------------------------------");
}