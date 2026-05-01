use std::collections::{HashMap, HashSet};
use std::ops::BitAnd;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Release};
use log::{debug, info, trace};
use rayon::current_thread_index;
use rayon::iter::MaxLen;
use crate::data_structs::bitmaps::BM;
use crate::data_structs::consts::{METHOD, N, N_CONF};
use crate::data_structs::graph::Graph;
use crate::data_structs::invariant_chain::InvariantChain;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::group_actions::permute_edges::PermuteEdges;



pub struct Memory{
    pub path: Vec<(usize,u128, Graph)>,
    pub tree: Vec<usize>,
    pub cert_usage: Vec<(usize,usize)>,
    pub early_leaves: Vec<Graph>,
    pub thread_redistribution_needed: bool,
    pub thread_redistribution_check: usize,
    pub active_treads: Option<Arc<RwLock<HashSet<usize>>>>,
    pub min_active_threads: usize,
    pub seen_leaves: usize,
    pub seen_vertices: usize,
    pub max_seen_level: usize,
}

impl Memory {
    pub fn new(k: usize, max_level: usize, active_threads: Option<Arc<RwLock<HashSet<usize>>>>) -> Memory {
        let ady_mat = vec![BM::new();N];
        let mut g = Graph {
            ady_matrix: ady_mat,
        };
        let path = vec![(0,0,g.clone());max_level];
        let tree = vec![0; max_level + 1];
        let cert_usage = vec![(0,0); max_level + 1];
        let final_nodes = Vec::new();
        Memory {
            path,
            tree,
            cert_usage,
            early_leaves: final_nodes,
            seen_leaves: 0 ,
            seen_vertices: 0,
            max_seen_level: 0,
            thread_redistribution_needed: false,
            thread_redistribution_check: 500,
            active_treads: active_threads,
            min_active_threads: 4
        }
    }
}

pub trait CanonicalAugmenter{

    fn get_subgraphs(graph: &Graph) -> Vec<Graph>;
    fn get_non_iso_edges(graph: &mut Graph, k: usize) -> Vec<Vec<(usize, usize)>>;

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

