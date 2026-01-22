use rayon::iter::ParallelIterator;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::time::{Duration, Instant};
use log::{debug, info};
use rayon::prelude::IntoParallelIterator;
use rayon::{current_num_threads, scope};
use crate::data_structs::bitmaps::BM;
use crate::data_structs::consts::{HMap, N, N_C, N_CONF};
use crate::data_structs::graph::Graph;
use crate::data_structs::logger::init_logger;
use crate::n_k_combinatorial::canaugFullPoint::CanaugFullPoint;
use crate::n_k_combinatorial::canonicalAugmenterTrait::{CanonicalAugmenter, Memory};
use crate::n_k_combinatorial::NkCom::{Config, NkComb};
use crate::n_k_combinatorial::canaugSingleEdge::SingleEdgeCanaug;

pub struct CanaugManager {}

impl CanaugManager {
    pub fn launch_parallel(k: usize, leaf_level: usize, leaf_level_2: usize, print: bool) -> Duration {
        let now = Instant::now();
        let ady_mat = vec![BM::new(); N];
        let mut g = Graph {
            ady_matrix: ady_mat,
        };
        let mut memory = Memory::new(k, N_CONF, None);

        CanaugFullPoint::canaug_corrected(k, &mut g, 0, &mut memory, leaf_level);

        let early_leaves = memory.early_leaves;
        let total_threads = early_leaves.len();
        let mut threads_done = Arc::new(Mutex::new(0));
        let mut solutions = Arc::new(Mutex::new(0));
        println!("Task to parallelize:{}", total_threads);
        //println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
        early_leaves.into_par_iter().for_each(|(mut leaf)|
            {
                // println!("Parallel {i} deployed!");
                let thread_now = Instant::now();
                let mut memory = Memory::new(k, N_CONF, None);
                CanaugFullPoint::canaug_corrected(k, &mut leaf, leaf_level
                                                   + 1, &mut memory, 70);
                {
                    let mut i = threads_done.lock().unwrap();
                    *i += 1;

                    if print {
                        println!("\t{i} done in {:?}, v: {} l:{}",
                                 now.elapsed(), memory.seen_vertices,
                                 memory.seen_leaves);
                    }

                    let mut i = solutions.lock().unwrap();
                    *i += memory.seen_leaves;
                }


                println!("%\tl_t\tg_t\tl_s\tg_s\tm_d\n{:.2} \t{}\t{}\t{}\t{}\t{}",
                         *threads_done.lock().unwrap() as f32 * 100.0 / total_threads as f32,
                         thread_now.elapsed().as_secs(),
                         now.elapsed().as_secs(),
                         memory.seen_leaves,
                         *solutions.lock().unwrap(),
                         memory.max_seen_level
                );


            });
        let elapsed = now.elapsed();

        //println!("Hashmap:{:?}", hm.last().unwrap().read().unwrap());
        println!("effective leaves:{:?}", solutions.lock().unwrap());
        //println!("leaves:{:?}", hm.read().unwrap());
        println!("Elapsed:{:?}", elapsed);
        elapsed
    }



    pub fn launch_auto_balanced_parallel(k: usize, print: bool) {
        let now = Instant::now();
        let ady_mat = vec![BM::new(); N];
        let mut g = Graph {
            ady_matrix: ady_mat,
        };
        let mut memory = Memory::new(k, N_CONF,None);

        let active_threads  = Arc::new(AtomicUsize::new(0));
        let num_total_threads = current_num_threads();
        let found_leaves = Arc::new(AtomicUsize::new(0));
        let interval = 10;
        info!("N_k conf {N_CONF}_{k}");
        info!("Total Rayon threads: {}", num_total_threads);
        info!("Interval: {}", interval);


        // 1. Create the Rayon Scope
        scope(|s| {
            // 2. Start the recursive process with the root node
            s.spawn(|inner| {
                let mut processed_nodes = 0;
                let mut local_found_leaves = 0;
                let max_level = 70; // Just to avoid early stop, max possible level is N_CONF * 2
                let mut offload_work = false;
                let mut active_count = 0;
                CanaugFullPoint::scoped_canaug(k, g.clone(), 0, max_level, interval, &active_threads, num_total_threads, &found_leaves, &mut local_found_leaves,&mut processed_nodes,true,0, &mut offload_work, &mut active_count ,inner);
            });

        });

        let elapsed = now.elapsed();

        //println!("Hashmap:{:?}", hm.last().unwrap().read().unwrap());
        info!("effective leaves: {:?}", found_leaves.load(std::sync::atomic::Ordering::SeqCst));
        //println!("leaves:{:?}", hm.read().unwrap());
        info!("Elapsed:{:?}", elapsed);
        //elapsed
    }


