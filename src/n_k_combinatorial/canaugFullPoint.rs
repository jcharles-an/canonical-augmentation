use std::collections::HashMap;
use log::debug;
use crate::data_structs::bitmaps::BM;
use crate::data_structs::consts::{N, N_CONF};
use crate::data_structs::graph::Graph;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::n_k_combinatorial::canonicalAugmenterTrait::{CanonicalAugmenter, Memory};
use crate::schreier::permute_edges::PermuteEdges;
use crate::schreier::schreier_permutation::{SChooseKInG, Schreier};

pub struct CanaugFullPoint {
}
impl CanonicalAugmenter for CanaugFullPoint {
    fn get_subgraphs(graph: &Graph) -> Vec<Graph> {
        let n  = graph.ady_matrix.len();
        let mut subobjects = Vec::with_capacity(n/2);
        for i in n/2..n {
            if graph.ady_matrix[i].is_empty() {
                continue;
            }
            let mut subobject = graph.clone();
            for neighbor in &graph.ady_matrix[i] {
                subobject.remove_edge(i, neighbor);
            }
            subobjects.push(subobject);
        }
        subobjects
    }

    fn get_non_iso_edges(graph: &mut Graph, k: usize) -> Vec<Vec<(usize, usize)>>{
        let n  = graph.ady_matrix.len();
        let mut non_iso_edges = Vec::new();

        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(graph).unwrap();
        debug!("current cert: {}", certificate);
        let schreier = Schreier::from_generators(gen);

        // Set all the iterators for the non-isomorphic point candidates
        //let non_iso_S = self.get_non_isomorfic_available_points(&schreier,S.as_slice());
        let available_points = Self::get_available_points_stat(graph,k);
        let available_lines = Self::get_available_lines_stat(graph,k);
        let available_points_representative = available_points[0];

        let data = schreier.get_orbit(available_points_representative, n);
        let schreier_for_stab = Schreier::from_generators(data.stab_generators);
        let s_choose_k_in_g_iter = SChooseKInG::new(schreier_for_stab, n, available_lines.as_slice(), k - graph.ady_matrix[available_points_representative].len());

        let mut cert_map = HashMap::new();

        for ends in s_choose_k_in_g_iter {
            for end in &ends{
                graph.add_edge(available_points_representative,*end);
            }

            if Self::is_feasible_stat(graph,available_points_representative,ends.as_slice()){
                let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(graph).unwrap();
                if cert_map.get(&certificate).is_some() {
                    for end in &ends{
                        graph.remove_edge(available_points_representative,*end);
                    }
                    continue;
                }

                cert_map.insert(certificate, true);
                let edges = ends.iter().map(|end| {(available_points_representative, *end)}).collect();
                non_iso_edges.push(edges);
            }

            for end in &ends{
                graph.remove_edge(available_points_representative,*end);
            }


        }
        non_iso_edges
    }
}


#[test]
fn test_canaug(){
    let ady_mat = vec![BM::new();N];
    let mut g = Graph {
        ady_matrix: ady_mat,
    };
    let k = 4;
    let mut memory = Memory::new(k, N_CONF);

    CanaugFullPoint::canaug_corrected(k,&mut g,0, &mut memory, 10);

    println!("Leaves: {}", memory.seen_leaves);
    return;
    let mut total_usage = 0;
    let mut total_number = 0;
    for (level, length) in memory.tree.iter().enumerate() {
        total_usage += memory.cert_usage[level].1;
        total_number += memory.cert_usage[level].0;
        println!("level={:02} length={:} \tusage={:?}", level, length, memory.cert_usage[level].1 as f64/ memory.cert_usage[level].0 as f64);
    }

    println!("Total usage {} total number {} percentage {}" , total_usage, total_number, total_usage as f64 / total_number as f64);


    let k = 3;
    let mut graph = Graph::from_string("180:280:80:300:100:200:0:7:19:2a:0:0:0:0");

    let edge_sets = CanaugFullPoint::get_non_iso_edges(&mut graph,k);

    println!("{}", graph.custom_print());

    for edge_set in &edge_sets {
        graph.add_edges(edge_set);
        let (gen, cert) = NautyGraph::get_aut_generators_and_cert(&graph).unwrap();
        print!("{:?}\n", edge_set);
        println!( "{cert} -> {graph}");
        println!("{}", graph.custom_print());


        graph.remove_edges(edge_set);

    }
}



#[test]
fn test_get_non_iso_edges(){
    let k = 3;
    let ady_mat = vec![BM::new();N];
    let mut graph = Graph{
        ady_matrix: ady_mat,
    };
    println!("{}", graph.custom_print());
    let edges = CanaugFullPoint::get_non_iso_edges(&mut graph,k);
    println!("{:?}", edges);

    graph.add_edges(edges[0].as_slice());

    let subgraphs = CanaugFullPoint::get_subgraphs(&mut graph);

    for subgraph in subgraphs {
        println!("{}", subgraph.custom_print());
    }
}

#[test]
fn test_s_choose_k_in_g(){
    let k = 3;
    let mut graph = Graph::from_string("180:280:80:300:100:200:0:7:19:2a:0:0:0:0");

    let edge_sets = CanaugFullPoint::get_non_iso_edges(&mut graph,k);

    println!("{:?}", edge_sets);

    for edge_set in &edge_sets {
        graph.add_edges(edge_set);
        let (gen, cert) = NautyGraph::get_aut_generators_and_cert(&graph).unwrap();
        println!( "{cert} -> {graph}");


        graph.remove_edges(edge_set);

    }

}