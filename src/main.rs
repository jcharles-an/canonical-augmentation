#![allow(dead_code)]
#![allow(unused)]
#![allow(non_snake_case)]
#![allow(refining_impl_trait)]

use std::collections::HashMap;
use std::env;
use std::time::Instant;
use log::{debug, info};
use rayon::prelude::*;
use crate::data_structs::consts::{N, N_CONF};
use crate::data_structs::graph::Graph;
use crate::n_k_combinatorial::n_k_combinatorial::CanonOption::{Inhouse, Nauty};
use crate::n_k_combinatorial::base_configs::*;
use crate::group_actions::groupAction::GroupAction;
use data_structs::permutation::Permutation;
use crate::schreier::stabilizer_chain::RecStabilizerChain;

use nauty_Traces_sys::*;
use std::ffi::{c_char, CString};
use std::ptr;
use crate::data_structs::bitmaps::BM;
use crate::data_structs::logger::init_logger;
use crate::n_k_combinatorial::{NkCom, NkCombManager};
use crate::n_k_combinatorial::canaugFullPoint::CanaugFullPoint;
use crate::n_k_combinatorial::canaugManager::CanaugManager;
use crate::n_k_combinatorial::canonicalAugmenterTrait::{CanonicalAugmenter, Memory};
use crate::n_k_combinatorial::n_k_combinatorial::{launch_parallel, NkCombinatorial};
use crate::n_k_combinatorial::NkCombManager::{manager, parallel_manager};

mod schreier;
mod data_structs;
mod n_k_combinatorial;
mod groups;
mod group_actions;

fn main_correct_pizana() {
    /*
    let mut nkComb = NkComb::new(4);
    nkComb.config = Config{leaf_level: N, save_leaves: true};
    let now = Instant::now();
    nkComb.rec_over_full_set(0);
    let duration = now.elapsed();
    println!("vertices: {}\nleaves: {}\nsols: {}", nkComb.seen_vertices, nkComb.seen_leaves, nkComb.early_leaves.read().unwrap().len());
    println!("Time elapsed: {:?}", duration);




    let mut nkComb = NkComb::new(4);
    nkComb.config = Config{leaf_level: N, save_leaves: true};
    let now = Instant::now();
    nkComb.rec_over_full_set_with_memory(0);
    let duration = now.elapsed();
    println!("vertices: {}\nleaves: {}\nsols: {}", nkComb.seen_vertices, nkComb.seen_leaves, nkComb.early_leaves.read().unwrap().len());
    println!("Time elapsed: {:?}", duration);
    */


    NkCombManager::launch_parallel_full_p_2(4,9,16,false);
}


fn main() {
    init_logger();

    let ady_mat = vec![BM::new();N];
    let mut g = Graph {
        ady_matrix: ady_mat,
    };
    let k = 3;
    println!("Canag calculation for N_k: {N_CONF}_{k}");

    let mut sum = 0;
    let total = 1;

    for i in 0..total {
        let mut memory = Memory::new(k, N_CONF, None);
        let now = Instant::now();
        CanaugFullPoint::canaug_corrected(k, &mut g, 0, &mut memory, 10);
        let elapsed = now.elapsed();
        sum += elapsed.as_millis();
        println!("{:?}\t{}", elapsed, memory.seen_leaves);
    }

    println!("Average: {}ms",sum as f64 / total as f64);
}
fn main_bfs() {
    init_logger();

    info!("Starting schreier");
    let args: Vec<String> = env::args().collect();

    let mut k = 3;
    if args.len() > 1 {
        k = args[1].parse::<usize>().unwrap();
        info!("Found k {k}");
    }

    CanaugManager::bfs(k);
    return;

}

#[test]
fn test_main(){

}

fn parse_graphs_from_string(graph_str: &str) -> impl Iterator + use<'_>{
    graph_str[1..graph_str.len() -1 ].split(",")
}

fn main_pre_share() {
    env_logger::init();
    info!("Starting schreier");
    let args: Vec<String> = env::args().collect();

    let mut k = 3;
    if args.len() > 1 {
        k = args[1].parse::<usize>().unwrap();

    }
    let lines = get_n_k_base(N_CONF,k);
    let leaf_level = 4;
    let print = false;
    
    /*
    let mut nkComb = crate::n_k_combinatorial::NkCom::NkComb::new(3);
    let now = Instant::now();
    nkComb.rec(0);
    let duration = now.elapsed();
    println!("vertices: {}\nleaves: {}\nsols: {}", nkComb.seen_vertices, nkComb.seen_leaves, nkComb.sols.len());
    println!("Time elapsed: {:?}", duration);

    let op = args[2].parse::<usize>().unwrap();
    

        {
            println!("\n\nSequential ISO prune");
            let mut n_k_combinatorial = NkCombinatorial::new(k, lines.clone());
            n_k_combinatorial.set_canon_method(Nauty);
            let now = Instant::now();
            n_k_combinatorial.iso_backtrack(k, 0);
            let elapsed_2 = now.elapsed();
            println!("Elapsed:{:?}", elapsed_2);
            println!("vertices: {}\nleaves: {}\nsols: {}\nmax_mem: {}",
                     n_k_combinatorial.vertices_seen, n_k_combinatorial.leaves_seen, n_k_combinatorial.early_leaves.len(), n_k_combinatorial.max_memory);

            for (i, val) in n_k_combinatorial.hashmaps.iter().enumerate(){
                println!("{} {}", i, val.len())
            }
        }

        {
            println!("\n\nSequential ISO prune pizaña");
            let mut nkComb = crate::n_k_combinatorial::NkCom::NkComb::new(k);
            let now = Instant::now();
            nkComb.rec_with_memory(0);
            let duration = now.elapsed();
            //println!("Memory: {:?}", nkComb.memory);
            println!("Time elapsed: {:?}", duration);
            println!("vertices: {}\nleaves: {}\nsols: {}\nmax_mem: {}",
                     nkComb.seen_vertices, nkComb.seen_leaves, nkComb.sols.len(), nkComb.max_memory);
            
            for (i, val) in nkComb.memory.iter().enumerate(){
                println!("{} {}", i, val.len())
            }
        }
    */

    
    println!("\n\nParallel Pizaña ISO prune");
    let elapsed = NkCombManager::launch_parallel_p(k,6, 9, print);
    
    
    
    println!("\n\nParallel ISO prune");
    let elapsed = launch_parallel(lines,k,5, print); 
    
}

