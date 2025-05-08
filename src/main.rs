#![allow(dead_code)]
#![allow(unused)]
#![allow(non_snake_case)]
#![allow(refining_impl_trait)]

use crate::data_structs::graph::Graph;
use crate::schreier::permutation::Permutation;
use crate::schreier::stabilizer_chain::RecStabilizerChain;

mod schreier;
mod data_structs;

fn main(){
    env_logger::init();


    let mut aut = crate::schreier::automorphism::Automorphism::new(
        Graph {
            ady_matrix: vec![
                crate::data_structs::bitmaps::from_BE_array(&[0,1,0,1,0,0,0,1]),
                crate::data_structs::bitmaps::from_BE_array(&[1,0,1,0,1,0,0,0]),
                crate::data_structs::bitmaps::from_BE_array(&[0,1,0,1,0,0,1,0]),
                crate::data_structs::bitmaps::from_BE_array(&[1,0,1,0,1,0,0,0]),
                crate::data_structs::bitmaps::from_BE_array(&[0,1,0,1,0,1,0,0]),
                crate::data_structs::bitmaps::from_BE_array(&[0,0,0,0,1,0,1,1]),
                crate::data_structs::bitmaps::from_BE_array(&[0,0,1,0,0,1,0,1]),
                crate::data_structs::bitmaps::from_BE_array(&[1,0,0,0,0,0,1,0]),
            ],
        }
    );

    let C = aut.cert_1();
    println!("Cert: {:}", C.into_value());
    println!("mu: {:?}", aut.mu);

    let C = aut.cert_2();
    println!("Cert: {:}", C.into_value());
    println!("mu: {:?}", aut.mu);
}

fn main_rubik() {

    let sc = RecStabilizerChain::get_stabilizer_chain(vec![
        Permutation::new(vec![1, 2, 3, 0]),
        Permutation::new(vec![3, 2, 1, 0]),
    ]);

    println!("C_4 order: {}\n\n",sc.get_order());




    let sc = RecStabilizerChain::get_stabilizer_chain(vec![
        Permutation::from_cycles(vec![vec![0, 1, 2]]),
        Permutation::from_cycles(vec![vec![1, 2, 3]]),
    ]);
    println!("Example order: {}",sc.get_order());

    let r = Permutation::new(vec![0, 1, 3, 2]);
    let belongs = sc.test_belonging(&r);
    println!("Does {r:?} belong to example? {belongs}");
    let r = Permutation::new(vec![1, 3, 2, 0]);
    let belongs = sc.test_belonging(&r);
    println!("Does {r:?} belong to example? {belongs}");
    println!("\n\n");




    let u =  Permutation::from_cycles(vec![
        vec![0, 2, 7, 5],
        vec![1, 4, 6, 3],
        vec![8,32,24,16],
        vec![9,33,25,17],
        vec![10,34,26,18],
    ]);

    let l = Permutation::from_cycles(vec![
        vec![8,10,15,13],
        vec![9,12,14,11],
        vec![0,16,40,39],
        vec![3,19,43,36],
        vec![5,21,45,34],
    ]);
    let f = Permutation::from_cycles(vec![
        vec![16,18,23,21],
        vec![17,20,22,19],
        vec![5,24,42,15],
        vec![6,27,41,12],
        vec![7,29,40,10],
    ]);
    let r = Permutation::from_cycles(vec![
        vec![24,26,31,29],
        vec![25,28,30,27],
        vec![2,37,42,18],
        vec![4,35,44,20],
        vec![7,32,47,23],
    ]);
    let b = Permutation::from_cycles(vec![
        vec![32,34,39,37],
        vec![33,36,38,35],
        vec![2, 8,45,31],
        vec![1,11,46,28],
        vec![0,13,47,26],
    ]);
    let d = Permutation::from_cycles(vec![
        vec![40,42,47,45],
        vec![41,44,46,43],
        vec![13,21,29,37],
        vec![14,22,30,38],
        vec![15,23,31,39],
    ]);
    let sc = RecStabilizerChain::get_stabilizer_chain(vec![u, l, f, r, d]);


    let order = sc.get_order();
    println!("rubik order according to literature: \n43252003274489856000\n",);
    println!("rubik order: \n{}",order);
}
