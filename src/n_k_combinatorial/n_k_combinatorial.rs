use std::any::Any;
use std::backtrace::BacktraceStatus::Captured;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::Bound::Unbounded;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::channel;
use std::time::Instant;
use graph_canon::CanonLabeling;
use log::debug;
use petgraph::Undirected;
use crate::data_structs;
use crate::data_structs::bitmaps::{BMConf, BM, BMC};
use crate::data_structs::consts::{HMap, N, N_CONF};
use crate::data_structs::graph::Graph;
use crate::schreier::automorphism::Automorphism;
use crate::schreier::group::Group;
use rayon::prelude::*;
use nauty_pet::prelude::*;
use petgraph::dot::Config::GraphContentOnly;
use petgraph::prelude::*;
use crate::n_k_combinatorial::base_configs::get_n_k_base;
use crate::n_k_combinatorial::n_k_combinatorial::CanonOption::Nauty;
use crate::schreier::permutation::Permutation;

pub struct NkCombinatorial {
    pub k: usize,
    pub lines: Vec<BMConf>,
    pub points: Vec<BMConf>,
    pub leaves_seen: usize,
    pub vertices_seen: usize,
    pub leaf_level: usize,
    pub hashmap: HMap,
    pub hashmaps: Vec<HMap>,
    pub hashmaps_lock: Arc<RwLock<Vec<HMap>>>,
    pub early_leaves: Vec<Vec<BMConf>>,
    get_canon: fn(&mut Self)->String,

}


#[derive(Eq, PartialEq)]
pub enum CanonOption{
    Nauty,
    Inhouse
}

impl NkCombinatorial {

    pub fn new(k: usize, lines: Vec<BMConf>) -> Self {
        let points = Graph::transpose(lines.as_slice());
        NkCombinatorial{ k, lines, points,
            leaves_seen: 0,
            vertices_seen: 0,
            leaf_level:  N_CONF - 1,
            hashmap: HMap::new(),
            hashmaps: vec![HMap::new(); N_CONF],
            early_leaves: Vec::new(),
            get_canon: Self::nauty_canon,
            hashmaps_lock: Arc::new(RwLock::new(vec![HMap::new(); N_CONF]))
        }
    }

    pub fn new_par(k: usize, lines: Vec<BMConf>, hashmaps_lock: Arc<RwLock<Vec<HMap>>>) -> Self {
        let points = Graph::transpose(lines.as_slice());
        NkCombinatorial{ k, lines, points,
            leaves_seen: 0,
            vertices_seen: 0,
            leaf_level:  N_CONF - 1,
            hashmap: HMap::new(),
            hashmaps: vec![HMap::new(); N_CONF],
            early_leaves: Vec::new(),
            get_canon: Self::nauty_canon,
            hashmaps_lock
        }
    }
    
    pub fn set_canon_method(&mut self, option: CanonOption){
        if option == Nauty {
            self.get_canon = Self::nauty_canon;
        } else { 
            self.get_canon = Self::inhouse_canon;
        }
    }
    
    pub fn set_leaf_level(&mut self, level: usize){
        self.leaf_level = level;
    }

    fn add_point_line_relation(&mut self, point: usize, line: usize) -> bool {
        self.lines[line].set(point, true);
        self.points[point].set(line, true);

        //Is this configuration valid?
        if self.lines[line].len() > self.k ||
            self.points[point].len() > self.k {
            self.delete_point_line_relation(point, line);
            return false
        }
        let mut result = BMConf::new();

        let mask = BMConf::mask(line);
        for i in &self.lines[line] {
            let current = self.points[i] & mask;
            let test = result & current;
            if !test.is_empty(){
                self.delete_point_line_relation(point, line);
                return false
            }
            result |= current;
        }

        true
    }

    fn delete_point_line_relation(&mut self, point: usize, line: usize){
        self.lines[line].set(point, false);
        self.points[point].set(line, false);
    }


    pub fn backtrack(&mut self, current_line:usize, current_point:usize) {
        self.vertices_seen += 1;
        //line is full, advance and reset point
        if self.lines[current_line].len() == self.k {
            //last line?
            if current_line == N_CONF - 1 {
                return self.process_leaf();
            }
            //advance line
            return self.backtrack(current_line + 1, 0);
        }

        //no more point but line is not full, not a solution
        if current_point == N_CONF{
            return;
        }

        if self.add_point_line_relation(current_point, current_line){
            // advance point
            self.backtrack(current_line, current_point + 1);
            self.delete_point_line_relation(current_point, current_line);
        }
        self.backtrack(current_line, current_point + 1);

    }
    

