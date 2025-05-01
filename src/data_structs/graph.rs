use crate::data_structs;
use crate::data_structs::bitmaps::{BM, BMC, DEGREE, N};

#[derive(Clone,Debug)]
pub struct Graph {
    pub ady_matrix: Vec<BM>,
}


impl Graph {
    pub fn is_set_dominant(&self, set: &BM) -> bool {
        for list in self.ady_matrix.iter() {
            if (*list & *set).len() == 0 {
                return false;
            }
        }
        true
    }

    pub fn is_set_separating(&self, set: &BM) -> bool {
        for (i, u) in self.ady_matrix.iter().enumerate() {
            for v in self.ady_matrix.iter().skip(i + 1) {
                if (*u & *set) == (*v & *set) {
                    return false;
                }
            }
        }
        true
    }

    fn get_num(ady_matrix: Vec<BM>)-> BMC{
        let mut num = BMC::new();
        let mut lower_limit = 1;
        let len = ady_matrix.len();
        let mut initial = 0;
        for n in (0..len).rev() {
            //println!("{:07b} -> {}", n.into_value(), n.into_value());
            for i in lower_limit..len{
                num.set(initial, ady_matrix[n].get(i));
                initial += 1;
            }
            //println!("{:021b}", num.into_value());
            lower_limit += 1;
        }
        num
    }


}


pub fn get_n_cube_ady_matrix() -> Vec<BM> {
    let degree = DEGREE;
    let n = N;
    let mut ady_matrix = vec![BM::new();n];
    for v in 0..n {
        for i in 0..degree {
            let neighbor = v ^ (1 << i);
            ady_matrix[v].set(neighbor, true);
            ady_matrix[neighbor].set(v, true);
        }
    }
    ady_matrix
}

#[test]
fn graph_test() {
    let mut graph = Graph { ady_matrix: vec![
        data_structs::bitmaps::from_BE_array(&[1,1,0,0,0,1]),
        data_structs::bitmaps::from_BE_array(&[1,1,1,0,0,0]),
        data_structs::bitmaps::from_BE_array(&[0,1,1,1,0,0]),
        data_structs::bitmaps::from_BE_array(&[0,0,1,1,1,0]),
        data_structs::bitmaps::from_BE_array(&[0,0,0,1,1,1]),
        data_structs::bitmaps::from_BE_array(&[1,0,0,0,1,1]),
    ] };

    assert_eq!(graph.is_set_dominant(&data_structs::bitmaps::from_BE_array(&[1,1,0,0,0,0])), false);
    assert_eq!(graph.is_set_dominant(&data_structs::bitmaps::from_BE_array(&[1,1,1,1,1,1])), true);
    assert_eq!(graph.is_set_dominant(&data_structs::bitmaps::from_BE_array(&[0,1,0,1,0,1])), true);
    assert_eq!(graph.is_set_dominant(&data_structs::bitmaps::from_BE_array(&[1,0,1,0,1,0])), true);
    assert_eq!(graph.is_set_dominant(&data_structs::bitmaps::from_BE_array(&[1,0,0,1,0,0])), true);

    /*
    assert_eq!(graph.is_set_separating(&data_structs::bitmaps::from_array(&[1,1,0,0,0,0])), false);
    assert_eq!(graph.is_set_separating(&data_structs::bitmaps::from_array(&[1,1,1,1,1,1])), true);
    assert_eq!(graph.is_set_separating(&data_structs::bitmaps::from_array(&[0,1,0,1,0,1])), true);
    assert_eq!(graph.is_set_separating(&data_structs::bitmaps::from_array(&[1,0,1,0,1,0])), true);

&     */
    //assert_eq!(graph.is_set_separating(&data_structs::bitmaps::from_array(&[1,0,0,1,0,0])), true);


    let d = Graph::get_num( vec![
        data_structs::bitmaps::from_BE_array(&[0,0,0,0,0,0,1]),
        data_structs::bitmaps::from_BE_array(&[0,0,0,0,1,1,0]),
        data_structs::bitmaps::from_BE_array(&[0,0,0,1,0,1,0]),
        data_structs::bitmaps::from_BE_array(&[0,0,1,0,1,1,1]),
        data_structs::bitmaps::from_BE_array(&[0,1,0,1,0,1,1]),
        data_structs::bitmaps::from_BE_array(&[0,1,1,1,1,0,1]),
        data_structs::bitmaps::from_BE_array(&[1,0,0,1,1,1,0]),
    ]);

    println!("{:021b}", d.into_value());

}