use std::cmp;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Chain;
use std::sync::OnceLock;
use env_logger::Builder;
use log::{debug, info, LevelFilter};
use crate::data_structs;
use crate::data_structs::bitmaps::BM;
use crate::data_structs::consts::{METHOD, N};
use crate::data_structs::graph::Graph;
use crate::data_structs::logger::init_logger;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::n_k_combinatorial::NkCom::{NkComb};

static CUSTOM_CHAIN: OnceLock<Vec<usize>> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct InvariantChainValues {
    connected_components: Option<usize>,
    degeneracy: Option<usize>,
    degree_sequence: Option<Vec<usize>>,
    certificate: Option<u128>,
}
pub struct InvariantChain {
    pub links: usize,
    max_links: usize,
}

impl InvariantChain {
    pub fn new(links: usize) -> InvariantChain {
        InvariantChain{links, max_links: 4}
    }

    pub fn set_custom_chain(mut chain: Vec<usize>) {
        if !chain.contains(&3) {
            chain.push(3);
        }
        let _ = CUSTOM_CHAIN.set(chain);
    }

    pub fn get_chain() -> &'static [usize] {
        CUSTOM_CHAIN.get().map(|v| v.as_slice()).unwrap_or(&[0, 1, 2, 3])
    }

    fn fill_invariant_value(link: usize, graph: &Graph, values: &mut InvariantChainValues) {
        match link {
            //0 => values.connected_components = Some(Self::get_connected_components(graph)),
            0 => values.connected_components = Some(Self::nr_connected_components(graph)),
            1 => values.degeneracy = Some(Self::get_degeneracy(graph)),
            2 => values.degree_sequence = Some(Self::get_degree_sequence(graph)),
            3 => values.certificate = Some(Self::get_certificate(graph)),
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


    fn connected_component(m: &[BM], root: usize) -> BM {
        let mut visited = BM::new();
        let mut frontier = BM::new();
        frontier.set(root, true);
        while let Some(visiting) = frontier.first_index() {
            frontier.set(visiting, false);
            visited.set(visiting, true);
            let visiting_neighbors = m[visiting];
            let new_discovered = visiting_neighbors & (!(visited | frontier));
            frontier |= new_discovered;
        }
        visited
    }

    fn nr_connected_components(graph: &Graph) -> usize {
        let n = graph.ady_matrix.len();
        let mut res = 0;
        let mut union = BM::new();
        let mut all_vertices = BM::new();
        for i in 0..n {
            all_vertices.set(i, true);
        }
        loop {
            let not_considered = all_vertices & (!union);
            let root = match not_considered.first_index() {
                Some(r) => r,
                None => break,
            };
            let new_component = Self::connected_component(graph.ady_matrix.as_slice(), root);
            res += 1;
            union |= new_component;
        }
        res
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

            //get vertex and turn it off
            let vertex = (&buckets[min_degree]).into_iter().next().unwrap();
            buckets[min_degree].set(vertex, false);

            // delete edges from neighborhood
            for neighbor in &graph.ady_matrix[vertex] {
                let old_deg = degrees[neighbor];

                if buckets[old_deg].get(neighbor) {
                    buckets[old_deg].set(neighbor, false);
                    degrees[neighbor] -= 1;
                    let new_deg = degrees[neighbor];
                    buckets[new_deg].set(neighbor, true);
                    if new_deg < min_degree {
                        min_degree = new_deg;
                    }
                }
            }
        }


        max_degree
    }

    fn get_certificate(graph: &Graph) -> u128 {
        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(graph).unwrap();
        certificate
    }

    pub fn get_graphs_isomorph_to_canonical(&self, subgraphs: &[Graph]) -> (bool, Vec<usize>) {
        match METHOD {
            0 => self.get_least_certificate(subgraphs),
            1 => self.get_unique_or_least_isomorphs(subgraphs),
            2 => self.get_least_fast_invariant_chain(subgraphs),
            3 => self.get_least_full_invariant_chain(subgraphs),
            _ => panic!("Not implemented"),
        }

    }

    pub fn get_least_certificate(&self, subgraphs: &[Graph]) -> (bool, Vec<usize>) {
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
        // info!("Least cert: {:?}", least_cert);

        (true, certificates.get(&least_cert).unwrap().clone())
    }

    pub fn get_unique_or_least_isomorphs(&self, subgraphs: &[Graph]) -> (bool, Vec<usize>) {
        let chain = Self::get_chain();
        // vec containing all subgraph invariants
        let mut invariants = vec![InvariantChainValues::default(); subgraphs.len()];

        // for each link in the chain (except terminal cert link)
        for &chain_index in chain.iter().filter(|&&link| link != 3) {

            // get invariant link for all subgraphs
            for (graph_index, graph) in subgraphs.iter().enumerate() {
                Self::fill_invariant_value(chain_index, graph, &mut invariants[graph_index]);
            }

            // count how many times each invariant is repeated and which graphs have them
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

            // check which invariants are present only once
            let mut canonical_candidates = Vec::new();
            for (invariant, graph_indices) in &invariant_count {
                if graph_indices.len() == 1 {
                    canonical_candidates.push((invariant, graph_indices[0]));
                }
            }

            if !canonical_candidates.is_empty() {
                canonical_candidates.sort();
                debug!("found single at chain_index: {:?}", chain_index);
                return (false, vec![canonical_candidates.first().unwrap().1])
            }

        }
        debug!("Went to certificates fallback ");

        self.get_least_certificate(subgraphs)
    }


    pub fn get_least_full_invariant_chain(&self, subgraphs: &[Graph]) -> (bool, Vec<usize>) {
        let chain = Self::get_chain();
        // vec containing all subgraph invariants
        let mut invariants = vec![InvariantChainValues::default(); subgraphs.len()];
        for &chain_index in chain {
            for (graph_index, graph) in subgraphs.iter().enumerate() {
                Self::fill_invariant_value(chain_index, graph, &mut invariants[graph_index]);
            }
        }

        let minInvariant = invariants.iter().min().unwrap();
        //info!("minInvariant: {:?}", minInvariant);

        //for (idx,invariant) in invariants.iter().enumerate() {
        //    info!("\t{:?} -> {:?}", idx, invariant);
        //}

        let indexes = invariants.iter().enumerate().filter(|(idx,invariant)| invariant==&minInvariant ).map(|(idx,invariant)| idx).collect::<Vec<usize>>();

        return (true, indexes);
    }


    /*
    This method will calculate the full chain in order. If it finds any smaller cert in the candidates it will update
    returns the indexes of the subgraph slice which correspond to the objects isomorph to the canonical one
     */
    pub fn get_least_fast_invariant_chain(&self, subgraphs: &[Graph]) -> (bool, Vec<usize>) {
        let chain = Self::get_chain();
        let mut invariants = vec![InvariantChainValues::default(); subgraphs.len()];
        let mut candidates = (0..subgraphs.len()).collect::<Vec<_>>();

        for &chain_index in chain {
            if candidates.is_empty() {
                break;
            }

            let mut filtered_candidates = vec![];
            let first_candidate = candidates[0];
            Self::fill_invariant_value(chain_index, &subgraphs[first_candidate], &mut invariants[first_candidate]);
            let mut least_invariant = invariants[first_candidate].clone();
            filtered_candidates.push(first_candidate);

            // Compare with the rest of the candidates
            for &graph_index in candidates.iter().skip(1) {
                let graph = &subgraphs[graph_index];
                Self::fill_invariant_value(chain_index, graph, &mut invariants[graph_index]);
                let current_invariant = &invariants[graph_index];

                if current_invariant < &least_invariant {
                    least_invariant = current_invariant.clone();
                    filtered_candidates.clear();
                    filtered_candidates.push(graph_index);
                } else if current_invariant == &least_invariant {
                    filtered_candidates.push(graph_index);
                }
            }

            if filtered_candidates.len() == 1 {
                return (chain_index == 3, filtered_candidates);
            }
            candidates = filtered_candidates;
        }
        (true, candidates)
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

    let index =  chain.get_unique_or_least_isomorphs(&subgraphs);
    println!("{:?}", index);

    let chain = InvariantChain::new(0);

    let index =  chain.get_unique_or_least_isomorphs(&subgraphs);
    println!("{:?}", index);

    let chain = InvariantChain::new(4);

    let index =  chain.get_least_fast_invariant_chain(&subgraphs);
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