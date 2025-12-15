use std::cmp;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Chain;
use env_logger::Builder;
use log::{debug, LevelFilter};
use crate::data_structs;
use crate::data_structs::bitmaps::BM;
use crate::data_structs::consts::N;
use crate::data_structs::graph::Graph;
use crate::data_structs::logger::init_logger;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::n_k_combinatorial::NkCom::{NkComb};


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct InvariantChainValues {
    connected_components: Option<usize>,
    degeneracy: Option<usize>,
    degree_sequence: Option<Vec<usize>>,
}
pub struct InvariantChain {
    pub links: usize,
}

impl InvariantChain {
    pub fn new(links: usize) -> InvariantChain {
        InvariantChain{links}
    }

  fn fill_invariant_value(index: usize, graph: &Graph, values: &mut InvariantChainValues) {
      match index {
          0 => values.connected_components = Some(Self::get_connected_components(graph)),
          1 => values.degeneracy = Some(Self::get_degeneracy(graph)),
          2 => values.degree_sequence = Some(Self::get_degree_sequence(graph)),
          _ => {}
      }
  }

    fn get_degree_sequence(graph: &Graph) -> Vec<usize> {
        let mut degree_sequence = graph.ady_matrix.iter().map(|x| x.len()).collect::<Vec<usize>>();
        degree_sequence.sort();
        degree_sequence
    }
    fn get_connected_components(graph: &Graph) -> usize {
        let mut connected_components = vec![];
        let mut visited = vec![false; graph.ady_matrix.len()];
        for i in 0..graph.ady_matrix.len() {
            if !visited[i] {
                connected_components.push(Self::get_connected_component(graph, i, &mut visited));
            }
        }
        connected_components.len()
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
    fn get_degeneracy(graph: &Graph) -> usize {
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
        max_degree
    }

    fn get_certificate(graph: &Graph) -> u128 {
        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(graph).unwrap();
        certificate
    }

    pub fn get_canonical_invariant_chain(&self, subgraphs: &[Graph]) -> (bool, Vec<usize>) {
        if self.links > 0 {
            // vec containing all subgraph invariants
            let mut invariants = vec![InvariantChainValues::default(); subgraphs.len()];
            for chain_index in 0..self.links {
                for (graph_index, graph) in subgraphs.iter().enumerate() {
                    Self::fill_invariant_value(chain_index, graph, &mut invariants[graph_index]);
                }
            }

            let mut invariant_count: HashMap<InvariantChainValues, Vec<usize>> = HashMap::new();
            let mut least_cert = invariants[0].clone();
            for (graph_index, invariant_vec) in invariants.iter().enumerate() {

                if invariant_vec < &least_cert {
                    least_cert = invariant_vec.clone();
                }

                let certificate_indexes = invariant_count.get_mut(&invariant_vec);
                if let Some(indexes) = certificate_indexes {
                    indexes.push(graph_index);
                } else{
                    invariant_count.insert(invariant_vec.clone(), vec![graph_index]);
                }
            }
            debug!("invariant Count:");
            for (cert, vec) in &invariant_count {
                debug!("\t{:?} -> {:?}", cert, vec);
            }

            return (false, invariant_count.get(&least_cert).unwrap().clone())
        }


        // run logic to get from multiple chains
        let mut certificates: HashMap<u128,Vec<usize>> = HashMap::new();
        let mut least_cert = u128::MAX;
        for (graph_index, graph) in subgraphs.iter().enumerate() {
            let invariant = Self::get_certificate(graph);

            if invariant < least_cert {
                least_cert = invariant;
            }

            let certificate_indexes = certificates.get_mut(&invariant);
            if let Some(indexes) = certificate_indexes {
                indexes.push(graph_index);
            } else{
                certificates.insert(invariant, vec![graph_index]);
            }
        }

        debug!("Went to certificates fallback: ");
        for (cert, vec) in &certificates {
            debug!("\t{:?} -> {:?}", cert, vec);
        }

        (true, certificates.get(&least_cert).unwrap().clone())
    }


}

#[test]
fn test_invariant_chain(){
    init_logger();
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
    //      0
    //     /|\
    //    1 2 3
    //   /| | |\
    //  4 5 6 7 8 9
    // /  | | |   |
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

    let chain = InvariantChain::new(0);

    let index =  chain.get_canonical_invariant_chain(&subgraphs);
    println!("{:?}", index);

    let chain = InvariantChain{links:0};

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
    assert_eq!(invariant, N);

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