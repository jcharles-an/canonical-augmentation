use rayon::iter::ParallelIterator;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use log::info;
use memmap2::MmapMut;
use nauty_Traces_sys::graph;
use rayon::iter::IntoParallelIterator;
use rayon::iter::IndexedParallelIterator;
use rayon::prelude::{ParallelSlice, ParallelSliceMut};
use crate::data_structs::bitmaps::BM;
use crate::data_structs::cert_graph_file::{CertGraphFile, MmapCertReader};
use crate::data_structs::consts::{BTree, GraphInfo, HMap, SolutionsBTree, N, N_CERT, N_CONF};
use crate::data_structs::graph::Graph;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::n_k_combinatorial::NkCom::{Config, NkComb};
use crate::n_k_combinatorial::NkCombManager;
use crate::schreier::permutation::Permutation;

pub fn get_extended_prefix_b_tree(k: usize, current_vertex: usize, graphsInfo: Vec<GraphInfo>) -> (BTree, BTree) {
    let mut extensions = BTreeMap::new();
    let mut solutions = BTreeMap::new();
    let mut n_k_combinatorial = NkComb::new(k);
    for graphInfo in graphsInfo {
        n_k_combinatorial.add_vertex_to_prefix(graphInfo.cert, graphInfo.gen, graphInfo.graph, current_vertex, &mut extensions, &mut solutions);
    }
    (extensions, solutions)
}

pub fn merge_sorted<T>(mut sorted: Vec<T>) -> Vec<<T as Iterator>::Item>
where
    T: Iterator,
    T::Item: Clone + Ord,
{
    // add a 0 to avoid setting nones, before when ordering
    let mut current_elements: Vec<(usize, Option<T::Item>, usize)> = sorted.iter_mut().enumerate()
        .map(|(index, iterator)| {
            let next = iterator.next();
            (if next.is_none() { 1 } else { 0 }, next, index)
        })
        .collect();

    let mut list = Vec::with_capacity(sorted.len());
    loop {
        let min = current_elements.iter().min();
        let (_, min_val, _) = min.unwrap();
        if min_val.is_none() {
            break;
        }
        let min_val = min_val.clone();
        current_elements = current_elements.into_iter().map(
            |(state, val, index)| {
                if val == min_val {
                    let next = sorted[index].next();
                    return (if next.is_none() { 1 } else { 0 }, next, index);
                }
                (state, val, index)
            }).collect::<Vec<(usize, Option<T::Item>, usize)>>();
        list.push(min_val.unwrap());
    }
    list
}

pub fn get_level(k: usize, current_line: usize) -> HashMap<u128, Graph> {
    let mut n_k_combinatorial = NkComb::new(k);
    n_k_combinatorial.config = Config {
        leaf_level: current_line,
        save_leaves: true,
    };
    n_k_combinatorial.rec_with_memory(current_line);

    return n_k_combinatorial.early_leaves.read().unwrap().clone();
}

pub fn get_level_from_graph(k: usize, current_line: usize, graph: Graph) -> HashMap<u128, Graph> {
    let mut n_k_combinatorial = NkComb::from_graph(k,
                                                   graph,
                                                   Config { leaf_level: current_line, save_leaves: true },
                                                   Arc::new(RwLock::new(HMap::new())),
                                                   Arc::new(RwLock::new(HashMap::new())),
    );
    n_k_combinatorial.rec_over_full_set_with_memory(current_line);

    return n_k_combinatorial.early_leaves.read().unwrap().clone();
}

