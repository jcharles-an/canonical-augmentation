use std::collections::HashMap;
use std::ops::BitAnd;
use log::debug;
use rayon::iter::MaxLen;
use crate::data_structs::bitmaps::BM;
use crate::data_structs::consts::{N, N_CONF};
use crate::data_structs::graph::Graph;
use crate::data_structs::invariant_chain::InvariantChain;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::schreier::permute_edges::PermuteEdges;



pub struct Memory{
    pub path: Vec<(usize,u128, Graph)>,
    pub tree: Vec<usize>,
    pub cert_usage: Vec<(usize,usize)>,
    pub early_leaves: Vec<Graph>,
    pub seen_leaves: usize,
    pub seen_vertices: usize,
    pub max_seen_level: usize,
}

impl Memory {
    pub fn new(k: usize, max_level: usize) -> Memory {
        let ady_mat = vec![BM::new();N];
        let mut g = Graph {
            ady_matrix: ady_mat,
        };
        let path = vec![(0,0,g.clone());max_level];
        let tree = vec![0; max_level + 1];
        let cert_usage = vec![(0,0); max_level + 1];
        let final_nodes = Vec::new();
        Memory {path,
            tree,
            cert_usage,
            early_leaves: final_nodes,
            seen_leaves: 0 ,
            seen_vertices: 0,
            max_seen_level: 0,
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

    fn is_isomorph_to_canonical(graph: &Graph, subgraphs:  &[Graph] ) -> (bool, bool) {
        let chain = InvariantChain::new();
        let (used_cert, canonical_index) = chain.get_canonical_invariant_chain(subgraphs);

        debug!("father: {}", graph.custom_print());
        for graph in subgraphs{
            debug!("sub: {}", graph.custom_print());
        }
        debug!("");

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

    fn canaug_corrected(k: usize, graph: &mut Graph, level: usize, memory:  &mut Memory, max_level: usize) {

        memory.tree[level] += 1;

        let S = Self::get_available_points_stat(&graph, k);
        if S.is_empty() || level > max_level {
            return Self::process_leaf(graph, level, max_level, memory);
        }
        if level > memory.max_seen_level {
            memory.max_seen_level = level;
        }

        debug!("");
        debug!("canaug starting for level {level}");
        debug!("graph: ");
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
            debug!("Obtaining possible subgraphs...");
            let subgraphs = Self::get_subgraphs(graph);
            graph.remove_edges(edges);

            let (isomorph_to_canonical, used_cert) = Self::is_isomorph_to_canonical(graph, subgraphs.as_slice());

            if used_cert {
                memory.cert_usage[level].1 += 1;
            }
            memory.cert_usage[level].0 += 1;

            if !isomorph_to_canonical {
                continue;
            }


            graph.add_edges(edges);
            let adyMat = graph.ady_matrix.clone();
            let (gen, cert ) = NautyGraph::get_aut_generators_and_cert(graph).unwrap();
            memory.path[level] = (edge_index,cert, Graph{
                ady_matrix: adyMat,
            });
            Self::canaug_corrected(k, graph, level + 1, memory, max_level);
            graph.remove_edges(edges);
        }
    }


}

