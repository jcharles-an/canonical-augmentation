use std::collections::HashMap;
use std::io::Lines;
use log::debug;
use crate::data_structs::bitmaps::{slice_into_BM64, BM64_as_set, BM};
use crate::data_structs::consts::{N, N_C, N_CONF};
use crate::data_structs::graph::Graph;
use crate::data_structs::logger::init_logger;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::n_k_combinatorial::canaugSingleEdge::SingleEdgeCanaug;
use crate::n_k_combinatorial::canonicalAugmenterTrait::{CanonicalAugmenter, Memory};
use crate::groups::group::Group;
use crate::group_actions::groupAction::GroupAction;
use crate::group_actions::permute_edges::PermuteEdges;
use crate::group_actions::permute_subset::PermuteSubset;
use crate::schreier::schreier_permutation::{SChooseKInG, Schreier};

pub struct CanaugFullPoint {
}


impl CanaugFullPoint {

    fn get_feasible_ends_for_line(graph: &mut Graph, k: usize , start:usize, available_lines: &[usize]) -> Vec<Vec<usize>> {
        let mut possible_ends = Schreier::get_S_choose_k(N,available_lines,k);
        let mut feasible_ends = vec![];
        for ends in possible_ends {
            let ends = BM64_as_set(&ends);
            for end in &ends {
                graph.add_edge(start, *end);
            }
            if Self::is_feasible_stat(&graph, start, ends.as_slice()){
                feasible_ends.push(ends.clone());
            }
            for end in &ends {
                graph.remove_edge(start, *end);
            }
        }
        feasible_ends
    }



    fn get_non_iso_edges_bis(graph: &mut Graph, k: usize) -> Vec<Vec<(usize, usize)>>{
        let n  = graph.ady_matrix.len();
        let mut non_iso_edges = Vec::new();

        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(graph).unwrap();
        debug!("current cert: {}", certificate);

        // Set all the iterators for the non-isomorphic point candidates
        //let non_iso_S = self.get_non_isomorfic_available_points(&schreier,S.as_slice());
        let available_points = Self::get_available_points_stat(graph,k);
        let available_lines = Self::get_available_lines_stat(graph,k);
        let available_points_representative = available_points[0];

        let feasible_ends = Self::get_feasible_ends_for_line(graph, k, available_points_representative, available_lines.as_slice());
        let schreier = Schreier::from_generators(gen);
        let data = schreier.get_orbit(available_points_representative, n);
        let identity = schreier.group.get_identity();
        let stabilizer_is_trivial = data.stab_generators.iter().all(|x| &identity == x);

        if (stabilizer_is_trivial){
            let mut non_iso_ends_map = HashMap::new();
            for ends in feasible_ends {
                for end in &ends {
                    graph.add_edge(available_points_representative, *end);
                }
                let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(graph).unwrap();
                for end in &ends {
                    graph.remove_edge(available_points_representative, *end);
                }
                let last = non_iso_ends_map.get(&certificate);
                if last.is_none() || last.is_some() && last.unwrap() > &ends {
                    non_iso_ends_map.insert(certificate, ends);
                }
            }

            for (cert, ends) in non_iso_ends_map {
                non_iso_edges.push(
                    ends.iter().
                        map(|x| (available_points_representative, *x)).
                        collect::<Vec<(usize, usize)>>()
                );
            }

        } else{

            let permuteSet = PermuteSubset::new(graph.ady_matrix.len(), k);
            let mut orbits = vec![None; permuteSet.get_range()];

            let schreier = crate::schreier::schreier::Schreier::from_generators(data.stab_generators, permuteSet);

            for ends in feasible_ends {
                let set = slice_into_BM64(ends.as_slice());
                let set_rank = schreier.action.get_index_from_target(&set);
                if orbits[set_rank].is_some() {
                    continue;
                }

                //calculate orbit for current edge
                let data = schreier.get_orbit(set);


                for element in data.orbit{
                    let element_index = schreier.action.get_index_from_target(&element);
                    orbits[element_index] = Some(data.orbit_representative);
                }

                non_iso_edges.push(
                    ends.iter().
                    map(|x| (available_points_representative, *x)).
                    collect::<Vec<(usize, usize)>>()
                );
            }
        }
        non_iso_edges
    }


