use std::collections::HashMap;

pub const DEGREE: usize = 4;
//pub const N: usize =  1 << DEGREE;

pub const N_CONF: usize = 9;
pub const N: usize =  N_CONF * 2;
pub const N_CERT: usize = N * (N - 1) / 2;

pub type HMap = HashMap<String, bool>;
