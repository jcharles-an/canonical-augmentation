use std::cmp::PartialEq;
use std::ops::{BitAnd, BitOr, BitXor};
use crate::data_structs::bitmaps;
use crate::data_structs::bitmaps::{into_set, BM, BMC, N};
use crate::data_structs::graph::Graph;
use crate::schreier::automorphism::CompareRes::{Better, Same, Worse};
use crate::schreier::group::Group;
use crate::schreier::permutation::Permutation;
use crate::schreier::permutation_group::PermutationGroup;
use crate::schreier::schreier::Schreier;
use log::{debug, info, trace, warn};


#[derive(Debug)]
enum CompareRes {
    Worse,
    Better,
    Same
}

pub(crate) struct Automorphism {
    T: BM,
    g: Graph,
    U: BM,
    S: Vec<BM>,
    N: usize,

    pub(crate) mu: Option<Vec<usize>>,
    schreier: Schreier<PermutationGroup>,
    stab_chain: Vec<Vec<Permutation>>,
    stab_chain_base: Permutation,
}

impl PartialEq for CompareRes {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Worse, Worse) => true,
            (Better, Better) => true,
            (Same, Same) => true,
            _ => false,
        }
    }
}

impl Automorphism {

    pub(crate) fn new(graph: Graph) -> Automorphism {
        let identity = Permutation::new((0..N).collect());
        let mut aut = Automorphism{
            T: Default::default(),
            g: graph,
            U: Default::default(),
            S: Default::default(),
            N: N,
            mu: None,
            schreier: Schreier::new(
                PermutationGroup::new(vec![identity.clone()]),),
            stab_chain: vec![vec![]],
            stab_chain_base: identity,
        };

        aut
    }

    fn refine(&mut self, A: Vec<BM>) -> Vec<BM> {
        let n = N;
        let mut B = Vec::with_capacity(n);
        for (i, a) in A.iter().enumerate() {
            B.insert(i, *a)
        }
        self.S = B.iter().map(|b| *b).rev().collect();
        self.U = BM::mask(N);
        self.N = B.len();
        while self.N != 0 {
            //debug!("");

            self.N -= 1;
            self.T = self.S[self.N];

            /*
            Self::print_set_list("B", &B);
            Self::print_set_list("S", &self.S);
            Self::print_set("T", &self.T);
             */


            if self.T.bitand(self.U) == self.T {
                self.U = self.U.bitxor(self.T);
                let mut j = 0;
                let len_b = B.len();
                while j < len_b && len_b < n {
                    if len_b != 1 {
                        j = self.split_and_update(n, &mut B, j)
                    }
                    j += 1;
                }
                if B.len() == n {
                    return B;
                }
            }
        }
        B
    }

    fn split_and_update(&mut self, n: usize, B: &mut Vec<BM>, mut j: usize) -> usize{
        let mut L = vec![BM::new(); n];
        for u in &B[j]{
            let h = self.T.bitand(self.g.ady_matrix[u]).len(); // how many neighbors of u are in T
            L[h].set(u, true); // add vertex u to L if length h
        }
        //Self::print_set_list(&format!("L[{j}]"),&L);

        let m : usize = L.iter()
            .map(|l| if l.is_empty() {0} else {1})
            .sum();

        if m > 1{
            for h in (j+1..B.len()).rev() {
                if B.len() < h + m {
                    B.resize(h + m ,BM::new());
                }
                B[h + m - 1] = B[h].clone();
            }
            let mut k = 0;
            for h in 0..n {
                if !L[h].is_empty(){
                    if B.len() < j + k + 1 {
                        B.resize(j + k + 1 ,BM::new());
                    }
                    B[j + k] = L[h].clone();
                    let s_index = self.N + m - k - 1;
                    if self.S.len() < s_index + 1{
                        self.S.resize(s_index + 1,BM::new());
                    }
                    self.S[s_index] = L[h].clone();
                    self.U = self.U.bitor(L[h]);
                    k += 1
                }
            }
            j += m - 1;
            self.N = self.N + m
        }
        j
    }



    fn my_refine(mut self, n: usize, A: Vec<BM>) {
        let mut B = Vec::with_capacity(n);
        for (i, a) in A.iter().enumerate() {
            B.insert(i, *a)
        }
        self.S = B.clone();
        self.U = BM::mask(N);
        self.N = B.len();
        while self.N != 0 {
            self.N -= 1;
            self.T = self.S[self.N];
            if self.T.bitand(self.U) == self.T {
                self.U = self.U.bitxor(self.T);
                let mut j = 0;
                while j < B.len() && B.len() < n {
                    if B.len() != 1 {
                        self.my_split_and_update(n, &mut B, j)
                    }
                    j += 1;
                }
                if B.len() == n {
                    return;
                }
            }
        }
    }