pub fn launch_parallel_p(k: usize, leaf_level: usize, leaf_level_2: usize, print: bool) -> Duration {
    let now = Instant::now();
    let mut n_k_combinatorial = NkComb::new(k);
    n_k_combinatorial.config = Config {
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
            let mut n_k_combinatorial = NkComb::from_graph(k,
                                                           leaf.clone(),
                                                           Config { leaf_level: leaf_level_2, save_leaves: true },
                                                           Arc::new(RwLock::new(HMap::new())),
                                                           hm.clone(),
            );
            n_k_combinatorial.rec_with_memory(leaf_level + 1);
            {
                let mut i = threads_done.lock().unwrap();
                *i += 1;

                if print {
                    println!("\t{i} done in {:?}, v: {} l:{}",
                             now.elapsed(), n_k_combinatorial.seen_vertices,
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
            let mut n_k_combinatorial = NkComb::from_graph(k,
                                                           leaf.clone(),
                                                           Config { leaf_level: N, save_leaves: false },
                                                           hm.clone(),
                                                           Arc::new(RwLock::new(HashMap::new())));
            n_k_combinatorial.rec(leaf_level_2 + 1);
            {
                let mut i = threads_done.lock().unwrap();
                *i += 1;

                if print {
                    println!("\t{i} done in {:?}, v: {} l:{}",
                             now.elapsed(), n_k_combinatorial.seen_vertices,
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


/**
* launches parallel correct implementation of pizaña et al, only one level
*/
pub fn launch_parallel_full_p(k: usize, leaf_level: usize, leaf_level_2: usize, print: bool) -> Duration {
    let now = Instant::now();
    let mut n_k_combinatorial = NkComb::new(k);
    n_k_combinatorial.config = Config {
        leaf_level,
        save_leaves: true,
    };
    n_k_combinatorial.rec_over_full_set_with_memory(0);
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
            let mut n_k_combinatorial = NkComb::from_graph(k,
                                                           leaf.clone(),
                                                           Config { leaf_level: leaf_level_2, save_leaves: true },
                                                           Arc::new(RwLock::new(HMap::new())),
                                                           hm.clone(),
            );
            n_k_combinatorial.rec_over_full_set_with_memory(leaf_level + 1);
            {
                let mut i = threads_done.lock().unwrap();
                *i += 1;

                if print {
                    println!("\t{i} done in {:?}, v: {} l:{}",
                             now.elapsed(), n_k_combinatorial.seen_vertices,
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
    let elapsed = now.elapsed();

    //println!("Hashmap:{:?}", hm.last().unwrap().read().unwrap());
    println!("effective leaves:{:?}", hm.read().unwrap().len());
    //println!("leaves:{:?}", hm.read().unwrap());
    println!("Elapsed:{:?}", elapsed);
    elapsed
}

/**
* launches parallel correct implementation of pizaña et al, teo parallel levels
*/
pub fn launch_parallel_full_p_2(k: usize, leaf_level: usize, leaf_level_2: usize, print: bool) -> Duration {
    let now = Instant::now();
    let mut n_k_combinatorial = NkComb::new(k);
    n_k_combinatorial.config = Config {
        leaf_level,
        save_leaves: true,
    };
    n_k_combinatorial.rec_over_full_set_with_memory(0);
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
            let mut n_k_combinatorial = NkComb::from_graph(k,
                                                           leaf.clone(),
                                                           Config { leaf_level: leaf_level_2, save_leaves: true },
                                                           Arc::new(RwLock::new(HMap::new())),
                                                           hm.clone(),
            );
            n_k_combinatorial.rec_over_full_set_with_memory(leaf_level + 1);
            {
                let mut i = threads_done.lock().unwrap();
                *i += 1;

                if print {
                    println!("\t{i} done in {:?}, v: {} l:{}",
                             now.elapsed(), n_k_combinatorial.seen_vertices,
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
            let mut n_k_combinatorial = NkComb::from_graph(k,
                                                           leaf.clone(),
                                                           Config { leaf_level: N, save_leaves: false },
                                                           hm.clone(),
                                                           Arc::new(RwLock::new(HashMap::new())));
            n_k_combinatorial.rec_over_full_set_with_memory(leaf_level_2 + 1);
            {
                let mut i = threads_done.lock().unwrap();
                *i += 1;

                if print {
                    println!("\n\t{i} done in {:?}, v: {} l:{}",
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
    println!("Elapsed:{:?} ", elapsed);
    elapsed
}


pub fn manager(n:usize, k:usize, cores: usize){
    let now = Instant::now();
    let ady_mat = vec![BM::new();n];
    let g = Graph {
        ady_matrix: ady_mat,
    };
    let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(&g).unwrap();

    let mut graphs =BTree::new();
    graphs.insert(certificate, (gen, g)) ;
    let mut current_vertex = 0;
    let mut count = 0;
    let mut global_solutions = vec![];

    while !graphs.is_empty() {

        let parallel_load = graphs.into_iter().map(|(cert, (gen, graph))|{
            GraphInfo{cert, gen, graph}
        }).collect::<Vec<_>>();

        let solutions:BTree;
        (graphs, solutions) = get_extended_prefix_b_tree(k,current_vertex,parallel_load);

        println!["{current_vertex},\t{},\t{}",graphs.len(), solutions.len()];
        global_solutions.push(solutions.into_iter().map(|(s, g)| s) );
        current_vertex += 1;
    }
    let elapsed_traversing = now.elapsed();

    let now = Instant::now();
    let sols = merge_sorted(global_solutions);
    let elapsed_ordering = now.elapsed();


    println!["Total solutions found: {}", sols.len()];
    println!["Time traversing: {:?}", elapsed_traversing];
    println!["Time ordering: {:?}", elapsed_ordering];
    println![];
}

pub fn parallel_manager(n:usize, k:usize, cores: usize, save_gen: bool){
    let file_prefix = format!("results/{}_{}", N_CONF, k);
    let now = Instant::now();
    let ady_mat = vec![BM::new();n];
    let g = Graph {
        ady_matrix: ady_mat,
    };
    let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(&g).unwrap();

    let mut graphs:Vec<GraphInfo> = vec![GraphInfo::new(certificate, gen, g)];
    let mut current_vertex = 0;
    let mut count = 0;
    let mut global_solutions = vec![];

    while !graphs.is_empty() {

        //prepare graphs for cores
        let chunk_size = (graphs.len() as f64/ cores as f64).ceil() as usize;
        let chunk_size = 200_000;
        let chunks = (graphs.len() as f64/ chunk_size as f64).ceil() as usize;
        info!("Chunk size: {} chunks: {}", chunk_size, chunks);
        let parallel_loads: Vec<((MmapCertReader,MmapCertReader),(usize,usize))> = graphs.par_chunks(chunk_size).enumerate().map(|(i,chunk)|{

            let now = Instant::now();

            // Get file names
            let g_filename = format!["{file_prefix}/g_l_{current_vertex:02}_c_{cores}_s_{chunk_size}_chunk_{i:03}.bin"];
            let s_filename = format!["{file_prefix}/s_l_{current_vertex:02}_c_{cores}_s_{chunk_size}_chunk_{i:03}.bin"];

            // Search for already created files:
            let level_graphs_result = CertGraphFile::get_list(&g_filename);
            let level_sols_result = CertGraphFile::get_list(&s_filename);

            let mmap :MmapMut;
            let graph_bytes_size :usize;
            let sol_mmap :MmapMut;
            let sol_bytes_size :usize;

            if !(level_graphs_result.is_ok() && level_sols_result.is_ok()) {
                let (graphs, solutions) = get_extended_prefix_b_tree(k,current_vertex,chunk.to_vec());

                (mmap, graph_bytes_size) = CertGraphFile::save_list(graphs, &g_filename, save_gen ).unwrap();
                (sol_mmap, sol_bytes_size) = CertGraphFile::save_list(solutions, &s_filename, false).unwrap();
            } else {
                mmap = level_graphs_result.unwrap();
                graph_bytes_size = mmap.len();
                sol_mmap = level_sols_result.unwrap();
                sol_bytes_size = sol_mmap.len();
            }
            info!["level_{current_vertex:02}_cores_{cores:.03}_chunk_{i:02} done in {:?}!", now.elapsed()];
            ((MmapCertReader::new(mmap), MmapCertReader::new(sol_mmap)), (graph_bytes_size, sol_bytes_size))
        }).collect();
        let (mmaps, sizes) : (Vec<(MmapCertReader, MmapCertReader)>, Vec<(usize,usize)>) = parallel_loads.into_iter().unzip();
        let (graphMmaps, solMmaps) : (Vec<MmapCertReader>, Vec<MmapCertReader>)= mmaps.into_iter().unzip();
        let (graph_bytes, sol_bytes) : (Vec<usize>, Vec<usize>)= sizes.into_iter().unzip();


        info!["level_{current_vertex:02} done! Now merging..."];
        let now = Instant::now();
        graphs = merge_sorted(graphMmaps);
        let solutions = merge_sorted(solMmaps);
        info!["level_{current_vertex:02} done merging in {:?}",now.elapsed()];

        let a:usize = graph_bytes.into_iter().sum();
        let b:usize = sol_bytes.into_iter().sum();
        info!["{current_vertex}\t{}\t{}\t{}\t{}\n",
            graphs.len(),
            solutions.len(),
            a as f64 / 1024.0f64.powi(2),
            b as f64 / 1024.0f64.powi(2)];

        global_solutions.push(solutions.into_iter().map(|graph_info: GraphInfo| graph_info.cert) );
        current_vertex += 1;
    }
    let elapsed_traversing = now.elapsed();

    let now = Instant::now();
    info!["Time traversing: {:?}", elapsed_traversing];


    if (false){
        for (i,sol) in global_solutions.into_iter().enumerate() {
            let so: Vec<u128> = sol.collect();
            info!["{i} sol: {:?}",so];
        }
    } else {
        let sols = merge_sorted(global_solutions);
        let elapsed_ordering = now.elapsed();
        info!["Total solutions found: {}", sols.len()];
        // info!["Total solutions found: {:?}", sols];
        info!["Time ordering: {:?}", elapsed_ordering];
    }
}


#[test]
fn test_get_level() {
    let hm = get_level(4, 0);
    println!("hm: {:?}", hm);
}

#[test]
fn merge_sorted_test(){

    let numstring = "18446741950994212328";
    let num = numstring.parse::<u128>().unwrap();
    println!("num: {}", num);


    let mut lists = vec![
        vec![0,1,3,7].into_iter(),
        vec![0,4,6,7].into_iter(),
        vec![0,2,8].into_iter(),
        vec![6,8].into_iter(),
        vec![9,10,11].into_iter(),
        vec![4].into_iter(),
    ];
    let sorted = merge_sorted(lists);
    println!("sorted: {:?}", sorted);

    let mut lists = vec![
        vec![(938463463374607431768211455u128, "my:graph"),(938469463374607431768211455, "my:graph"),(939463463374607431768211455, "my:graph"),(1938463463374607431768211455, "my:graph"),].into_iter(),
        vec![(463463374607431768211455, "my:graph"),(998463463374607431768211455, "my:graph"),(999463463374607431768211455, "my:graph")].into_iter(),
        vec![(1938463463374607431768211455, "my:graph"),(1938463463374607431768211459, "my:graph"),(1998463463374607431768211455, "my:graph"),(1999463463374607431768211455, "my:graph"),(2999463463374607431768211455, "my:graph"),(9999463463374607431768211455, "my:graph"),].into_iter(),
    ];
    let sorted = merge_sorted(lists);
    println!("sorted: {:?}", sorted);
}

#[test]
fn test_k_comb() {
    launch_parallel_full_p(4,9,N,false);
}
#[test]
fn test_k_comb_2() {

}

#[test]
fn test_manager() {
    /*
    let mut nkComb = NkComb::new(4);
    nkComb.config = Config{leaf_level: N, save_leaves: true};
    let now = Instant::now();
    nkComb.rec_over_full_set_with_memory(0);
    let duration = now.elapsed();
    println!("vertices: {}\nleaves: {}\nsols: {}", nkComb.seen_vertices, nkComb.seen_leaves, nkComb.early_leaves.read().unwrap().len());
    println!("Time elapsed: {:?}", duration);
    println!("TREE: \n");
    for (i, val) in nkComb.tree.iter().enumerate() {
        println!("  {}, {}", i, val);
    }

     */

    // manager(N,4,10);
    parallel_manager(N,4,10,true);
    // parallel_manager(N,4,10,false);
}