    // TODO: vec<vec<usize>> could be vec<Bitmap(32)>
    fn get_available_lines_partitions(graph: &Graph, k: usize) -> Vec<Vec<usize>> {
        let n = graph.ady_matrix.len();

        let mut neighborhoods = graph.ady_matrix.iter().
            enumerate().
            take(n/2).
            filter(|&(_,neighbors)| neighbors.len() < k).
            map(|(i,neighbors)| (neighbors, i)).collect::<Vec<(&BM, usize)>>();

        neighborhoods.sort();

        // create index partitions from sorted neighborhoods
        let mut last_neighborhood = neighborhoods[0].0;
        let mut partitions = vec![];
        let mut current_partition = vec![];
        for (neighborhood, idx) in neighborhoods {
            if (neighborhood != last_neighborhood){
                // save current partition
                partitions.push(current_partition.clone());
                last_neighborhood = neighborhood;
                //Start new partition
                current_partition.clear();
                current_partition.push(idx);
            } else {
                current_partition.push(idx);
            }
        }

        // save last partition
        partitions.push(current_partition.clone());

        partitions
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

    fn is_isomorph_to_canonical(graph: &Graph, subgraphs:  &[Graph]) -> (bool, bool) {
        let chain = InvariantChain::new(0);
        //let (used_cert, canonical_index) = chain.get_full_canonical_invariant_chain(subgraphs);
        let (used_cert, canonical_index) = chain.get_graphs_isomorph_to_canonical(subgraphs);

        //info!("Debugging with certificates: ");
        //let (used_cert_bis, canonical_index_bis) = chain.get_full_canonical_invariant_chain(subgraphs);


        trace!("father: {}", graph.custom_print());
        for graph in subgraphs{
            trace!("sub: {}", graph.custom_print());
        }
        trace!("");

        // get real parent index inside subgraphs
        let mut graph_index= None;
        for (index, subgraph) in subgraphs.iter().enumerate() {
            if subgraph == graph {
                graph_index = Some(index);
                break
            }
        }
        //info!("Canonical indexes: {:?}", canonical_index);
        //info!("Full would choose : {:?}\n\n", canonical_index_bis);
        let result = canonical_index.contains(&graph_index.unwrap());
        if result{
            debug!("Found that p(X) is isomorph to m(X) with index {}",graph_index.unwrap())
        } else {
            debug!("p(X) is not isomorph to m(X) index {} NOT FOUND",graph_index.unwrap());
        }
        (result, used_cert)
    }

    fn process_leaf(graph: &Graph, level: usize, max_level: usize,memory: &mut Memory){
        memory.seen_leaves  += 1;
        if level > max_level {
            memory.early_leaves.push(graph.clone());
        }

        if std::env::var("RUST_LOG") == Ok("debug".to_string()) {
            let (gen, certificate) = NautyGraph::get_aut_generators_and_cert(graph).unwrap();
            debug!("{certificate} -> {}", graph);
            debug!("{level}:");
            for step in &memory.path {
                debug!("\t{}\t{}\t{}", step.0, step.1, step.2);
            }
        }
        return;
    }

    fn process_autobalance_leaf(graph: &Graph, level: usize, max_level: usize, found_leaves: &mut usize){

        *found_leaves += 1;
        let thread_id = current_thread_index().expect("Must be run inside a Rayon thread.");
        //info!("[{thread_id}] local solutions: {:?}", found_leaves);

        //info!("current sols: {current}");
        if std::env::var("RUST_LOG") == Ok("debug".to_string()) {
            let (gen, certificate) = NautyGraph::get_aut_generators_and_cert(graph).unwrap();
            debug!("{certificate} -> {}", graph);
            debug!("{level}:");
        }
    }

    fn canaug_corrected(k: usize, graph: &mut Graph, level: usize, memory:  &mut Memory, max_level: usize) {

        memory.tree[level] += 1;

        let S = Self::get_available_points_stat(graph, k);
        if S.is_empty() || level > max_level {
            return Self::process_leaf(graph, level, max_level, memory);
        }

        if let Some(idle) = &memory.active_treads {
            if memory.thread_redistribution_needed {
                memory.early_leaves.push(graph.clone());
                return;
            }
            if memory.seen_vertices > memory.thread_redistribution_check {
                memory.seen_vertices = 0;
                if idle.read().unwrap().len() < memory.min_active_threads {
                    memory.thread_redistribution_needed = true;
                }
            }
        }
        if level > memory.max_seen_level {
            memory.max_seen_level = level;
        }

        debug!("");
        debug!("canaug starting for level {level}");
        debug!("graph: {}", graph);
        //Graph::print_ady_mat(&graph.ady_matrix);

        let mut non_iso_edges = Self::get_non_iso_edges(graph, k);
        //print_nested_optional_vec(&orbits);
        debug!("non_iso_edges for level {level}: {:?}", non_iso_edges);


        /*
        debug!("$$ Certificates for level {level}:");
        for (edge_index, edges) in non_iso_edges.iter().enumerate() {
            graph.add_edges(edges);
            let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(graph).unwrap();
            graph.remove_edges(edges);
            debug!("edges{edges:?} -> certificate:{certificate}");
        }
         */



        // Now we search in non iso (X)
        for (edge_index, edges) in non_iso_edges.iter().enumerate() {
            debug!("$$ Extending for level {level} -> edge {edges:?}");
            graph.add_edges(edges);
            debug!("\nObtaining possible subgraphs...");
            let subgraphs = Self::get_subgraphs(graph);
            graph.remove_edges(edges);

            debug!("\nChecking if father is iso to canonical...");
            let (isomorph_to_canonical, used_cert) = Self::is_isomorph_to_canonical(graph, subgraphs.as_slice(), );

            if used_cert {
                memory.cert_usage[level].1 += 1;
            }
            memory.cert_usage[level].0 += 1;

            if !isomorph_to_canonical {
                continue;
            }


            graph.add_edges(edges);
            let adyMat = graph.ady_matrix.clone();
            /*
            let (gen, cert ) = NautyGraph::get_aut_generators_and_cert(graph).unwrap();
            memory.path[level] = (edge_index,cert, Graph{
                ady_matrix: adyMat,
            });
             */
            Self::canaug_corrected(k, graph, level + 1, memory, max_level);
            graph.remove_edges(edges);
        }
    }

    fn scoped_canaug(
        k: usize, mut graph: Graph, level: usize, max_level: usize, interval: usize,
        active_threads: &Arc<AtomicUsize>,
        num_total_threads: usize,
        found_leaves: &Arc<AtomicUsize>,
        local_found_leaves: &mut usize,
        processed_nodes: &mut usize,
        is_new_thread: bool,
        mut thread_id: usize,
        offload_work: &mut bool,
        active_count: &mut usize,
        s: &rayon::Scope,
    ){
        // This block adds the new thread to the active pool
        if is_new_thread {
            thread_id = current_thread_index().expect("Must be run inside a Rayon thread.");
            //info!("[{thread_id}] started new thread");
            active_threads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        //info!("[{thread_id}] local solutions: {:?}", local_found_leaves);


        let S = Self::get_available_points_stat(&graph, k);
        if S.is_empty() || level > max_level {
            //active_threads.write().unwrap().remove(&index);
            let current_found_leaves = found_leaves.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            //info!("[{thread_id}] New solution found, solution number: {:?}", current_found_leaves);
            Self::process_autobalance_leaf(&graph, level, max_level, local_found_leaves);
            //info!("[{thread_id}] local solutions mutated to: {:?}", local_found_leaves);
            return;
        }

        debug!("");
        debug!("canaug starting for level {level}");
        debug!("graph: {}", graph);
        //Graph::print_ady_mat(&graph.ady_matrix);

        let mut non_iso_edges = Self::get_non_iso_edges(&mut graph, k);
        //print_nested_optional_vec(&orbits);
        debug!("non_iso_edges for level {level}: {:?}", non_iso_edges);



        /* debugging, uncomment to check each certificate
        debug!("$$ Certificates for level {level}:");
        for (edge_index, edges) in non_iso_edges.iter().enumerate() {
            graph.add_edges(edges);
            let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(&graph).unwrap();
            graph.remove_edges(edges);
            debug!("edges{edges:?} -> certificate:{certificate}");
        }
         */

        if *processed_nodes % interval == 0 {
            // Monitoring requires read Lock
            // We check the count right after inserting our own index
            *active_count = active_threads.load(std::sync::atomic::Ordering::SeqCst);
            debug!("[{thread_id}] processed nodes: {processed_nodes:?} active threads: {:?}/{num_total_threads}", active_count);

            // Check for under-utilization
            *offload_work = *active_count < num_total_threads
        }


        // Now we search in non iso (X)
        for (edge_index, edges) in non_iso_edges.iter().enumerate() {
            debug!("$$ Extending for level {level} -> edge {edges:?}");
            graph.add_edges(edges);
            debug!("\nObtaining possible subgraphs/fathers...");
            let subgraphs = Self::get_subgraphs(&graph);
            graph.remove_edges(edges);

            debug!("\nChecking if father is iso to canonical...");
            let (isomorph_to_canonical, used_cert) = Self::is_isomorph_to_canonical(&graph, subgraphs.as_slice());

            if !isomorph_to_canonical {
                continue;
            }


            graph.add_edges(edges);
            let adyMat = graph.ady_matrix.clone();
            let ( gen, cert ) = NautyGraph::get_aut_generators_and_cert(&graph).unwrap();

            if *offload_work {
                //send to stack
                debug!("[{thread_id}] [WARN] Node detected imbalance: {}/{} active.", active_count, num_total_threads);
                let graph_copy = graph.clone();
                let active_threads_copy = active_threads.clone();
                let found_leaves_copy = found_leaves.clone();
                s.spawn(move |inner| {
                    let mut processed_nodes = 0;
                    let mut local_found_leaves = 0;
                    let mut offload_work = false;
                    let mut active_count = 0;
                    Self::scoped_canaug(k, graph_copy, level, max_level, interval,&active_threads_copy, num_total_threads,  &found_leaves_copy, &mut local_found_leaves ,  &mut processed_nodes,true,0, &mut offload_work,&mut active_count, inner);

                });
            } else {
                // call in the same thread
                *processed_nodes += 1;
                Self::scoped_canaug(k, graph.clone(), level + 1, max_level, interval, active_threads, num_total_threads, found_leaves, local_found_leaves , processed_nodes, false,thread_id, offload_work,active_count, s);
               // info!("[{thread_id}] finished son");
            }
            graph.remove_edges(edges);
        }

        // This block deletes the thread from the active pool, as it has already finished its work
        if is_new_thread{
            //info!("[{thread_id}] finished thread sols {local_found_leaves}");
            active_threads.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            //found_leaves.fetch_add(*local_found_leaves, std::sync::atomic::Ordering::SeqCst);
        }
    }


}

