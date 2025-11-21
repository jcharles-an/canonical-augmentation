use std::cmp;
use std::collections::HashMap;
use std::io::Chain;
use crate::data_structs;
use crate::data_structs::bitmaps::BM;
use crate::data_structs::consts::N;
use crate::data_structs::graph::Graph;
use crate::data_structs::invariant_chain::Invariant::U128;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::n_k_combinatorial::NkCom::{NkComb};


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Invariant {
    USize(usize),
    U128(u128),
    Tuple(i32, i32),
    VecUSize(Vec<usize>),
}

pub struct InvariantChain {
    links: Vec<(&'static str, fn(&Graph) -> Invariant)>,
}

impl InvariantChain {
    pub fn new() -> InvariantChain {
        InvariantChain{links: vec![
            ("connected_components", Self::get_connected_components as fn(&Graph) -> Invariant),
            ("degeneracy", Self::get_degeneracy as fn(&Graph) -> Invariant),
            ("degree_sequence", Self::get_degree_sequence as fn(&Graph) -> Invariant),
        ]}
    }

    fn get_degree_sequence(graph: &Graph) -> Invariant {
        let mut degree_sequence = graph.ady_matrix.iter().map(|x| x.len()).collect::<Vec<usize>>();
        degree_sequence.sort();
        Invariant::VecUSize(degree_sequence)
    }
    fn get_connected_components(graph: &Graph) -> Invariant {
        let mut connected_components = vec![];
        let mut visited = vec![false; graph.ady_matrix.len()];
        for i in 0..graph.ady_matrix.len() {
            if !visited[i] {
                connected_components.push(Self::get_connected_component(graph, i, &mut visited));
            }
        }
        Invariant::USize(connected_components.len())
    }
    fn get_connected_component(graph: &Graph, start: usize, visited: &mut Vec<bool>) -> Vec<usize> {

        let mut connected_component = vec![];
        for neighbor in &graph.ady_matrix[start] {
            let has_been_visited = visited[neighbor];
            visited[neighbor] = true;
            if !has_been_visited {
                Self::get_connected_component(graph, neighbor, visited);
            }
            connected_component.push(neighbor);
        }
        connected_component
    }
    fn get_degeneracy(graph: &Graph) -> Invariant {
        let n = graph.ady_matrix.len();
        let mut buckets = vec![BM::new(); n];  // n buckets for degrees 0 to n-1
        let mut degrees = vec![0; n];
        let mut max_degree = 0;
        let mut min_degree = usize::MAX;

        for i in 0..n {
            let degree = graph.ady_matrix[i].len();
            degrees[i] = degree;
            buckets[degree].set(i, true);
            min_degree = cmp::min(min_degree, degree);
        }

        for i in 0..n {
            while min_degree < n && buckets[min_degree].is_empty() {
                min_degree += 1;
            }
            if min_degree >= n {
                break;  // All vertices processed
            }
            max_degree = cmp::max(max_degree, min_degree);
            let vertex = (&buckets[min_degree]).into_iter().next().unwrap();
            buckets[min_degree].set(vertex, false);

            for neighbor in &graph.ady_matrix[vertex] {
                buckets[degrees[neighbor]].set(neighbor, false);
                degrees[neighbor] -= 1;
                buckets[degrees[neighbor]].set(neighbor, true);
            }
        }
        Invariant::USize(max_degree)
    }

    fn get_certificate(graph: &Graph) -> Invariant {
        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(graph).unwrap();
        U128(certificate)
    }

