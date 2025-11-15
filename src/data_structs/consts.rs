use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use crate::data_structs::graph::Graph;
use crate::schreier::permutation::Permutation;

include!(concat!(env!("OUT_DIR"), "/consts.rs"));


pub const DEGREE: usize = 4;
//pub const N: usize =  1 << DEGREE;

pub const N_CONF: usize = 7;
pub const N: usize =  N_CONF * 2;
pub const N_CERT: usize = N * (N - 1) / 2;

pub const STORE_BYTES: usize = match N {
    N if N <= 8 => 1,
    N if N <= 16 => 2,
    N if N <= 32 => 4,
    N if N <= 64 => 8,
    _ => 16
};

pub type HMap = HashMap<u128, bool>;


#[derive(Debug, Clone)]
pub struct GraphInfo{
    pub cert: u128,
    pub gen: Vec<Permutation>,
    pub graph: Graph
}


impl PartialEq for GraphInfo {
    fn eq(&self, other: &Self) -> bool {
        self.cert == other.cert
    }
}

impl Eq for GraphInfo {}

impl PartialOrd for GraphInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cert.partial_cmp(&other.cert)
    }
}

impl Ord for GraphInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cert.cmp(&other.cert)
    }
}

impl GraphInfo{
    pub fn new(cert: u128, gen: Vec<Permutation>, graph: Graph) -> GraphInfo{
        GraphInfo{cert, gen, graph}
    }
}


pub type BTree = BTreeMap<u128, (Vec<Permutation>,Graph)>;
pub type SolutionsBTree = BTreeMap<u128, Graph>;