    pub fn bfs(k: usize){
        let ady_mat = vec![BM::new(); N];
        let mut g = Graph {
            ady_matrix: ady_mat,
        };
        let mut graphs = vec![g];
        let mut solutions = 0;

        let now = Instant::now();
        for i in 0..N_CONF {
            info!("level {i}");
            let now = Instant::now();
            info!("tasks to process {}", graphs.len());
            (graphs, solutions) = Self::calculate_level(k,i,false, graphs.as_mut_slice());
            info!("elapsed: {:?}", now.elapsed());
            info!("");
        }

        info!("All Solutions:{}", solutions);
        info!("elapsed: {:?}", now.elapsed());
    }

    pub fn calculate_level(k: usize, leaf_level: usize, print: bool, graphs: &mut [Graph] ) -> (Vec<Graph>, usize) {
        let now = Instant::now();
        let ady_mat = vec![BM::new(); N];
        let mut g = Graph {
            ady_matrix: ady_mat,
        };

        let total_threads = graphs.len();
        let mut threads_done = Arc::new(AtomicUsize::new(0));
        let mut solutions = Arc::new(AtomicUsize::new(0));
        let mut timer = Arc::new(Mutex::new(Instant::now()));
        debug!("Tasks to parallelize:{}", total_threads);
        //println!("Seen vertices:{}", n_k_combinatorial.vertices_seen);
        let all_graphs = graphs.into_par_iter().map(|(mut leaf)|
            {
                // println!("Parallel {i} deployed!");
                let thread_now = Instant::now();
                let mut memory = Memory::new(k, N_CONF + 1, None);
                CanaugFullPoint::canaug_corrected(k, &mut leaf, leaf_level
                    + 1, &mut memory, leaf_level + 1);

                let mut current_threads_done = threads_done.fetch_add(1,SeqCst);
                if print {
                    debug!("\t{current_threads_done} done in {:?}, v: {} l:{}",
                             now.elapsed(), memory.seen_vertices,
                             memory.seen_leaves);
                }

                solutions.fetch_add( memory.seen_leaves,SeqCst);

                if current_threads_done % 1000 == 0 {
                    let secs = timer.lock().unwrap().elapsed().as_secs();
                    let timePerThread = secs as f32 / (current_threads_done) as f32;

                    let remaining = (total_threads - current_threads_done) as f32 * timePerThread ;
                    let remaining_minutes = (remaining) as f32 / 60.0    ;
                    info!("threads done:{:?}, {:.2}% \t secs : {secs} will finish in {remaining_minutes:.2} minutes", threads_done,
                         current_threads_done as f32 * 100.0 / total_threads as f32);
                }


                /*
                println!("%\tl_t\tg_t\tl_s\tg_s\tm_d\n{:.2} \t{}\t{}\t{}\t{}\t{}",
                         *threads_done.lock().unwrap() as f32 * 100.0 / total_threads as f32,
                         thread_now.elapsed().as_secs(),
                         now.elapsed().as_secs(),
                         memory.seen_leaves,
                         *solutions.lock().unwrap(),
                         memory.max_seen_level
                );

                 */

                memory.early_leaves
            }).collect::<Vec<_>>();

        let mut leaves = Vec::new();
        for graphs in all_graphs {
            for graph in graphs {
                leaves.push(graph);
            }
        }
        let solutions = leaves.len();
        debug!("solutions:{}", solutions);

        (leaves,solutions)
    }


}
#[test]
fn test_launch_parallel() {
    init_logger();
    CanaugManager::launch_auto_balanced_parallel(3,false);
}