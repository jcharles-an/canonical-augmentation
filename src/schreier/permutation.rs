use std::cmp;

#[derive(Debug,Clone,PartialEq)]
pub struct Permutation {
    pub map: Vec<usize>,
    pub size: usize
}
impl Permutation{
    pub fn new(map: Vec<usize>) -> Permutation{
        Permutation{size: map.len(), map}
    }

    pub fn from_cycles(cicles: Vec<Vec<usize>>) -> Permutation{
        let mut tuples: Vec<Vec<(usize, usize)>> = Vec::new();
        let mut max_cycle_value = 0usize;
        for cicle in cicles.iter(){
            let mut tuple: Vec<(usize, usize)> = Vec::with_capacity(cicle.len());
            max_cycle_value = cmp::max(max_cycle_value, *cicle.iter().max().unwrap());
            for window in cicle.windows(2){
                tuple.push((window[0], window[1]));
            }
            tuple.push((*cicle.last().unwrap(), cicle[0]));
            tuples.push(tuple);
        }
        let mut permutation: Vec<usize> = (0..max_cycle_value+1).into_iter().collect();
        for tuple in tuples.iter(){
            for i in 0..tuple.len(){
                permutation[tuple[i].0] = tuple[i].1;
            }
        }
        Permutation::new(permutation)
    }
}

#[test]
fn permutation_test() {
    let permutation = Permutation::from_cycles(vec![vec![0,1,2]]);
    assert_eq!(permutation.map, vec![1,2,0]);

    let permutation = Permutation::from_cycles(vec![vec![1,2,3]]);
    assert_eq!(permutation.map, vec![0,2,3,1]);
    let permutation = Permutation::from_cycles(vec![
        vec![0, 2, 7, 5],
        vec![1, 4, 6, 3],
        vec![8,32,24,16],
        vec![9,33,25,17],
        vec![10,34,26,18],
    ]);
    let permutation = Permutation::from_cycles(vec![
        vec![8,10,15,13],
        vec![9,12,14,11],
        vec![0,16,40,39],
        vec![3,19,43,36],
        vec![5,21,45,34],
    ]);
    let permutation = Permutation::from_cycles(vec![
        vec![16,18,23,21],
        vec![17,20,22,19],
        vec![5,24,42,15],
        vec![6,27,41,12],
        vec![7,29,40,10],
    ]);
    let permutation = Permutation::from_cycles(vec![
        vec![24,26,31,29],
        vec![25,28,30,27],
        vec![2,37,42,18],
        vec![4,35,44,20],
        vec![7,32,47,23],
    ]);
    let permutation = Permutation::from_cycles(vec![
        vec![32,34,39,37],
        vec![33,36,38,35],
        vec![2, 8,45,31],
        vec![1,11,46,28],
        vec![0,13,47,26],
    ]);
    let permutation = Permutation::from_cycles(vec![
        vec![40,42,47,45],
        vec![41,44,46,43],
        vec![13,21,29,37],
        vec![14,22,30,38],
        vec![15,23,31,39],
    ]);


    assert_eq!(permutation.map, vec![0,2,3,1]);
}


