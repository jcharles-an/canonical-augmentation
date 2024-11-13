use crate::schreier::permutation::Permutation;

mod schreier;

fn main() {
    let mut test = vec![1,2];
    let mut iter = test.iter().peekable();
    while let Some(val) = iter.next(){
        if val == &1{
            iter.
        }

    }
    let permutation = Permutation::from_slice_cicle(&[1,2,3,5]);
    println!("{:?}", permutation);
}
