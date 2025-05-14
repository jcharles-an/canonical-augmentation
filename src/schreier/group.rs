use std::cmp;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use crate::data_structs::consts::N;
use crate::data_structs::stabilizer_chain::StabChain;
use crate::schreier::permutation::Permutation;
use crate::schreier::permutation_group::PermutationGroup;

type Target = usize;
pub trait Group {
    type Element: Debug + Clone + Eq + Hash;
    type Target:Debug + Eq + Hash + Clone + Copy;

    fn target_to_usize(target: Self::Target) -> usize;

    fn operate(element_a: &Self::Element, element_b: &Self::Element) -> Self::Element;
    fn get_inverse(element: &Self::Element) -> Self::Element;
    fn apply_to(element: &Self::Element, target: &Self::Target) -> Self::Target;
    fn is_identity(element: &Self::Element) -> bool;

    fn get_identity(&self) -> Self::Element;
    fn get_identity_ref(&self) -> &Self::Element;
    fn get_identity_length(&self) -> usize;

    fn get_generators(&self) -> &Vec<Self::Element>;
    fn get_generators_as_mut(&mut self) -> &mut Vec<Self::Element>;
    fn get_stab_chain(&self) -> &StabChain<Self::Element>;
    fn set_stab_chain(&mut self, stab_chain: StabChain<Self::Element>);
    fn get_range(&self) -> usize;
    fn add_generator(&mut self, new_generator: Self::Element);

    fn new_empty()->impl Group;


}