    pub fn iso_backtrack(&mut self, current_line:usize, current_point:usize) {
        //line is full, advance and reset point
        if self.lines[current_line].len() == self.k {
            //is line isomorphic to previous one?
            let mut aut = Automorphism::new(self.to_levy_graph());
            let C = aut.cert_2();
            if self.hashmaps[current_line].insert(format!("{:?}",C), true).is_some() {
                debug!("Found iso config");
                return;
            }


            //last line?
            if current_line == self.leaf_level {
                return self.process_leaf_iso();
            }
            //advance line
            return self.iso_backtrack(current_line + 1, 0);
        }

        //no more point but line is not full, not a solution
        if current_point == N_CONF{
            return;
        }
        
        self.vertices_seen += 1;
        if self.add_point_line_relation(current_point, current_line){
            // advance point
            self.iso_backtrack(current_line, current_point + 1);
            self.delete_point_line_relation(current_point, current_line);
        }
        self.iso_backtrack(current_line, current_point + 1);

    }

    pub fn memory_iso_backtrack(&mut self, current_line:usize, current_point:usize, mut aut: Automorphism) -> Automorphism {
        self.vertices_seen += 1;
        //line is full, advance and reset point
        if self.lines[current_line].len() == self.k {

            let back_aut = aut.clone();
            //is line isomorphic to previous one?
            aut.set_graph(self.to_levy_graph());
            let now = Instant::now();
            let C = aut.cert_2();
            println!("elapsed: {:?}", now.elapsed());
            if self.hashmaps[current_line].insert(format!("{:?}",C), true) != None {
                debug!("Found iso config");
                return back_aut;
            }


            //last line?
            if current_line == self.leaf_level {
                self.process_leaf_iso();
                return back_aut;
            }
            //advance line
            self.memory_iso_backtrack(current_line + 1, 0, aut);
            return back_aut;
        }

        //no more point but line is not full, not a solution
        if current_point == N_CONF{
            return aut;
        }

        if self.add_point_line_relation(current_point, current_line){
            // advance point
            aut = self.memory_iso_backtrack(current_line, current_point + 1, aut);
            self.delete_point_line_relation(current_point, current_line);
        }
        self.memory_iso_backtrack(current_line, current_point + 1, aut)
    }
    
    

    pub fn iso_var_backtrack(&mut self, current_line:usize, current_point:usize,point_count:usize, interval: usize) {
        //line is full, advance and reset point
        if self.lines[current_line].len() == self.k {
            //last line?
            if current_line == self.leaf_level  {
                return self.process_leaf_iso();
            }
            //advance line
            return self.iso_var_backtrack(current_line + 1, 0,0,interval);
        }

        //no more point but line is not full, not a solution
        if current_point == N_CONF{
            return;
        }


        self.vertices_seen += 1;
        if self.add_point_line_relation(current_point, current_line){
            //is line isomorphic to previous one?
            if point_count == interval - 1 {
                //is line isomorphic to previous one?
                let C = (self.get_canon)(self);
                if self.hashmaps[current_line].insert(format!("{:?}",C), true).is_some()  {
                    debug!("Found iso config");
                }else {
                    // advance point
                    self.iso_var_backtrack(current_line, current_point + 1, 0, interval);
                }
            } else {
                self.iso_var_backtrack(current_line, current_point + 1,point_count + 1, interval);
            }

            self.delete_point_line_relation(current_point, current_line);
        }
        self.iso_var_backtrack(current_line, current_point + 1,point_count, interval);

    }


