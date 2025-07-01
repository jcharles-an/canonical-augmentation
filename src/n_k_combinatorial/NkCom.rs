use rayon::iter::ParallelIterator;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{BitAnd, BitOr, Deref};
use std::path::Prefix;
use std::process;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use libc::confstr;
use log::{debug, trace};
use memory_stats::memory_stats;
use nauty_pet::graph::CanonGraph;
use nauty_Traces_sys::graph;
use petgraph::dot::Dot;
use petgraph::visit::Walker;
use rayon::iter::IntoParallelIterator;
use crate::data_structs::bitmaps::{BMConf, BM};
use crate::data_structs::consts::*;
use crate::data_structs::graph::Graph;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::n_k_combinatorial::n_k_combinatorial::CanonOption::Nauty;
use crate::n_k_combinatorial::n_k_combinatorial::NkCombinatorial;
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;
use crate::schreier::schreier_permutation::Schreier;

#[derive(Clone, Default)]
pub struct SolutionProperties {
    new_vertices: Vec<usize>,
    canon_certificate: String,
    solution_set: Vec<Vec<usize>>,
    solution_certificate: u128,
    ady_certificate: String,
}

pub struct Config {
    leaf_level: usize,
    save_leaves: bool,
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
    pub memory: Vec<HMap>,
    pub max_memory: f64,
    // paralellize
    early_leaves: Arc<RwLock<HashMap<String,Graph>>>,
    config: Config
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

