use std::collections::HashMap;
use log::debug;
use crate::data_structs::bitmaps::BM;
use crate::data_structs::consts::{N, N_CONF};
use crate::data_structs::graph::Graph;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::n_k_combinatorial::canonicalAugmenterTrait::{CanonicalAugmenter, Memory};
use crate::schreier::permute_edges::PermuteEdges;

pub struct SingleEdgeCanaug {
}

impl SingleEdgeCanaug {
    fn get_possible_edges(graph: &mut Graph, k: usize ) -> Vec<(usize, usize)>{
        let available_points = Self::get_available_points_stat(graph,k);
        let available_lines = Self::get_available_lines_stat(graph,k);
        let mut possible_edges = vec![];
        for point in &available_points {
            for line in &available_lines {
                if graph.has_edge(*point,*line){
                    continue;
                }
                graph.add_edge(*point, *line);
                if Self::is_feasible_stat(&graph, *point, [*line].as_slice()){
                    possible_edges.push(( *line, *point ));
                }
                graph.remove_edge(*point, *line);
            }
        }
        possible_edges
    }
}

impl CanonicalAugmenter for SingleEdgeCanaug {
    fn get_subgraphs(graph: &Graph) -> Vec<Graph> {
        let n  = graph.ady_matrix.len();
        let mut subobjects = Vec::with_capacity(n);
        for i in 0..n - 1 {
            for j in i + 1 ..n {
                if graph.has_edge(i, j){
                    let mut subobject = graph.clone();
                    subobject.remove_edge(i, j);
                    subobjects.push(subobject);
                }
            }
        }
        subobjects
    }

    fn get_non_iso_edges(graph: &mut Graph, k: usize) -> Vec<Vec<(usize, usize)>>{
        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(graph).unwrap();
        debug!("current cert: {}", certificate);

        let mut non_iso_edges = Vec::new();
        let possible_edges = Self::get_possible_edges(graph, k);
        if (gen.is_empty()){
            let mut non_iso_edges_map = HashMap::new();
            for  edge in possible_edges {
                graph.add_edge(edge.0, edge.1);
                let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(graph).unwrap();
                graph.remove_edge(edge.0, edge.1);
                let last = non_iso_edges_map.get(&certificate);
                if last.is_none() || last.is_some() && last.unwrap() > &edge {
                    non_iso_edges_map.insert(certificate, edge);
                }
            }

            for (cert, edge) in non_iso_edges_map {
                non_iso_edges.push(vec![edge]);
            }

        } else{
            let permuteEdges = PermuteEdges::from_vertices_range(graph.ady_matrix.len());
            let schreier = crate::schreier::schreier::Schreier::from_generators(gen, permuteEdges);
            let mut orbits = vec![vec![None; N];N];

            for edge in possible_edges {
                if orbits[edge.0][edge.1].is_some() {
                    continue;
                }

                //calculate orbit for current edge
                let data = schreier.get_orbit(edge);


                for element in data.orbit{
                    orbits[element.0][element.1] = Some(data.orbit_representative);
                }
                non_iso_edges.push(vec![edge]);
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
    let k = 3;
    let mut memory = Memory::new(k, N_CONF );

    SingleEdgeCanaug::canaug_corrected(k,&mut g,0, &mut memory, 10);
    let mut total_usage = 0;
    let mut total_number = 0;
    for (level, length) in memory.tree.iter().enumerate() {
        total_usage += memory.cert_usage[level].1;
        total_number += memory.cert_usage[level].0;
        println!("level={:02} length={:} \tusage={:?}", level, length, memory.cert_usage[level].1 as f64/ memory.cert_usage[level].0 as f64);
    }

    println!("Total usage {} total number {} percentage {}" , total_usage, total_number, total_usage as f64 / total_number as f64);
}

#[test]
fn test_graph(){

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let graph = &mut Graph::from_string("1300:500:2600:900:1c00:b000:6800:c200:b:85:16:58:31:64:c0:a0");
    let k = 3;
    let mut memory = Memory::new(k, N_CONF);

    SingleEdgeCanaug::canaug_corrected(k, graph, 0, &mut memory , 10);
}

#[test]
fn test_gen(){
    let graph = &mut Graph::from_string("1300:500:2600:900:1c00:b000:6800:c200:b:85:16:58:31:64:c0:a0");
    println!("{}", graph.custom_print());
    let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(graph).unwrap();
    println!("Gen: {:?}", gen);
    println!("{:?}", certificate);
}