    pub fn iso_var_backtrack_share(&mut self, current_line:usize, current_point:usize,point_count:usize, interval: usize) {
        //line is full, advance and reset point
        if self.lines[current_line].len() == self.k {
            //last line?
            if current_line == self.leaf_level  {
                return self.process_leaf_iso();
            }
            //advance line
            return self.iso_var_backtrack_share(current_line + 1, 0,0,interval);
        }

        //no more point but line is not full, not a solution
        if current_point == N_CONF{
            return;
        }


        self.vertices_seen += 1;
        if self.add_point_line_relation(current_point, current_line){
            //is line isomorphic to previous one?
            if point_count == interval - 1 {
                //is line isomorphic to previous one?
                let C = (self.get_canon)(self);
                let mut is_old_certificate = false;
                {
                    let b = self.hashmaps_lock.read().unwrap();
                    is_old_certificate = b[current_line].contains_key(&C);
                }
                if is_old_certificate {
                    debug!("Found iso config");
                }else {
                    {
                        let mut a = self.hashmaps_lock.write().unwrap();
                        a[current_line].insert(C, true);
                    }
                    // advance point
                    self.iso_var_backtrack_share(current_line, current_point + 1, 0, interval);
                }
            } else {
                self.iso_var_backtrack_share(current_line, current_point + 1,point_count + 1, interval);
            }

            self.delete_point_line_relation(current_point, current_line);
        }
        self.iso_var_backtrack_share(current_line, current_point + 1,point_count, interval);

    }

    fn process_leaf(&mut self){
        self.leaves_seen += 1;
        let mut aut = Automorphism::new(self.to_levy_graph());
        let C = aut.cert_2();
        if self.hashmap.insert(format!("{:?}",C), true).is_none() {
            debug!("Found new config");
            println!("Found new config");
            Graph::print_ady_mat_BMConf(self.lines.as_slice());
        }
    }

    fn process_leaf_iso(&mut self){
        self.leaves_seen += 1;
        self.early_leaves.push(self.lines.clone());
        // println!("Found new config");
        // Graph::print_ady_mat_BMConf(self.lines.as_slice());
    }

    pub fn inhouse_canon(&mut self) -> String{
        let mut aut = Automorphism::new(self.to_levy_graph());
        let C = aut.cert_2();
        format!("{:?}", C)
    }
    
    pub fn nauty_canon(&mut self) -> String{
        let canon = CanonGraph::from(self.to_pet_graph());
        let mut hasher = DefaultHasher::new();
        canon.hash(&mut hasher);
        hasher.finish().to_string()
    }
    
    pub fn to_levy_graph(&self) -> Graph {
        let mut ady_matrix = vec![BM::new(); N];
        let quarter = Graph::transpose(self.lines.as_slice());
        let quarter_len = quarter.len();
        for i in 0..quarter_len {
            let mut new_bitmap = BM::new();
            for j in &self.lines[i]{
                new_bitmap.set(j+N_CONF, true);
            }
            ady_matrix[i] = new_bitmap;
        }
        for i in quarter_len..N {
            for j in &quarter[i-N_CONF]{
                ady_matrix[i].set(j, true);
            }
        }
        Graph { ady_matrix }
    }
    

    pub fn to_pet_graph(&self) -> petgraph::Graph::<(), (), Undirected> {
        let mut g = petgraph::Graph::new_undirected();
        for _ in 0..N_CONF*2{
            g.add_node(());
        }
        for (l,line) in self.lines.iter().enumerate() {
            for p in line {
                g.add_edge(NodeIndex::new(l), NodeIndex::new(p+N_CONF),());
            }
        }
        g
    }


    pub fn from_levy_graph(graph: &Graph) -> NkCombinatorial {
        let mut lines = vec![BMConf::new(); N_CONF];

        for i in 0..N_CONF {
            for bit_index in &graph.ady_matrix[i]{
                lines[i].set(bit_index - N_CONF, true);
            }
        }
        let mut k = lines[0].len();

        NkCombinatorial::new(k, lines)
    }
}

fn get_jobs(lines: Vec<BMConf>, k: usize, leaf_level: usize){
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(k, lines.clone());
    n_k_combinatorial.set_canon_method(Nauty);
    n_k_combinatorial.set_leaf_level(leaf_level);
    n_k_combinatorial.iso_var_backtrack_share(k,0, 0,k);
    let hm = n_k_combinatorial.hashmaps_lock;


    println!("Seen leaves:{}", n_k_combinatorial.early_leaves.len());
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
}

