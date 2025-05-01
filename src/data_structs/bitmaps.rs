use bitmaps::Bitmap;

pub const DEGREE: usize = 8;
pub const N: usize = 8;

pub const N_CERT: usize = N * (N - 1) / 2;
pub type BM = Bitmap<N>;

pub type BMC = Bitmap<N_CERT>;

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