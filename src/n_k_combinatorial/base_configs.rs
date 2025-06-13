use crate::data_structs;
use crate::data_structs::bitmaps::*;

pub fn get_n_k_base(n: usize, k: usize) -> Vec<BMConf> {
    match (n,k) { 
        ( 7, 3) => get_7_3_config(),
        ( 8, 3) => get_8_3_config(),
        ( 9, 3) => get_9_3_config(),
        (10, 3) => get_10_3_config(),
        (11, 3) => get_11_3_config(),
        (12, 3) => get_12_3_config(),
        (13, 3) => get_13_3_config(),
        
        (13, 4) => get_13_4_config(),
        (14, 4) => get_14_4_config(),
        (15, 4) => get_15_4_config(),
        (16, 4) => get_16_4_config(),
        (17, 4) => get_17_4_config(),
        _ => Vec::new()
    }
}

fn get_7_3_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0]),
    ]
}
fn get_8_3_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0]),
    ]
}

fn get_9_3_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ]
}

fn get_10_3_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ]
}

fn get_11_3_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ];
    vec![
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ]
}

fn get_12_3_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ]
}

fn get_13_3_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ]
}

fn get_13_4_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ]
}

fn get_14_4_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ]
}
fn get_15_4_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ]
}
fn get_16_4_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ]
}
fn get_17_4_config() -> Vec<BMConf> {
    vec![
        BMConf_from_BE_array(&[1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        BMConf_from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ]
}