pub fn launch_parallel(lines: Vec<BMConf>, k: usize, leaf_level: usize) {
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(k, lines.clone());
    n_k_combinatorial.set_canon_method(Nauty);
    n_k_combinatorial.set_leaf_level(leaf_level);
    n_k_combinatorial.iso_var_backtrack_share(k,0, 0,k);
    let hm = n_k_combinatorial.hashmaps_lock;


    println!("Seen leaves:{}", n_k_combinatorial.early_leaves.len());
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);

    (0..n_k_combinatorial.early_leaves.len()).into_par_iter().for_each(|i|
        {
            println!("Parallel {i} deployed!");
            let now = Instant::now();
            let leaf = &n_k_combinatorial.early_leaves[i];
            let mut n_k_combinatorial = NkCombinatorial::new_par(k, leaf.clone(), hm.clone());
            n_k_combinatorial.iso_var_backtrack_share(leaf_level,0,0,k);
            println!("Parallel {i} done in {}!", now.elapsed().as_secs());
        });

    println!("Hashmap:{:?}", hm.read().unwrap().last().unwrap());
    println!("effective leaves:{:?}", hm.read().unwrap().last().unwrap().len());

    println!("Elapsed:{:?}", now.elapsed());
}


#[test]
fn test_7_3() {
    let k = 3;
    let lines = get_n_k_base(7, k);

    println!();
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(k, lines.clone());
    n_k_combinatorial.iso_var_backtrack(k,0,0,k);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmaps_lock.read().unwrap().last().unwrap());
    
    println!();
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(k, lines.clone());
    n_k_combinatorial.iso_var_backtrack_share(k,0,0,k);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmaps_lock.read().unwrap().last().unwrap());
}

#[test]
fn test_7_3_iso() {
    let lines = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
    ];

    let lines = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 1, 0, 1, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 0, 1, 0, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 1, 0, 0, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 0, 1, 1, 0]),
    ];


    let lines = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
    ];
    
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    let g = n_k_combinatorial.to_levy_graph();
    let mut aut = Automorphism::new(g.clone());
    aut.cert_2();
    
    Graph::print_ady_mat(g.ady_matrix.as_slice());
    println!();
    
    println!("Perm:{:}", Permutation::new(aut.mu.clone().unwrap()));
    
    let ady = Automorphism::change_ady_mat(g.ady_matrix.as_slice(),aut.mu.unwrap());
    Graph::print_ady_mat(ady.as_slice());
    println!();
    let g1 = Graph{ady_matrix: ady.clone()};
    let nk = NkCombinatorial::from_levy_graph(&g1);
    Graph::print_ady_mat_BMConf(nk.lines.as_slice());
}

#[test]
fn test_9_3() {
    env_logger::init();
    let k = 3;
    let lines = get_n_k_base(9, k);

    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(k, lines.clone());
    n_k_combinatorial.set_canon_method(Nauty);
    n_k_combinatorial.iso_backtrack(k,0);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmaps.last().unwrap());
    println!();
    
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    n_k_combinatorial.set_canon_method(Nauty);
    n_k_combinatorial.iso_var_backtrack(3,0,0,3);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmaps.last().unwrap());
    println!();

    /*
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    n_k_combinatorial.iso_point_backtrack(3,0);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);
    println!();

    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    n_k_combinatorial.iso_var_backtrack(3,0, 0,4);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);
    println!();
*/

    /*
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    n_k_combinatorial.early_iso_backtrack(3,0, 5);
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);
    println!();

    (0..n_k_combinatorial.early_leaves.len()).into_par_iter().for_each(|i|
        {
            let leaf = &n_k_combinatorial.early_leaves[i];
            let mut n_k_combinatorial = NkCombinatorial::new(3, leaf.clone());
            n_k_combinatorial.iso_backtrack(5,0);
            println!("Hashmap:{:?}", n_k_combinatorial.hashmap);
        });

    println!("Elapsed:{:?}", now.elapsed());


     */
    /*
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    n_k_combinatorial.iso_short_backtrack(3,0);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);
    println!();

     */

}


#[test]
pub fn test_10_3() {
    let k = 3;
    let lines = get_n_k_base(10, k);
    println!();
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    n_k_combinatorial.iso_var_backtrack(3,0,0,3);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmaps.last().unwrap());
    println!();

    launch_parallel(lines, k, 4);
}

#[test]
fn test_13_4() {
    let k = 4;
    let lines = get_n_k_base(13, k);

    println!();
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(k, lines.clone());
    n_k_combinatorial.iso_var_backtrack(k,0,0,k);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);


}