fn main_nk_combination(){
    env_logger::init();
    info!("Starting schreier");
    let args: Vec<String> = env::args().collect();

    let mut k = 3;
    if args.len() > 1 {
        k = args[1].parse::<usize>().unwrap();
        
    }
    let lines = get_n_k_base(N_CONF,k);

    //get_jobs(lines, k, 6);
    /*
    println!("No ISO prune");
    let mut n_k_combinatorial = NkCombinatorial::new(k, lines.clone());
    n_k_combinatorial.set_canon_method(Nauty);
    let now = Instant::now();
    n_k_combinatorial.backtrack(k,0);
    let elapsed_1 = now.elapsed();
    println!("Elapsed:{:?}", elapsed_1);
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Non-isomorphic solutions:{:?}", n_k_combinatorial.hashmap.len());
    
*/

    

    println!("\n\nSequential ISO prune");
    let mut n_k_combinatorial = NkCombinatorial::new(k, lines.clone());
    n_k_combinatorial.set_canon_method(Nauty);
    let now = Instant::now();
    n_k_combinatorial.iso_backtrack(k,0);
    let elapsed_2 = now.elapsed();
    println!("Elapsed:{:?}", elapsed_2);
    println!("Seen leaves:{}", n_k_combinatorial.leaves_seen);
    println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
    println!("Non-isomorphic solutions:{:?}", n_k_combinatorial.early_leaves.len());

    

    println!("\n\nParallel ISO prune");
    let elapsed = launch_parallel(lines,k,4, false);
    // println!("\n\nTime after parallelization: {:.4}%", (elapsed.as_secs_f64() / elapsed_2.as_secs_f64()) * 100.0);
    
}


fn main_rubik() {

    let sc = RecStabilizerChain::get_stabilizer_chain(vec![
        Permutation::new(vec![1, 2, 3, 0]),
        Permutation::new(vec![3, 2, 1, 0]),
    ]);

    println!("C_4 order: {}\n\n",sc.get_order());




    let sc = RecStabilizerChain::get_stabilizer_chain(vec![
        Permutation::from_cycles(vec![vec![0, 1, 2]]),
        Permutation::from_cycles(vec![vec![1, 2, 3]]),
    ]);
    println!("Example order: {}",sc.get_order());

    let r = Permutation::new(vec![0, 1, 3, 2]);
    let belongs = sc.test_belonging(&r);
    println!("Does {r:?} belong to example? {belongs}");
    let r = Permutation::new(vec![1, 3, 2, 0]);
    let belongs = sc.test_belonging(&r);
    println!("Does {r:?} belong to example? {belongs}");
    println!("\n\n");




    let u =  Permutation::from_cycles(vec![
        vec![0, 2, 7, 5],
        vec![1, 4, 6, 3],
        vec![8,32,24,16],
        vec![9,33,25,17],
        vec![10,34,26,18],
    ]);

    let l = Permutation::from_cycles(vec![
        vec![8,10,15,13],
        vec![9,12,14,11],
        vec![0,16,40,39],
        vec![3,19,43,36],
        vec![5,21,45,34],
    ]);
    let f = Permutation::from_cycles(vec![
        vec![16,18,23,21],
        vec![17,20,22,19],
        vec![5,24,42,15],
        vec![6,27,41,12],
        vec![7,29,40,10],
    ]);
    let r = Permutation::from_cycles(vec![
        vec![24,26,31,29],
        vec![25,28,30,27],
        vec![2,37,42,18],
        vec![4,35,44,20],
        vec![7,32,47,23],
    ]);
    let b = Permutation::from_cycles(vec![
        vec![32,34,39,37],
        vec![33,36,38,35],
        vec![2, 8,45,31],
        vec![1,11,46,28],
        vec![0,13,47,26],
    ]);
    let d = Permutation::from_cycles(vec![
        vec![40,42,47,45],
        vec![41,44,46,43],
        vec![13,21,29,37],
        vec![14,22,30,38],
        vec![15,23,31,39],
    ]);
    let sc = RecStabilizerChain::get_stabilizer_chain(vec![u, l, f, r, d]);


    let order = sc.get_order();
    println!("rubik order according to literature: \n43252003274489856000\n",);
    println!("rubik order: \n{}",order);
}