    fn get_non_iso_edges_n_choose_k_in_g(graph: &mut Graph, k: usize) -> Vec<Vec<(usize, usize)>>{
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

        let identity = schreier.group.get_identity();
        let stabilizer_is_trivial = data.stab_generators.iter().all(|x| &identity == x);
        let schreier_for_stab = Schreier::from_generators(data.stab_generators);

        let s_choose_k_in_g_iter = SChooseKInG::new(schreier_for_stab, n, available_lines.as_slice(), k - graph.ady_matrix[available_points_representative].len());

        // RECURSIVE FOR DEBUGGING
        //let s_choose_k_in_g_iter = Schreier::get_S_choose_k_in_G(schreier_for_stab, n, available_lines.as_slice(), k - graph.ady_matrix[available_points_representative].len()).into_iter();

        //let s_choose_k_in_g_iter = Schreier::get_S_choose_k_in_G_2(schreier_for_stab, n, available_lines.as_slice(), k).into_iter();



        let mut cert_map = HashMap::new();

        for ends in  s_choose_k_in_g_iter {
            for end in &ends{
                graph.add_edge(available_points_representative,*end);
            }

            if Self::is_feasible_stat(graph,available_points_representative,ends.as_slice()) {
                let (gen, certificate) = NautyGraph::get_aut_generators_and_cert(graph).unwrap();
                if cert_map.contains_key(&certificate) {
                    for end in &ends {
                        graph.remove_edge(available_points_representative, *end);
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
       Self::get_non_iso_edges_n_choose_k_in_g(graph, k)
    }


}


#[test]
fn test_canaug(){
    init_logger();

    let ady_mat = vec![BM::new();N];
    let mut g = Graph {
        ady_matrix: ady_mat,
    };
    let k = 3;
    let mut memory = Memory::new(k, N_CONF, None);

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

#[test]
fn test_s_choose_k_in_g_fix_bug(){
    let k = 3;
    // N= 9 -> cause: trivial stabilizer
    let mut graph = Graph::from_string("2a00:5200:8200:c00:3400:4400:8800:1000:e000:7:38:49:92:111:122:144:0:0");;
    // N= 10
    let mut graph = Graph::from_string("c00:2400:8400:4800:8800:3000:5000:1000:a000:4000:7:19:e0:122:248:114:0:0:0:0");

    let edge_sets = CanaugFullPoint::get_non_iso_edges(&mut graph,k);


    for edge_set in &edge_sets {
        graph.add_edges(edge_set);
        let (gen, cert) = NautyGraph::get_aut_generators_and_cert(&graph).unwrap();
        println!( "{edge_set:?}\t{cert}\t{graph}");


        graph.remove_edges(edge_set);

    }

}

#[test]
fn test_s_choose_k_in_g_2(){
    let k = 3;
    let ady_mat = vec![BM::new();N];
    let mut graph = Graph{
        ady_matrix: ady_mat,
    };

    let edge_sets = CanaugFullPoint::get_non_iso_edges_bis(&mut graph,k);
    println!("{:?}", edge_sets);

    let edge_sets = CanaugFullPoint::get_non_iso_edges_n_choose_k_in_g(&mut graph,k);
    println!("{:?}", edge_sets);

    for edge_set in &edge_sets {
        graph.add_edges(edge_set);
        let (gen, cert) = NautyGraph::get_aut_generators_and_cert(&graph).unwrap();
        println!( "{cert} -> {graph}");


        graph.remove_edges(edge_set);

    }

}