    pub fn get_canonical_invariant_chain(&self, subgraphs: &[Graph]) -> Vec<usize> {
        let mut invariants = vec![vec![]; subgraphs.len()];
        for (chain_index, (func_name, link)) in self.links.iter().enumerate() {
            let mut invariant_count = HashMap::new();

            for (graph_index, graph) in subgraphs.iter().enumerate() {
                let invariant = link(graph);
                invariants[graph_index].push(invariant.clone());
                let count = invariant_count.get(&invariant).unwrap_or(&(0,0)).0 + 1;
                invariant_count.insert(invariant, (count, graph_index));
            }

            let mut canonical_candidates = Vec::new();
            for (invariant, (count, graph_index)) in invariant_count {
                if count == 1 {
                    canonical_candidates.push((invariant, graph_index));
                }
            }

            println!("Chain link [{}]: {}", chain_index, func_name);
            for (graph_idx, invariant) in invariants.iter().enumerate() {
                println!("  Graph {} -> {:?}", graph_idx, invariant);
            }

            if !canonical_candidates.is_empty() {
                canonical_candidates.sort();
                return vec![canonical_candidates.first().unwrap().clone().1]
            }
        }
        // run logic to get from multiple chains
        let mut certificates: HashMap<Invariant,Vec<usize>> = HashMap::new();
        let mut least_cert = U128(u128::MAX);
        for (graph_index, graph) in subgraphs.iter().enumerate() {
            let invariant = Self::get_certificate(graph);

            if invariant > least_cert {
                continue;
            }
            least_cert = invariant;

            let certificate_indexes = certificates.get_mut(&least_cert);
            if certificate_indexes.is_some() {
                certificate_indexes.unwrap().push(graph_index);
            } else{
                certificates.insert(least_cert.clone(), vec![graph_index]);
            }
        }

        println!("Went to certificates fallback: \n {:?}", certificates);

        certificates.get(&least_cert).unwrap().clone()

    }


}

#[test]
fn test_invariant_chain(){
    let mut ady_mat = vec![
        data_structs::bitmaps::from_BE_array(&[0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),

    ];
    let mut g_star = Graph {
        ady_matrix: ady_mat,
    };

    // Path Graph (Linear chain: 0-1-2-3-...-13) - 13 edges
    let mut ady_mat_path = vec![
        data_structs::bitmaps::from_BE_array(&[0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0]),
    ];
    let mut g_path = Graph {
        ady_matrix: ady_mat_path,
    };

    // Balanced Tree - 13 edges
    //     0
    //    /|\
    //   1 2 3
    //  /| | |\
    // 4 5 6 7 8 9
    // /       | |  |   |
    let mut ady_mat_tree = vec![
        data_structs::bitmaps::from_BE_array(&[0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]),
    ];
    let mut g_tree = Graph {
        ady_matrix: ady_mat_tree,
    };

    // Ring with Branches - 13 edges
    // Ring: 0-1-2-3-4-5-0 (6 edges)
    // Branches: 0-6, 1-7, 2-8, 3-9, 4-10, 5-11, 5-12, 5-13 (7 edges)
    let mut ady_mat_ring = vec![
        data_structs::bitmaps::from_BE_array(&[0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
    ];
    let mut g_ring = Graph {
        ady_matrix: ady_mat_ring,
    };
    let mut ady_mat = vec![BM::new();N];
    let mut g_empty = Graph {
        ady_matrix: ady_mat,
    };

    //let subgraphs = vec![g_path,g_tree,g_star,g_ring];
    let subgraphs = vec![g_path,g_tree,g_ring];

    let chain = InvariantChain::new();

    let index =  chain.get_canonical_invariant_chain(&subgraphs);
    println!("{:?}", index);

}

#[test]
fn test_invariants(){
    let mut ady_mat = vec![BM::new();N];
    let mut g = Graph {
        ady_matrix: ady_mat,
    };
    let invariant = InvariantChain::get_connected_components(&g);
    println!("{:?}", invariant);
    assert_eq!(invariant, Invariant::USize(N));

    let mut ady_mat = vec![
        data_structs::bitmaps::from_BE_array(&[0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),

    ];
    let mut g = Graph {
        ady_matrix: ady_mat,
    };
    let invariant = InvariantChain::get_connected_components(&g);
    println!("{:?}", invariant);

    let invariant = InvariantChain::get_degeneracy(&g);
    println!("{:?}", invariant);

}