    pub fn from_graph(k: usize, graph: Graph, config: Config, sols_lock: Arc<RwLock<HMap>>, early_leaves:Arc<RwLock<HashMap<String, Graph>>>) -> NkComb {
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
            memory: vec![HashMap::new(); n/2],
            max_memory: 0.0,
            config,
            early_leaves: early_leaves,
        }
    }
    
    
    pub fn rec( &mut self, current_line: usize){
        self.seen_vertices += 1;
        
        let S = self.get_available_points();

        if S.is_empty() || current_line > self.config.leaf_level {
            self.process_leaf();
            return;
        }


        let (gen, canon)= NautyGraph::get_aut_generators_and_cert(&self.graph).unwrap();
        // println!("{:?}", gen);

        let schreier = Schreier::from_generators(gen);
        let data = schreier.get_orbit(current_line, self.n);
        let schreier = Schreier::from_generators(data.stab_generators);
        
        let mut prefixes_holder = Schreier::get_S_choose_k_in_G(schreier.clone(), self.n, &S, self.k);

       // println!("Subsets: {:?}", prefixes_holder);

        for prefix in prefixes_holder {
            self.add_edges(current_line, &prefix);
            
            if self.is_feasible(current_line, &prefix) {
                //self.gather_info(current_line, &prefix);
                self.rec(current_line + 1);
            }

            self.remove_edges(current_line, &prefix);
        }
    }


    pub fn rec_with_memory( &mut self, current_line: usize){
        self.seen_vertices += 1;

        /*
        if self.seen_vertices % 100000 == 0 {
            println!("seen_vertices: {}", self.seen_vertices);
        }
        
         */


        let S = self.get_available_points();
        if S.is_empty() || current_line > self.config.leaf_level {
            self.process_leaf();
            return;
        }


        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(&self.graph).unwrap();
        if self.memory[current_line].insert(certificate, true).is_some() {
            return;
        }
        
        let schreier = Schreier::from_generators(gen);
        let mut prefixes_holder = Schreier::get_S_choose_k_in_G(schreier.clone(), self.n, &S, self.k);

        // println!("Subsets: {:?}", prefixes_holder);

        debug!("prefixes for line {current_line}: \n{:?}", prefixes_holder);

        /*
        for prefix in &prefixes_holder {
            self.add_edges(current_line, prefix);

            if self.is_feasible(current_line, prefix) {
                //if self.print > 0 {
                //    self.is_feasible(current_line, &prefix);

                // 4. Print the result
                //     println!("{}  is feasible\n", self.graph.custom_print());
                // }
                debug!("feasible line {current_line}: {:?}", prefix);
            }

            self.remove_edges(current_line, prefix);
        }
        
         */


        for prefix in prefixes_holder {
            self.add_edges(current_line, &prefix);

            if self.is_feasible(current_line, &prefix) {
                self.rec_with_memory(current_line + 1);
            }

            self.remove_edges(current_line, &prefix);
        }
    }

    fn get_available_points(&self) -> Vec<usize> {
        let mut available_points = vec![];
         for (i,neighbors) in self.graph.ady_matrix.iter().enumerate().skip(self.n/2) {
             if neighbors.len() < self.k {
                 available_points.push(i);
             }
         }
        available_points
    }

    fn add_edges(&mut self,line:usize, points: &[usize]) {
        for point in points{
            self.graph.add_edge(line,*point);
        }
    }
    fn remove_edges(&mut self,line:usize, points: &[usize]){
        for point in points{
            self.graph.remove_edge(line,*point);
        }
    }
    fn is_feasible(&self, line:usize, points:&[usize]) -> bool {

        let mask = BM::mask(line);

        for i in (0..points.len()-1){
            let neighborhood_1 = self.graph.ady_matrix[points[i]] & mask;
            for j in i+1..points.len(){
                if !neighborhood_1.bitand(self.graph.ady_matrix[points[j]]).is_empty() {
                    return false;
                }
            }
        }
        true
    }
    fn process_leaf(&mut self){
        self.seen_leaves += 1;
        if let Some(usage) = memory_stats() {
            let memory = usage.physical_mem as f64 / (1024.0 * 1024.0);
            if self.max_memory < memory {
                self.max_memory = memory;
            }
        }

        let canon = self.get_canon_certificate();
        if self.sols_lock.write().unwrap().insert( canon.clone(), true).is_none() {
            if self.config.save_leaves{
                self.early_leaves.write().unwrap().insert(canon.clone(), self.graph.clone());
            } else {
                //println!("{} {canon}", self.sols.len());
            }
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


pub fn launch_parallel_p(k: usize, leaf_level: usize, leaf_level_2: usize, print: bool) -> Duration {
    let now = Instant::now();
    let mut n_k_combinatorial = NkComb::new(k);
    n_k_combinatorial.config = Config{
        leaf_level,
        save_leaves: true,
    };
    n_k_combinatorial.rec_with_memory(0);
    let hm = Arc::new(RwLock::new(HashMap::new()));



    let early_leaves = n_k_combinatorial.early_leaves.read().unwrap().clone();
    let total_threads = early_leaves.len();
    let mut threads_done = Arc::new(Mutex::new(0));
    println!("Task to parallelize:{}", total_threads);
    //println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    early_leaves.into_par_iter().for_each(|(canon, leaf)|
        {
            // println!("Parallel {i} deployed!");
            let now = Instant::now();
            let mut n_k_combinatorial =  NkComb::from_graph(k, 
                                                            leaf.clone(),
                                                            Config{leaf_level: leaf_level_2, save_leaves: true},  
                                                            Arc::new(RwLock::new(HMap::new())),
                                                            hm.clone(),
            );
            n_k_combinatorial.rec_with_memory(leaf_level + 1);
            {
                let mut i = threads_done.lock().unwrap();
                *i += 1;

                if print {
                    println!("\t{i} done in {:?}, v: {} l:{}",
                             now.elapsed(),n_k_combinatorial.seen_vertices,
                             n_k_combinatorial.seen_leaves);

                    println!("\t\t{} ", hm.read().unwrap().len());
                    println!("\t{i} used memory: {}", n_k_combinatorial.max_memory)
                }
            }
            println!("Threads done: % {} solutions: {} mem: {}", 
                     *threads_done.lock().unwrap() as f32 * 100.0 / total_threads as f32,
                hm.read().unwrap().len(),
                n_k_combinatorial.max_memory
            );
        });



    let early_leaves = hm.read().unwrap().clone();
    let total_threads = early_leaves.len();
    println!("Task to parallelize:{}", total_threads);
    //println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    let mut threads_done = Arc::new(Mutex::new(0));
    let hm = Arc::new(RwLock::new(HMap::new()));
    early_leaves.into_par_iter().for_each(|(canon, leaf)|
        {
            // println!("Parallel {i} deployed!");
            let now = Instant::now();
            let mut n_k_combinatorial =  NkComb::from_graph(k, 
                                                            leaf.clone(),
                                                            Config{leaf_level: N,save_leaves: false}, 
                                                            hm.clone(),
                                                            Arc::new(RwLock::new(HashMap::new())));
            n_k_combinatorial.rec(leaf_level_2 + 1);
            {
                let mut i = threads_done.lock().unwrap();
                *i += 1;
            
                if print {
                    println!("\t{i} done in {:?}, v: {} l:{}",
                             now.elapsed(),n_k_combinatorial.seen_vertices,
                             n_k_combinatorial.seen_leaves);
    
                    println!("\t\t{} ", hm.read().unwrap().len());
                    println!("\t{i} used memory: {}", n_k_combinatorial.max_memory)
                }
            }
            println!("Threads done: % {} used mem: {}", *threads_done.lock().unwrap() as f32 * 100.0 / total_threads as f32, n_k_combinatorial.max_memory);
        });
    let elapsed = now.elapsed();

    //println!("Hashmap:{:?}", hm.last().unwrap().read().unwrap());
    println!("effective leaves:{:?}", hm.read().unwrap().len());
    //println!("leaves:{:?}", hm.read().unwrap());
    println!("Elapsed:{:?}", elapsed);
    elapsed
}


#[test]
fn test_n_comb() {
    let mut nkComb = NkComb::new(3);
    let now = Instant::now();
    nkComb.rec(0);
    let duration = now.elapsed();
    println!("vertices: {}\nleaves: {}\nsols: {}", nkComb.seen_vertices, nkComb.seen_leaves, nkComb.sols.len());
    println!("Time elapsed: {:?}", duration);
}