use bincode::config::Configuration;
use serde::Serialize;
use crate::data_structs::consts::GraphInfo;
use crate::data_structs::graph::Graph;
use crate::data_structs::nauty_graph::NautyGraph;
use crate::schreier::permutation::Permutation;
use crate::schreier::schreier_permutation::Schreier;

pub struct CertGraph {}

impl CertGraph {

    const BINCODE_CONFIG: Configuration = bincode::config::standard().with_variable_int_encoding();
    pub fn to_bytes(btree_node:  (u128, (Vec<Permutation>,Graph))) -> Vec<u8> {
        let cert = btree_node.0;
        let gen = btree_node.1.0;
        let graph = btree_node.1.1;

        let mut key_val = Vec::<u8>::new();

        let cert_bytes = cert.to_le_bytes();
        let cert_len = [cert_bytes.len() as u8]; //this is always 16

        let gen_bytes = bincode::encode_to_vec(gen, Self::BINCODE_CONFIG).unwrap();
        let gen_len:[u8;2] = (gen_bytes.len() as u16).to_le_bytes();


        let graph_bytes = graph.to_bytes();
        let graph_len:[u8;2] = (graph_bytes.len() as u16).to_le_bytes();

        key_val.extend_from_slice(&cert_len);
        key_val.extend_from_slice(&cert_bytes);
        key_val.extend_from_slice(&gen_len);
        key_val.extend_from_slice(&gen_bytes);
        key_val.extend_from_slice(&graph_len);
        key_val.extend(graph.to_bytes());
        key_val
    }

    pub fn from_bytes(bytes: &[u8]) -> GraphInfo{
        let key_len = bytes[0] as usize;
        let key = u128::from_le_bytes(bytes[1..key_len+1].try_into().unwrap());
        let gen_len = u16::from_le_bytes(bytes[key_len + 1..key_len + 3].try_into().unwrap()) as usize;
        let (gen, _ ) = bincode::borrow_decode_from_slice(&bytes[key_len + 3..key_len + 3 + gen_len], Self::BINCODE_CONFIG ).unwrap();
        let val_len = u16::from_le_bytes(bytes[key_len + gen_len + 3..key_len + gen_len + 5].try_into().unwrap()) as usize;
        let val = Graph::from_bytes(&bytes[key_len + gen_len + 5..key_len + gen_len + 5 + val_len]);
        GraphInfo{
            cert: key,
            gen,
            graph: val
        }
    }
}

#[test]
fn test_cert_graph() {
    let graph = Graph::from_string("1c00:6400:18400:a800:800:13000:9000:0:0:0:7:19:61:2a:2:4c:24:0:0:0");

    let (gen, cert) = NautyGraph::get_aut_generators_and_cert(&graph).unwrap();

    let original = (cert,(gen, graph));
    let bytes = CertGraph::to_bytes(original.clone());
    let node = CertGraph::from_bytes(bytes.as_slice());
    println!("original\n{:?}", original);
    println!("bytes\n{:?}", bytes);
    println!("decoded\n{:?}", node);
    assert_eq!(original.0, node.cert);
    assert_eq!(original.1.1.to_string(), node.graph.to_string());
}