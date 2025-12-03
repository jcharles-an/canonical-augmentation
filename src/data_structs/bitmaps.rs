use bitmaps::{Bitmap, Bits};
use crate::data_structs::consts::{N, N_CERT, N_CONF};

pub type BM64 = Bitmap<64>;
pub type BM = Bitmap<N>;
pub type BMC = Bitmap<N_CERT>;
pub type BMConf = Bitmap<N_CONF>;

fn from_array(bools: &[u8], LE: bool) -> BM{
    let mut bitmap = BM::new();
    let iter : Box<dyn Iterator<Item = &u8>> = if LE {
        Box::new(bools.iter().rev())
    } else {
        Box::new(bools.iter())
    };
    for (i, &b) in iter.enumerate() {
        if b > 0 {
            bitmap.set(i, true);
        }
    }
    bitmap
}

pub fn from_LE_array(bools: &[u8]) -> BM {
    from_array(bools, true)
}

pub fn from_BE_array(bools: &[u8]) -> BM {
    from_array(bools, false)
}

pub fn into_set(set: &BM) -> Vec<usize> {
    let mut v = Vec::with_capacity(set.len());
    for i in set{
        v.push(i);
    }
    v
}


fn BMConf_from_array(bools: &[u8], LE: bool) -> BMConf {
    let mut bitmap = BMConf::new();
    let iter : Box<dyn Iterator<Item = &u8>> = if LE {
        Box::new(bools.iter().rev())
    } else {
        Box::new(bools.iter())
    };
    for (i, &b) in iter.enumerate() {
        if b > 0 {
            bitmap.set(i, true);
        }
    }
    bitmap
}

pub fn BMConf_from_LE_array(bools: &[u8]) -> BMConf {
    BMConf_from_array(bools, true)
}

pub fn BMConf_from_BE_array(bools: &[u8]) -> BMConf {
    BMConf_from_array(bools, false)
}