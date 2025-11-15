use rayon::iter::ParallelIterator;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use glob::glob;
use log::info;
use memmap2::{MmapMut, MmapOptions};
use nauty_Traces_sys::graph;
use rayon::iter::IntoParallelIterator;
use rayon::iter::IndexedParallelIterator;
use rayon::prelude::{ParallelSlice, ParallelSliceMut};
use regex::Regex;
use crate::data_structs::bitmaps::BM;
use crate::data_structs::cert_graph::CertGraph;
use crate::data_structs::cert_graph_file::{CertGraphFile, MmapCertReader, SliceReader};
use crate::data_structs::consts::{BTree, GraphInfo, HMap, SolutionsBTree, N, N_CERT, N_CONF};
use crate::data_structs::graph::Graph;
use crate::data_structs::iterator_merger::IteratorMerger;
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

pub fn get_extended_prefix_b_tree_from_bytes(k: usize, current_vertex: usize, graph_bytes: &[u8]) -> (BTree, BTree) {
    let mut extensions = BTreeMap::new();
    let mut solutions = BTreeMap::new();
    let mut n_k_combinatorial = NkComb::new(k);
    let mut sliceReader = SliceReader::new(graph_bytes);

    for graphInfo in sliceReader {
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


pub fn manager(n:usize, k:usize){
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
    let mut current_vertex = 0;


    let last =  glob(format!("./{}/g_*_c_{cores}.bin", file_prefix).as_str()).expect("Failed to read glob pattern").last();
    let mut graphs_mmap = if last.is_some(){
        let res = last.unwrap().unwrap();
        let backing_file = fs::OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .write(true)
            .open(&res).unwrap();

        let filename = res.file_name().unwrap().to_str().unwrap();

        let re = Regex::new(r"g_l_(\d+)_c.*").unwrap();

        // Search the text for a match
        let caps = re.captures(filename).unwrap();
        // The first capture group (index 1) contains the integer string
        // The full match is at index 0
        let int_str = caps.get(1).map(|m| m.as_str()).unwrap();

        current_vertex = int_str.parse::<usize>().unwrap();

        unsafe { MmapMut::map_mut(&backing_file).unwrap() }
    } else {
        CertGraphFile::get_initial_mmap(n)
    };


    let mut sols_mmap = if current_vertex > 0 {
        let sol_file = format!("./{}/s_l_{:02}_c_{cores}.bin", file_prefix, current_vertex);

        let backing_file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .write(true)
            .open(sol_file).unwrap();
        unsafe { MmapMut::map_mut(&backing_file).unwrap() }
    } else {
        CertGraphFile::get_empty_mmap(n)
    };



    let mut graphs_count = CertGraphFile::get_graphs_number(&graphs_mmap);

    while graphs_count > 0 {


        //prepare graphs for cores
        let chunk_size = (graphs_count as f64/ cores as f64).ceil() as usize;
        let chunk_size = std::cmp::min(500_000, chunk_size);
        let chunks = (graphs_count as f64/ chunk_size as f64).ceil() as usize;
        info!("Chunk size: {} chunks: {}", chunk_size, chunks);
        let indexes = CertGraphFile::get_chuck_indexes(&graphs_mmap, chunk_size);


        // launch parallel tasks
        let parallel_loads: Vec<((MmapCertReader,MmapCertReader),(usize,usize))> = indexes.par_windows(2).enumerate().map(|(i,window)|{

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
                // extend the current chunk
                info!["level_{current_vertex:02}_cores_{cores:.03}_chunk_{i:02} extension starting..."];
                let (graphs, solutions) = get_extended_prefix_b_tree_from_bytes(k,current_vertex,&graphs_mmap[window[0]..window[1]]);

                (mmap, graph_bytes_size) = CertGraphFile::save_list(graphs, &g_filename, save_gen ).unwrap();
                (sol_mmap, sol_bytes_size) = CertGraphFile::save_list(solutions, &s_filename, false).unwrap();
            } else {
                info!["level_{current_vertex:02}_cores_{cores:.03}_chunk_{i:02} previous results found!"];
                mmap = level_graphs_result.unwrap();
                graph_bytes_size = mmap.len();
                sol_mmap = level_sols_result.unwrap();
                sol_bytes_size = sol_mmap.len();
            }
            info!["level_{current_vertex:02}_cores_{cores:.03}_chunk_{i:02} done in {:?}!", now.elapsed()];
            ((MmapCertReader::new(mmap), MmapCertReader::new(sol_mmap)), (graph_bytes_size, sol_bytes_size))
        }).collect();

        //Collect parallel tasks resultas
        let (mmaps, sizes) : (Vec<(MmapCertReader, MmapCertReader)>, Vec<(usize,usize)>) = parallel_loads.into_iter().unzip();
        let (graphMmaps, mut solMmaps) : (Vec<MmapCertReader>, Vec<MmapCertReader>)= mmaps.into_iter().unzip();
        let (graph_bytes, sol_bytes) : (Vec<usize>, Vec<usize>)= sizes.into_iter().unzip();


        info!["level_{current_vertex:02} done! Now merging..."];
        let now = Instant::now();

        let g_filename = format!["{file_prefix}/g_l_{current_vertex:02}_c_{cores}.bin"];
        (graphs_mmap, graphs_count) = CertGraphFile::save_and_merge_iter(graphMmaps, &g_filename, save_gen ).unwrap();
        if current_vertex > 0 {
            let g_filename = format!["{file_prefix}/g_l_{:02}_c_{cores}.bin", current_vertex - 1];
            fs::remove_file(g_filename);
        }


        let s_filename = format!["{file_prefix}/s_l_{current_vertex:02}_c_{cores}.bin"];
        solMmaps.push(MmapCertReader::new(sols_mmap));
        let mut sol_count = 0;
        (sols_mmap, sol_count) = CertGraphFile::save_and_merge_iter(solMmaps, &s_filename, save_gen ).unwrap();
        if current_vertex > 0 {
            let s_filename = format!["{file_prefix}/s_l_{:02}_c_{cores}.bin", current_vertex - 1];
            fs::remove_file(s_filename);
        }


        for result in glob(format!("./{}/*chunk*.bin", file_prefix).as_str()).expect("Failed to read glob pattern") {
            fs::remove_file(result.unwrap());
        }

        info!["level_{current_vertex:02} done merging in {:?}",now.elapsed()];

        let a:usize = graph_bytes.into_iter().sum();
        let b:usize = sol_bytes.into_iter().sum();
        info!["{current_vertex}\t{}\t{}\t{}\t{}\n",
            graphs_count,
            sol_count,
            a as f64 / 1024.0f64.powi(2),
            b as f64 / 1024.0f64.powi(2)];
        current_vertex += 1;
    }
    let elapsed_traversing = now.elapsed();

    let now = Instant::now();
    info!["Time traversing: {:?}", elapsed_traversing];


    if (false){
        for (i,sol) in MmapCertReader::new(sols_mmap).into_iter().enumerate() {
            info!["{i} sol: {:?}",sol.cert];
        }
    } else {
        info!["Total solutions found: {}", CertGraphFile::get_graphs_number(&sols_mmap)];
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

