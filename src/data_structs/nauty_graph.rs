use crate::data_structs::graph::Graph;
use crate::schreier::permutation::Permutation;
use nauty_Traces_sys::{densenauty, empty_graph, graph, nauty_check, optionblk, statsblk, ADDONEEDGE, FALSE, NAUTYVERSIONID, SETWORDSNEEDED, TRUE, WORDSIZE};
use petgraph::visit::Walker;
use std::os::raw::c_int;
use std::sync::{Arc, RwLock};
use rayon::current_num_threads;
use crate::schreier::automorphism::Automorphism;
use crate::schreier::schreier_permutation::Schreier;

static mut GENERATORS: Option<Arc<RwLock<Vec<Permutation>>>> = None;


pub struct NautyGraph {
    pub g: Vec<graph>,
    pub n: usize,
    pub m: usize,
    pub lab: Vec<c_int>,
    pub ptn: Vec<c_int>,
}

impl NautyGraph {
    pub fn from( graph: &Graph ) -> NautyGraph {

        let n = graph.ady_matrix.len();
        let m = SETWORDSNEEDED(n);
        let mut g = empty_graph(m, n);
        
        for (i, bm) in graph.ady_matrix.iter().enumerate() {
            for j in bm {
                if j > i {
                    ADDONEEDGE(&mut g, i, j, m);
                }
            }
        }
        
        let lab = (0..n).map(|x| x as c_int).collect::<Vec<c_int>>();
        let mut ptn = vec![1 as c_int; n];
        ptn[n/2 - 1] = 0;
        ptn[n - 1] = 0;

        // [[[1, 3]], [[2, 4], [5, 6]], [[0, 2], [6, 7]]]
        // [[[1, 3]], [[2, 4], [5, 6]], [[0, 2], [6, 7]]]
        NautyGraph{
            g,
            m,
            n,
            lab,
            ptn,
        }
    }

    unsafe extern "C" fn autom_proc(
        count: c_int,       // We don't use `count` in this example, hence `_count`
        generator: *mut c_int,
        orbits: *mut c_int,
        n_orb: c_int,
        stab_vert: c_int,      // We don't use `digraph`
        n: c_int,      // We don't use `digraph`
    ) {
        if let Some(ref generators_mutex) = GENERATORS {
            let mut generators = generators_mutex.write().unwrap(); // Acquire mutex lock
            let mut current_generator = Vec::with_capacity(n as usize);

            for i in 0..n {
                current_generator.push(*generator.add(i as usize) as usize);
            }

            generators.push(Permutation::new(current_generator));
        }
    }

    pub fn get_aut_generators(graph: &Graph) -> Option<Vec<Permutation>> {
        unsafe {
            GENERATORS = Some(Arc::new(RwLock::new(Vec::new())));
        }

        let mut options = optionblk{
            userautomproc: Some(Self::autom_proc),
            defaultptn: FALSE,
            ..Default::default()
        };

        let mut stats = statsblk::default();

        let nauty_graph = NautyGraph::from(graph);
        let n = nauty_graph.n;
        let m = nauty_graph.m;
        let mut g = nauty_graph.g;
        let mut lab = nauty_graph.lab;
        let mut ptn =nauty_graph.ptn;

        unsafe {
            nauty_check(WORDSIZE as c_int, m as c_int, n as c_int, NAUTYVERSIONID as c_int);
        }
        let mut orbits = vec![0; n];


        unsafe {
            densenauty(
                g.as_mut_ptr(),
                lab.as_mut_ptr(),
                ptn.as_mut_ptr(),
                orbits.as_mut_ptr(),
                &mut options,
                &mut stats,
                m as c_int,
                n as c_int,
                std::ptr::null_mut()
            );
        }

        let mut generators = None;
        unsafe {
            generators = Some(GENERATORS.as_ref()?.read().unwrap().clone());
        }

        generators
    }
}


#[test]
fn test_automorphism() {
    let ady_mat = vec![
        crate::data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 1, 1, 0, 0]),
        crate::data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0]),
        crate::data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0]),
        crate::data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0]),
        crate::data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0]),
        crate::data_structs::bitmaps::from_BE_array(&[1, 0, 0, 0, 0, 0, 0, 0]),
        crate::data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0]),
        crate::data_structs::bitmaps::from_BE_array(&[0, 0, 0, 0, 0, 0, 0, 0]),
    ];
    let mut g = Graph {
        ady_matrix: ady_mat.clone(),
    };
    
    ////////////////////////////////////////////////
    
    
    //schreier.group.simple_generate_group();
    
    
}