    fn my_split_and_update(&mut self, n: usize, B: &mut Vec<BM>, mut current_index: usize){
        let mut L = vec![BM::new(); n];
        for u in &B[current_index]{
            let h = self.T.bitand(self.g.ady_matrix[u]).len(); // how many neighbors of u are in T
            L[h].set(u, true); // add vertex u to L if length h
        }

        let non_empty:usize = L.iter()
            .map(|l| -> usize {if l.is_empty() {0} else {1}})
            .sum();

        if non_empty > 1{
            for h in (current_index + 1..B.len()).rev() {
                B[non_empty - 1 + h] = B[h].clone();
            }
            let mut k = 0;
            for h in 0..n {
                if !L[h].is_empty(){
                    if B.len() < current_index + k + 1 {
                        B.resize(current_index + k + 1, BM::new());
                    }
                    B.insert(current_index + k, L[h].clone());
                    if self.S.len() < self.N + k + 1{
                        self.S.resize(self.N + k + 1,BM::new());
                    }
                    self.S[self.N + k] = L[h].clone();
                    self.U = self.U.bitor(L[h]);
                    k += 1
                }
                current_index += non_empty - 1;
                self.N = self.N + non_empty
            }
        }
    }


    fn compare(&self, p: &Vec<usize>, l: usize) -> CompareRes{
        if let Some(mu) = &self.mu{
            for j in 1..l-1 {
                for i in 0..=j-1{
                    let x = self.g.ady_matrix[mu[i]].get(mu[j]);
                    let y = self.g.ady_matrix[p[i]].get(p[j]);
                    if x < y {
                        return Worse;
                    }
                    if x > y {
                        return Better;
                    }
                }
            }
        }
        Same
    }

    // Algorithm 7.7
    fn cannon_1(&mut self, P: Vec<BM>) {
        Self::print_set_list("P",&P);
        let n = N;
        let mut pi = vec![0;n];
        let mut Q = self.refine(P);
        Self::print_set_list("Q",&Q);
        let mut l = 0;
        let mut bestExists = (&self.mu).is_some();
        for (i,q) in Q.iter().enumerate(){
            l = i;
            if q.len() > 1{
                break
            }
        }


        let mut res = Better;
        pi = self.partition_to_perm(&Q);
        if bestExists{
            res = self.compare(&pi, l);
        }

        if Q.len() == n {
            debug!("Res {:?} partition {:?}", res, self.partition_to_perm(&Q));
            let cert = self.get_cert(&pi).into_value();
            debug!("Cert: {:}", cert);
            debug!("Cert: {:b}", cert);
            if res == Better {
                self.mu = Some(pi.clone());
                let cert = self.get_cert(&pi).into_value();
            }
            debug!("");
        } else {
           if res != Worse {
                let C = Q[l];
                let mut D = C;
                let mut R= vec![BM::new(); Q.len() + 1];
                for j in 0..l{
                    R[j] = Q[j]
                }
                for j in l+1 .. Q.len() {
                    R[j+1] = Q[j]
                }
                for u in &C {
                    R[l] = BM::from_value(1<<u);
                    R[l+1] = D;
                    R[l+1].set(u,false);
                    self.cannon_1(R.clone());
                }

           }
        }

    }

    // Algorithm 7.8
    pub(crate) fn cert_1(&mut self) -> BMC {
        self.mu = None;
        self.cannon_1( vec![BM::mask(N)]);
        let mut C = BMC::new();
        if let Some(mu) = &self.mu{
            C = self.get_cert( mu )
        }
        C
    }

    fn partition_to_perm(&self, partition: &Vec<BM>) -> Vec<usize>{
        let mut perm = vec![0; N];
        let mut index = 0;
        for element in partition{
            for bitIndex in element{
                perm[index] = bitIndex;
                index += 1;
            }
        }
        perm
    }

    fn get_cert(&self, mu: &[usize]) -> BMC {
        let mut k = 0;
        let mut C = BMC::new();
        for j in (1..N).rev() {
            for i in (0..j).rev() {
                C.set(k, self.g.ady_matrix[mu[i]].get(mu[j]));
                k += 1;
            }
        }
        C
    }