#[test]
fn test_14_4() {
    let k = 4;
    let lines = get_n_k_base(14, k);

    println!();
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(4, lines.clone());
    let max_line = 5;
    n_k_combinatorial.iso_backtrack(4,0);
    n_k_combinatorial.set_canon_method(Nauty);
    println!("Elapsed:{:?}", now.elapsed());
    
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);
    println!();

    return;
    (0..n_k_combinatorial.early_leaves.len()).into_par_iter().for_each(|i|
        {
            let leaf = &n_k_combinatorial.early_leaves[i];
            let mut n_k_combinatorial = NkCombinatorial::new(4, leaf.clone());
            n_k_combinatorial.iso_backtrack(max_line,0);
            println!("Hashmap:{:?}", n_k_combinatorial.hashmap);
        });

    println!("Elapsed:{:?}", now.elapsed());
}

#[test]
fn test_15_4() {
    let k = 4;
    let lines = get_n_k_base(15, k);

    println!();
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(k, lines.clone());
    let leaf_level = N_CONF - 1;
    n_k_combinatorial.set_leaf_level(leaf_level);
    n_k_combinatorial.iso_var_backtrack(k,0,0,k);
    n_k_combinatorial.set_canon_method(Nauty);
    println!("Elapsed:{:?}", now.elapsed());

    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmaps[leaf_level]);
    println!();


    launch_parallel(lines, k, 5);
}


#[test]
fn test_16_4() {
    let k = 4;
    let lines = get_n_k_base(16, k);
    launch_parallel(lines, k, 6);
}


#[test]
fn test_17_4() {
    let k = 4;
    let lines = get_n_k_base(17, k);
    launch_parallel(lines, k, 7);
}


#[test]
fn test_n_k_combinatorial() {
    let lines_1 = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 1, 0, 0, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 0, 1, 1, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 1, 0, 1, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 0, 1, 0, 1]),
    ];

    /*
    let lines_1 = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 0, 0, 0, 0]),
    ];

     */

    let lines_2 = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 1, 0, 0, 0]),
    ];

    let nk_combinatorial = NkCombinatorial::new(3, lines_1);
    let nk_combinatorial_2 = NkCombinatorial::new(3, lines_2);


    let g = nk_combinatorial.to_levy_graph();
    let mut aut = Automorphism::new(g);
    let C = aut.cert_2();
    let mu = aut.mu.unwrap();
    println!("102981028767737582336");
    println!("Cert: {:?}", C);
    println!("Group order: {}", aut.group.get_group_order());
    //println!("mu: {:?}", &mu);

    let g = nk_combinatorial_2.to_levy_graph();
    let mut aut = Automorphism::new(g);
    let C = aut.cert_2();
    let mu = aut.mu.unwrap();
    println!("Cert: {:?}", C);
    println!("Group order: {}", aut.group.get_group_order());
    //println!("mu: {:?}", &mu);


   // Graph::print_ady_mat(&aut.g.ady_matrix);

    /*
    let nk_combinatorial = NkCombinatorial::from_levy_graph(&aut.g);
    println!("nk_combinatorial:");
    Graph::print_ady_mat_BMConf(&nk_combinatorial.sets);

    let new_mat = Automorphism::change_ady_mat(&aut.g.ady_matrix.as_slice(), mu );

    println!("new_mat: ");
    Graph::print_ady_mat(&new_mat);
    let nk_combinatorial = NkCombinatorial::from_levy_graph(&Graph{ady_matrix: new_mat});
    println!("nk_combinatorial:");
    Graph::print_ady_mat_BMConf(&nk_combinatorial.sets);

     */

}

#[test]
fn test_petgraph(){
    let lines = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ];

    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    let mut aut = Automorphism::new(n_k_combinatorial.to_levy_graph());
    let C = aut.cert_2();
    println!("{:?}",now.elapsed());

    let now = Instant::now();
    let g_1 = CanonGraph::from(n_k_combinatorial.to_pet_graph());
    println!("{:?}",now.elapsed());
    
    //let l = CanonLabeling::new(&g_1);


    let lines = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 1, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ];

    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    let mut aut = Automorphism::new(n_k_combinatorial.to_levy_graph());
    let C_2 = aut.cert_2();
    println!("{:?}",now.elapsed());

    let now = Instant::now();
    let g_2_1 = CanonGraph::from(n_k_combinatorial.to_pet_graph());
    //let l_2 = CanonLabeling::new(&g_2_1);
    println!("{:?}",now.elapsed());
    
    

    let mut hm = HashMap::new();
    hm.insert(&g_1, true);
    hm.insert(&g_2_1, true);

    println!("are graphs identical? {:?}",g_1.is_identical(&g_2_1));
    println!();

    println!("{:?}",hm.len());
    println!("{:?} {:?}",C,C_2);



}