use std::fmt::Display;
use libc::signal;
use nauty_Traces_sys::{empty_graph, graph, ADDONEEDGE, SETWORDSNEEDED};
use petgraph::graph::NodeIndex;
use petgraph::Undirected;
use crate::{data_structs};
use crate::data_structs::bitmaps::{BM, BMC, BMConf};
use crate::data_structs::consts::{DEGREE, N, N_CONF, STORE_BYTES};
use crate::data_structs::nauty_graph::NautyGraph;

#[derive(Clone,Debug,Ord, PartialOrd, Eq, PartialEq)]
pub struct Graph {
    pub ady_matrix: Vec<BM>,
}

impl Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let format = self.ady_matrix.iter().fold("".to_string(), |acc, x| { format!("{acc}:{x:?}")});

        write!(f, "{}",&format[1..])
    }
}


impl Graph {

    /** encoding format is 1 byte for length, then 'length' bytes for value eg
    * 1c00 -> 02 00 1c (two bytes, then the two bytes for value)
    * 18400 -> 03 00 84 01
    */
    pub fn to_bytes(&self) -> Vec<u8> {
        let bytes = self.ady_matrix.iter().fold(Vec::new(), |mut acc, x| {

            let empty_bytes = (x.as_value().leading_zeros() / 8) as usize;
            let effective_length = STORE_BYTES - empty_bytes; // for bitmaps of size less than 256 this can perfectly fit in a u8

            acc.extend_from_slice(&[effective_length as u8]);
            acc.extend_from_slice(x.as_bytes()[0..effective_length].as_ref());
            acc
        });
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Graph {
        let mut ady_matrix = Vec::with_capacity(N);
        let mut offset = 0;
        while offset < bytes.len() {
            let length = bytes[offset] as usize;
            let mut vec = bytes[offset + 1 .. offset + 1 + length].to_vec();
            offset += 1 + length;
            vec.resize(STORE_BYTES, 0);

            ady_matrix.push(BM::try_from(vec.as_slice()).unwrap());
        }
        Graph{
            ady_matrix
        }
    }

    pub fn is_set_dominant(&self, set: &BM) -> bool {
        for list in self.ady_matrix.iter() {
            if (*list & *set).len() == 0 {
                return false;
            }
        }
        true
    }

    pub fn is_set_separating(&self, set: &BM) -> bool {
        for (i, u) in self.ady_matrix.iter().enumerate() {
            for v in self.ady_matrix.iter().skip(i + 1) {
                if (*u & *set) == (*v & *set) {
                    return false;
                }
            }
        }
        true
    }

    fn get_num(ady_matrix: Vec<BM>)-> BMC{
        let mut num = BMC::new();
        let mut lower_limit = 1;
        let len = ady_matrix.len();
        let mut initial = 0;
        for n in (0..len).rev() {
            //println!("{:07b} -> {}", n.into_value(), n.into_value());
            for i in lower_limit..len{
                num.set(initial, ady_matrix[n].get(i));
                initial += 1;
            }
            //println!("{:021b}", num.into_value());
            lower_limit += 1;
        }
        num
    }

    pub fn transpose(ady_matrix: &[BMConf]) -> Vec<BMConf> {
        let mut transposed_matrix: Vec<BMConf> = vec![BMConf::new(); ady_matrix.len()];
        for (i,row) in ady_matrix.iter().enumerate(){
            for bit_index in row {
                transposed_matrix[bit_index].set(i,true);
            }
        }
        transposed_matrix
    }

    pub fn print_ady_mat(ady_mat: &[BM]){
        for num in ady_mat{
            let normal = format!("{:016b}", num.into_value());
            // Reverse the bits in the string
            let reversed: String = normal.chars().rev().map(|c| "  ".to_string() + &c.to_string()).collect();
            println!("{}", reversed);
        }

    }

    pub fn print_ady_mat_BMConf(ady_mat: &[BMConf]){
        for num in ady_mat{
            let normal = format!("{:09b}", num.into_value());
            // Reverse the bits in the string
            let reversed: String = normal.chars().rev().map(|c| "  ".to_string() + &c.to_string()).collect();
            println!("{}", reversed);
        }
    }
    
    
    pub fn add_edge(&mut self, v: usize,w: usize){
        self.ady_matrix[v].set(w,true);
        self.ady_matrix[w].set(v,true);
    }
    pub fn remove_edge(&mut self, v: usize,w: usize){
        self.ady_matrix[v].set(w,false);
        self.ady_matrix[w].set(v,false);
    }
    
    pub fn has_edge(&self, v: usize,w: usize) -> bool {
        self.ady_matrix[v].get(w) && self.ady_matrix[w].get(v)
    }


    pub fn to_pet_graph(&self) -> petgraph::Graph::<(), (), Undirected> {
        let mut g = petgraph::Graph::new_undirected();
        let n = self.ady_matrix.len();
        for _ in 0..n {
            g.add_node(());
        }
        for (l,line) in self.ady_matrix.iter().enumerate(){
            for p in line {
                g.add_edge(NodeIndex::new(p), NodeIndex::new(l),());
            }
        }
        g
    }

    pub fn to_pet_levy_graph(&self) -> petgraph::Graph::<usize, (), Undirected> {
        let mut g = petgraph::Graph::new_undirected();
        let n = self.ady_matrix.len();
        for _ in 0..n/2 {
            g.add_node(0);
        }
        for _ in n/2..n {
            g.add_node(1);
        }
        for (l,line) in self.ady_matrix.iter().enumerate().skip(n/2) {
            for p in line {
                g.add_edge(NodeIndex::new(p), NodeIndex::new(l),());
            }
        }
        g
    }
    
    pub fn custom_print(&self) -> String {
        let graph = self.to_pet_graph();
        let mut edge_list: Vec<(usize, usize)> = Vec::new();

        for edge_idx in graph.edge_indices() {
            let (u, v) = graph.edge_endpoints(edge_idx).unwrap();
            let u_idx = u.index(); // Get the integer index of the node
            let v_idx = v.index(); // Get the integer index of the node

            // Normalize the edge pair: always (smaller_index, larger_index) for undirected graphs.
            // This is important for consistent sorting and to avoid duplicate entries for the same edge.
            let mut undirected_edge;
            if u_idx < v_idx {
                undirected_edge = (u_idx, v_idx);
            } else {
                undirected_edge = (v_idx, u_idx);
            }

            if !edge_list.contains(&undirected_edge) {
                edge_list.push(undirected_edge);
            }
        }

        //    b. Sort the collected edge list.
        //       This will sort primarily by the first element (smaller vertex),
        //       and then by the second element (larger vertex) if the first elements are equal.
        edge_list.sort_unstable(); // `sort_unstable` is often faster if order of equal elements doesn't matter

        // If you explicitly wanted to sort only by the smaller vertex and *then* by something else
        // or didn't care about the second vertex, you'd use `sort_by_key`:
        // edge_list.sort_by_key(|&(u, _)| u);

        // 3. Manually construct the DOT string
        let mut dot_output = String::new();
        dot_output.push_str("graph G {\n");
        dot_output.push_str("  node [shape=circle];\n"); // Optional: common node styling

        // Add nodes explicitly. petgraph NodeIndex starts from 0 up to node_count - 1
        //for i in 0..graph.node_count() {
        //    dot_output.push_str(&format!("  {};\n", i));
        //}

        // Add edges from the sorted list
        for (u, v) in edge_list.iter() {
            dot_output.push_str(&format!("  {} -- {};\n", u, v));
        }

        dot_output.push_str("}\n");
        dot_output
    }

    pub fn from_string(graph_string: &str) -> Graph {
        let mut ady_matrix = Vec::with_capacity(N);
        for (i, val) in graph_string.split(":").enumerate() {
            ady_matrix.push(BM::from_value(u16::from_str_radix(val,16).unwrap().into()));
        }
        Graph{
            ady_matrix
        }
    }


    
}


pub fn get_n_cube_ady_matrix() -> Vec<BM> {
    let degree = DEGREE;
    let n = N;
    let mut ady_matrix = vec![BM::new();n];
    for v in 0..n {
        ady_matrix[v].set(v, true);
        for i in 0..degree {
            let neighbor = v ^ (1 << i);
            ady_matrix[v].set(neighbor, true);
            ady_matrix[neighbor].set(v, true);
        }
    }
    ady_matrix
}


#[test]
fn graph_test() {
    let mut graph = Graph { ady_matrix: vec![
        data_structs::bitmaps::from_BE_array(&[1,1,0,0,0,1]),
        data_structs::bitmaps::from_BE_array(&[1,1,1,0,0,0]),
        data_structs::bitmaps::from_BE_array(&[0,1,1,1,0,0]),
        data_structs::bitmaps::from_BE_array(&[0,0,1,1,1,0]),
        data_structs::bitmaps::from_BE_array(&[0,0,0,1,1,1]),
        data_structs::bitmaps::from_BE_array(&[1,0,0,0,1,1]),
    ] };

    assert_eq!(graph.is_set_dominant(&data_structs::bitmaps::from_BE_array(&[1,1,0,0,0,0])), false);
    assert_eq!(graph.is_set_dominant(&data_structs::bitmaps::from_BE_array(&[1,1,1,1,1,1])), true);
    assert_eq!(graph.is_set_dominant(&data_structs::bitmaps::from_BE_array(&[0,1,0,1,0,1])), true);
    assert_eq!(graph.is_set_dominant(&data_structs::bitmaps::from_BE_array(&[1,0,1,0,1,0])), true);
    assert_eq!(graph.is_set_dominant(&data_structs::bitmaps::from_BE_array(&[1,0,0,1,0,0])), true);

    /*
    assert_eq!(graph.is_set_separating(&data_structs::bitmaps::from_array(&[1,1,0,0,0,0])), false);
    assert_eq!(graph.is_set_separating(&data_structs::bitmaps::from_array(&[1,1,1,1,1,1])), true);
    assert_eq!(graph.is_set_separating(&data_structs::bitmaps::from_array(&[0,1,0,1,0,1])), true);
    assert_eq!(graph.is_set_separating(&data_structs::bitmaps::from_array(&[1,0,1,0,1,0])), true);

&     */
    //assert_eq!(graph.is_set_separating(&data_structs::bitmaps::from_array(&[1,0,0,1,0,0])), true);


    let d = Graph::get_num( vec![
        data_structs::bitmaps::from_BE_array(&[0,0,0,0,0,0,1]),
        data_structs::bitmaps::from_BE_array(&[0,0,0,0,1,1,0]),
        data_structs::bitmaps::from_BE_array(&[0,0,0,1,0,1,0]),
        data_structs::bitmaps::from_BE_array(&[0,0,1,0,1,1,1]),
        data_structs::bitmaps::from_BE_array(&[0,1,0,1,0,1,1]),
        data_structs::bitmaps::from_BE_array(&[0,1,1,1,1,0,1]),
        data_structs::bitmaps::from_BE_array(&[1,0,0,1,1,1,0]),
    ]);

    println!("{:?}", d.into_value());

}

#[test]
fn test_parsing(){
    let graph_string = "1e000:0:0:0:0:0:0:0:0:0:0:0:0:1:1:1:1:0:0:0:0:0:0:0:0:0";
    let g = Graph::from_string(graph_string);
    let g_1 = format!("{}", g).trim().to_string();
    println!("{}", g_1);

    assert_eq!(graph_string, &g_1);
}

#[test]
fn encodings(){
}


#[test]
fn test_bytes(){
    let mut graph = Graph { ady_matrix: vec![
        data_structs::bitmaps::from_BE_array(&[1, 1, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[1, 1, 1, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 1, 1, 1, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 1, 1, 1, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 1, 1, 1]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 1, 1]),
    ] };

    println!("{:?}", graph);

   let bytes = graph.to_bytes();
    println!("{:?}", bytes);

    let graph_from = Graph::from_bytes(&bytes);
    println!("{:?}", graph_from);

}

#[test]
fn nauty(){
    let mut graph = Graph { ady_matrix: vec![
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ] };

    let nauty_graph = NautyGraph::get_aut_generators_and_cert(&graph).unwrap();
}