    // Algorithm 7.7
    fn cannon_2(&mut self, P: Vec<BM>) {
        Self::print_set_list("P",&P);
        let n = N;
        let mut pi = vec![0;n];
        let mut Q = self.refine(P);
        Self::print_set_list("Q",&Q);
        debug!("");
        let mut l = 0;
        let mut bestExists = (&self.mu).is_some();
        for (i,q) in Q.iter().enumerate(){
            l = i;
            if q.len() > 1{
                break
            }
        }


        let mut res = Better;
        pi = self.partition_to_perm(&Q);
        if bestExists{
            res = self.compare(&pi, l);
        }

        if Q.len() == n {
            debug!("Res {:?} partition {:?}", res, self.partition_to_perm(&Q));
            let cert = self.get_cert(&pi).into_value();
            debug!("Cert: {:}", cert);
            debug!("Cert: {:b}", cert);
            if res == Better {
                self.mu = Some(pi.clone());
                let cert = self.get_cert(&pi).into_value();
            } else if res == Same {
                let Pi =Permutation::new(pi);
                let Mu = Permutation::new(self.mu.clone().unwrap().clone());
                debug!("pi: {:?}", Pi.map);
                debug!("mu: {:?}", Mu.map);

                let mut pi_2 = PermutationGroup::operate(&Pi, &PermutationGroup::get_inverse(&Mu));
                debug!("Will introduce permutation: {}", pi_2.to_string());
                let mut max_depth = 0;
                self.schreier.enter_2(n, &mut max_depth, &mut pi_2, &mut self.stab_chain, &self.stab_chain_base);
                debug!("Max depth: {}", max_depth);
            }
            debug!("");
        } else {
            if res != Worse {
                let mut C = Q[l];
                let mut D = C;
                let mut R= vec![BM::new(); Q.len() + 1];
                for j in 0..l{
                    R[j] = Q[j]
                }
                for j in l+1 .. Q.len() {
                    R[j+1] = Q[j]
                }
                while let Some(u) = C.first_index() {
                    R[l] = BM::from_value(1<<u);
                    R[l+1] = D;
                    R[l+1].set(u,false);
                    self.cannon_2(R.clone());
                    let mut new_base = Permutation::new(self.partition_to_perm(&R));
                    self.stab_chain =  self.schreier.change_base(n,&mut self.stab_chain,&new_base);
                    self.stab_chain_base = new_base;
                    for g in self.stab_chain[l].iter(){
                        C.set(PermutationGroup::apply_to(g,&u),false);
                    }
                }

            }
        }

    }

    // Algorithm 7.10
    pub(crate) fn cert_2(&mut self) -> BMC {
        self.mu = None;
        self.stab_chain = vec![vec![self.stab_chain_base.clone()]; N];
        self.cannon_2( vec![BM::mask(N)]);
        let mut C = BMC::new();
        if let Some(mu) = &self.mu{
            C = self.get_cert( mu )
        }
        C
    }

    pub fn print_set(name: &str, set: &BM){
        debug!("{name}: {:?}", into_set(set));
    }

    pub fn print_set_list(name: &str, set: &Vec<BM>){
        let mut a = format!("{name}: [");
        for s in set {
            a += &format!("{:?},", into_set(s));
        }
        a+= &"]".to_string();
        debug!("{a}");
    }

}


#[test]
fn test_automorphism() {
    let mut aut = Automorphism::new(
        Graph {
            ady_matrix: vec![
                bitmaps::from_BE_array(&[0,1,0,1,0,0,0,1]),
                bitmaps::from_BE_array(&[1,0,1,0,1,0,0,0]),
                bitmaps::from_BE_array(&[0,1,0,1,0,0,1,0]),
                bitmaps::from_BE_array(&[1,0,1,0,1,0,0,0]),
                bitmaps::from_BE_array(&[0,1,0,1,0,1,0,0]),
                bitmaps::from_BE_array(&[0,0,0,0,1,0,1,1]),
                bitmaps::from_BE_array(&[0,0,1,0,0,1,0,1]),
                bitmaps::from_BE_array(&[1,0,0,0,0,0,1,0]),
            ],
        }
    );

    let C = aut.cert_2();
    println!("Cert: {:}", C.into_value());
    println!("mu: {:?}", aut.mu);

/*
    let mut B = aut.refine(vec![
        bitmaps::from_BE_array(&[0,0,1,0,0,0,0,0]),
        bitmaps::from_BE_array(&[1,1,0,1,1,1,1,1]),
    ]);

    for b in &B{
        println!("{:?}", into_set(&b));
    }

   // B.sort();
    let pi = aut.partition_to_perm(&B);
    println!("{pi:?}");

 */
}