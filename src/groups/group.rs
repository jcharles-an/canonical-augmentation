use std::cmp;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use crate::data_structs::consts::N;
use crate::data_structs::stabilizer_chain::StabChain;
use crate::group_actions::groupAction::GroupAction;
use crate::data_structs::permutation::Permutation;
use crate::groups::permutation_group::PermutationGroup;

type Target = usize;
pub trait Group:Clone{
    type Element: Debug + Clone + Eq + Hash;

    fn new(generators: Vec<Self::Element>) -> Self;
    fn operate(element_a: &Self::Element, element_b: &Self::Element) -> Self::Element;
    fn get_inverse(element: &Self::Element) -> Self::Element;
    fn is_identity(element: &Self::Element) -> bool;

    fn get_identity(&self) -> Self::Element;
    fn get_identity_ref(&self) -> &Self::Element;
    fn get_identity_length(&self) -> usize;

    fn get_generators(&self) -> &Vec<Self::Element>;
    fn get_generators_as_mut(&mut self) -> &mut Vec<Self::Element>;
    fn get_stab_chain(&self) -> &StabChain<Self::Element>;
    fn set_stab_chain(&mut self, stab_chain: StabChain<Self::Element>);
    fn add_generator(&mut self, new_generator: Self::Element);

    fn new_empty()->impl Group;
}