use std::cmp;
use std::fmt::{Debug, Display, Formatter};
use bincode::{Decode, Encode};
use bitmaps::Bitmap;

#[derive(Clone, Ord, Eq, Hash, PartialEq, Encode, Decode, PartialOrd)]
pub struct Permutation {
    pub map: Vec<usize>,
    pub size: usize,
    pub type_array: Option<Vec<usize>>,
}

impl Display for Permutation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}",self.to_cycles())
    }
}

impl Debug for Permutation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}",self.to_cycles())
    }
}

impl Permutation{
    pub fn new(map: Vec<usize>) -> Permutation{
        Permutation{size: map.len(), map, type_array: None}
    }

    pub fn from_cycles(cicles: Vec<Vec<usize>>) -> Permutation{
        let mut max_cycle_value = 0usize;
        for cicle in cicles.iter(){
            max_cycle_value = cmp::max(max_cycle_value, *cicle.iter().max().unwrap());
        }

        let mut permutation: Vec<usize> = (0..max_cycle_value+1).into_iter().collect();
        for cicle in cicles.iter(){
            for window in cicle.windows(2){
                permutation[window[0]] = window[1];
            }
            permutation[*cicle.last().unwrap()] = cicle[0];
        }
        Permutation::new(permutation)
    }

    pub fn to_cycles(&self) -> Vec<Vec<usize>>{
        let n = self.size;
        let mut visited = Bitmap::<64>::new();
        let mut cycles = vec![vec![0usize]; 0];
        for (index, mut value) in self.map.iter().enumerate(){
            if visited.get(index){
                continue;
            }
            let mut cycle = vec![index];
            visited.set(index, true);
            while &index != value {
                cycle.push(*value);
                visited.set(*value, true);

                value = &self.map[*value];
            }
            if cycle.len() > 1{
                cycles.push(cycle);
            }
        }
        cycles
    }

    pub fn get_type(&mut self) -> &[usize]{
        if self.type_array.is_none(){
            let mut p = vec![true; self.size];
            let mut t = vec![0; self.size];
            let mut l;
            let mut j;
            for i in 0..self.size{
                if p[i]{
                    l = 1;
                    j = i;
                    p[j] = false;
                    let permuted = self.map[j];
                    while p[permuted] {
                        l += 1;
                        j = permuted;
                        p[j] = false;
                    }
                    t[l] += 1;
                }
            }
            self.type_array = Some(t)
        }
        self.type_array.as_ref().unwrap()
    }
}

#[test]
fn permutation_test() {
    let permutation = Permutation::from_cycles(vec![vec![0,1,2]]);
    assert_eq!(permutation.map, vec![1,2,0]);
    assert_eq!(permutation.to_cycles(), vec![vec![0,1,2]]);


    let permutation = Permutation::from_cycles(vec![vec![1,2,3]]);
    assert_eq!(permutation.map, vec![0,2,3,1]);

    let mut permutation = Permutation::from_cycles(vec![vec![1,2,3], vec![4,7]]);
    println!("{:?}", permutation.get_type());
    assert_eq!(permutation.map, vec![0, 2, 3, 1, 7, 5, 6, 4]);
    assert_eq!(permutation.to_cycles(), vec![vec![1,2,3], vec![4,7]]);

    return;
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


