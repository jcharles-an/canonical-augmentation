use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{BitAnd, BitOr, Deref};
use std::path::Prefix;
use std::process;
use std::time::Instant;
use log::{debug, trace};
use memory_stats::memory_stats;
use nauty_pet::graph::CanonGraph;
use petgraph::dot::Dot;
use petgraph::visit::Walker;
use crate::data_structs::bitmaps::{BMConf, BM};
use crate::data_structs::consts::*;
use crate::data_structs::graph::Graph;
use crate::data_structs::nauty_graph::NautyGraph;
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

pub(crate) struct NkComb {
    graph: Graph,
    n: usize,
    k: usize,
    pub seen_vertices: usize,
    pub seen_leaves:usize,
    pub print:usize,
    pub sols: HMap,
    pub solution_path: Vec<SolutionProperties>,
    pub memory: Vec<HMap>,
    pub max_memory: f64,
}

impl NkComb {
    pub fn new(k: usize) -> NkComb {
        let n = N;
        let ady_mat = vec![BM::new();n];
        let g = Graph {
            ady_matrix: ady_mat,
        };
        NkComb {
            graph: g ,
            n,
            k,
            seen_vertices: 0,
            seen_leaves: 0,
            print: 1,
            sols: HashMap::new(),
            solution_path: vec![SolutionProperties::default(); n/2],
            memory: vec![HashMap::new(); n/2],
            max_memory: 0.0,
        }
    }
    pub fn rec( &mut self, current_line: usize){
        self.seen_vertices += 1;
        
        let S = self.get_available_points();

        if S.is_empty() || current_line > self.n {
            self.process_leaf();
            return;
        }


        let gen = NautyGraph::get_aut_generators(&self.graph).unwrap();
        // println!("{:?}", gen);

        let schreier = Schreier::from_generators(gen);
        let data = schreier.get_orbit(current_line, self.n);
        let schreier = Schreier::from_generators(data.stab_generators);
        
        let mut prefixes_holder = Schreier::get_S_choose_k_in_G(schreier.clone(), self.n, &S, self.k);

       // println!("Subsets: {:?}", prefixes_holder);

        for prefix in prefixes_holder {
            self.add_edges(current_line, &prefix);
            
            if self.is_feasible(current_line, &prefix) {
                self.gather_info(current_line, &prefix);
                //if self.print > 0 {
                //    self.is_feasible(current_line, &prefix);

                    // 4. Print the result
               //     println!("{}  is feasible\n", self.graph.custom_print());
               // }
                self.rec(current_line + 1);
            }

            self.remove_edges(current_line, &prefix);
        }
    }


    pub fn rec_with_memory( &mut self, current_line: usize){
        self.seen_vertices += 1;

        if self.seen_vertices % 100000 == 0 {
            println!("seen_vertices: {}", self.seen_vertices);
        }


        let S = self.get_available_points();
        if S.is_empty() || current_line > self.n {
            self.process_leaf();
            return;
        }
       

        let gen = NautyGraph::get_aut_generators(&self.graph).unwrap();
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
                let certificate = self.get_canon_certificate();
                if self.memory[current_line].insert(certificate, true).is_none(){
                    //self.gather_info(current_line, &prefix);
                    self.rec_with_memory(current_line + 1);
                }
                //self.solution_path[current_line]= (prefix.clone(), self.get_canon_certificate(), current_sol, NautyGraph::get_aut_generators(&self.graph).unwrap());
                //if self.print > 0 {
                //    self.is_feasible(current_line, &prefix);

                // 4. Print the result
                //     println!("{}  is feasible\n", self.graph.custom_print());
                // }
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
        if self.sols.insert( canon.clone(), true).is_none() {
            println!("{} {canon}", self.sols.len());
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
    let mut nkComb = NkComb::new(3);
    let now = Instant::now();
    nkComb.rec(0);
    let duration = now.elapsed();
    println!("vertices: {}\nleaves: {}\nsols: {}", nkComb.seen_vertices, nkComb.seen_leaves, nkComb.sols.len());
    println!("Time elapsed: {:?}", duration);
}