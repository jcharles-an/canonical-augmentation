use std::collections::HashMap;
use std::time::Instant;
use log::debug;
use crate::data_structs;
use crate::data_structs::bitmaps::{BMConf, BM};
use crate::data_structs::consts::{HMap, N, N_CONF};
use crate::data_structs::graph::Graph;
use crate::schreier::automorphism::Automorphism;
use crate::schreier::group::Group;

pub struct NkCombinatorial {
    pub k: usize,
    pub lines: Vec<BMConf>,
    pub points: Vec<BMConf>,
    pub leaves_seen: usize,
    pub vertices_seen: usize,
    pub hashmap: HMap,
    pub hashmaps: Vec<HMap>,

}

impl NkCombinatorial {

    fn new(k: usize, lines: Vec<BMConf>) -> Self {
        let points = Graph::transpose(lines.as_slice());
        NkCombinatorial{ k, lines, points, leaves_seen: 0, vertices_seen: 0, hashmap: HMap::new(), hashmaps: vec![HMap::new(); N_CONF]}
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
            result = result | current;
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
        self.vertices_seen += 1;
        //line is full, advance and reset point
        if self.lines[current_line].len() == self.k {
            //is line isomorphic to previous one?
            let mut aut = Automorphism::new(self.to_levy_graph());
            let C = aut.cert_2();
            if self.hashmaps[current_line].insert(format!("{:?}",C), true) != None {
                debug!("Found iso config");
                return;
            }


            //last line?
            if current_line == N_CONF - 1 {
                return self.process_leaf();
            }
            //advance line
            return self.iso_backtrack(current_line + 1, 0);
        }

        //no more point but line is not full, not a solution
        if current_point == N_CONF{
            return;
        }

        if self.add_point_line_relation(current_point, current_line){
            // advance point
            self.iso_backtrack(current_line, current_point + 1);
            self.delete_point_line_relation(current_point, current_line);
        }
        self.iso_backtrack(current_line, current_point + 1);

    }

    pub fn iso_point_backtrack(&mut self, current_line:usize, current_point:usize) {
        self.vertices_seen += 1;
        //line is full, advance and reset point
        if self.lines[current_line].len() == self.k {
            //last line?
            if current_line == N_CONF - 1 {
                return self.process_leaf();
            }
            //advance line
            return self.iso_point_backtrack(current_line + 1, 0);
        }

        //no more point but line is not full, not a solution
        if current_point == N_CONF{
            return;
        }



        if self.add_point_line_relation(current_point, current_line){
            //is line isomorphic to previous one?
            let mut aut = Automorphism::new(self.to_levy_graph());
            let C = aut.cert_2();
            if self.hashmaps[current_line].insert(format!("{:?}",C), true) != None {
                debug!("Found iso config");
            } else {
                // advance point
                self.iso_point_backtrack(current_line, current_point + 1);
            }
            self.delete_point_line_relation(current_point, current_line);
        }
        self.iso_point_backtrack(current_line, current_point + 1);

    }

    fn process_leaf(&mut self){
        self.leaves_seen += 1;
        let mut aut = Automorphism::new(self.to_levy_graph());
        let C = aut.cert_2();
        if self.hashmap.insert(format!("{:?}",C), true) == None {
            debug!("Found new config");
            //println!("Found new config");
            //Graph::print_ady_mat_BMConf(self.lines.as_slice());
        }
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


#[test]
fn test_7_3() {
    let lines = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
    ];
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    n_k_combinatorial.backtrack(3,0);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);

    println!();
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    n_k_combinatorial.iso_backtrack(3,0);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);
}

#[test]
fn test_9_3() {
    env_logger::init();
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
    n_k_combinatorial.iso_point_backtrack(3,0);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);

    println!();
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    n_k_combinatorial.iso_backtrack(3,0);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);
}


#[test]
fn test_10_3() {
    let lines = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ];
    /*
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    n_k_combinatorial.backtrack(3,0);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);

     */

    println!();
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines.clone());
    n_k_combinatorial.iso_backtrack(3,0);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);
}

#[test]
fn test_13_4() {
    let lines = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ];

    println!();
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(4, lines.clone());
    n_k_combinatorial.iso_backtrack(4,0);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);
}

#[test]
fn test_14_4() {
    let lines = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ];

    println!();
    let now = Instant::now();
    let mut n_k_combinatorial = NkCombinatorial::new(4, lines.clone());
    n_k_combinatorial.iso_backtrack(4,0);
    println!("Elapsed:{:?}", now.elapsed());
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);
}


/*
fn test_9_3(){
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
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines);

    println!("lines:");
    Graph::print_ady_mat_BMConf(n_k_combinatorial.lines.as_slice());
    println!("Points:");
    Graph::print_ady_mat_BMConf(n_k_combinatorial.points.as_slice());

    n_k_combinatorial.backtrack(3,0);
    println!("Seen:{}", n_k_combinatorial.leaves_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);


}

#[test]
fn test_9_3(){
    /*
    let lines = vec![BMConf::new(); N_CONF];
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines);


    n_k_combinatorial.add_point_line_relation(0, 0);
    n_k_combinatorial.add_point_line_relation(1, 0);
    n_k_combinatorial.add_point_line_relation(2, 0);
    n_k_combinatorial.add_point_line_relation(0, 1);
    n_k_combinatorial.add_point_line_relation(3, 1);
    n_k_combinatorial.add_point_line_relation(4, 1);
    n_k_combinatorial.add_point_line_relation(0, 2);
    n_k_combinatorial.add_point_line_relation(5, 2);
    n_k_combinatorial.add_point_line_relation(6, 2);

    let mut aut = Automorphism::new(n_k_combinatorial.to_levy_graph());
    let C = aut.cert_2();
    println!("{C:?}");

    let lines = vec![BMConf::new(); N_CONF];
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines);


    n_k_combinatorial.add_point_line_relation(0, 0);
    n_k_combinatorial.add_point_line_relation(1, 0);
    n_k_combinatorial.add_point_line_relation(2, 0);
    n_k_combinatorial.add_point_line_relation(3, 1);
    n_k_combinatorial.add_point_line_relation(1, 1);
    n_k_combinatorial.add_point_line_relation(5, 1);
    n_k_combinatorial.add_point_line_relation(6, 2);
    n_k_combinatorial.add_point_line_relation(1, 2);
    n_k_combinatorial.add_point_line_relation(8, 2);

    let mut aut = Automorphism::new(n_k_combinatorial.to_levy_graph());
    let C = aut.cert_2();
    println!("{C:?}");

     */


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
    let mut n_k_combinatorial = NkCombinatorial::new(3, lines);

    println!("lines:");
    Graph::print_ady_mat_BMConf(n_k_combinatorial.lines.as_slice());
    println!("Points:");
    Graph::print_ady_mat_BMConf(n_k_combinatorial.points.as_slice());

    n_k_combinatorial.backtrack(3,0);
    println!("Seen:{}", n_k_combinatorial.leaves_seen);
    println!("Hasmap:{:?}", n_k_combinatorial.hashmap);


}



 */


#[test]
pub fn test_iso(){
    //env_logger::init();
    // 9 3 cases -> 3 solutions
    let lines_1 = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 1, 0, 1, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 0, 1, 0, 0, 1, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 1, 0, 0, 0, 0, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 0, 0, 0, 1, 1, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 1, 0, 1, 0, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 1, 0, 1, 1]),
    ];

    let lines_2 = vec![
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 1, 0, 1, 0, 0, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 1, 0, 0, 1, 0, 0, 1, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 0, 1, 0, 0, 0, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 1, 0, 0, 0, 1, 1, 0]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 1, 0, 0, 1, 0, 1]),
        data_structs::bitmaps::BMConf_from_BE_array(&[0, 0, 0, 0, 0, 1, 0, 1, 1]),
    ];

    let nk_1 = NkCombinatorial::new(3, lines_1);
    let nk_2 = NkCombinatorial::new(3, lines_2);

    let mut aut_1 = Automorphism::new(nk_1.to_levy_graph());
    let mut aut_2 = Automorphism::new(nk_2.to_levy_graph());

    aut_1.get_naive_partition();
    let now = Instant::now();
    let C_1 =aut_1.cert_2();
    let group_order = aut_1.group.get_group_order();
    aut_1.group.run();
    aut_1.group.simple_generate_group();
    let time_1 = now.elapsed();

    //aut_1.set_graph(nk_2.to_levy_graph());

    aut_1.group.change_base(aut_1.group.get_identity());

    let C_1_1 = aut_1.get_naive_partition();
    let now = Instant::now();
    let C_2 =aut_1.cert_2();
    let time_2 = now.elapsed();



    println!("group order {}", group_order);
    println!("time1 [{:?}] time21 [{:?}]", time_1, time_2);
    println!("C_1:{:?} C_2:{:?}", C_1, C_2);

    //let c3 = aut_1.cert_2();

    //println!("c3:{:?} C_2:{:?}", c3, C_2);















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