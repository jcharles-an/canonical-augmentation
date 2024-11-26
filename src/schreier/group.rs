use std::fmt::Debug;
use std::hash::Hash;

pub trait Group {
    type Element: Debug + Clone;
    type Target:Debug + Eq + Hash + Clone + Copy;

    fn target_to_usize(target: Self::Target) -> usize;

    fn operate(element_a: &Self::Element, element_b: &Self::Element) -> Self::Element;
    fn get_inverse(element: &Self::Element) -> Self::Element;
    fn apply_to(element: &Self::Element, target: &Self::Target) -> Option<Self::Target>;
    fn is_identity(element: &Self::Element) -> bool;

    fn get_identity(&self) -> Self::Element;
    fn get_identity_ref(&self) -> &Self::Element;
    fn get_identity_length(&self) -> usize;

    fn get_generators(&self) -> &Vec<Self::Element>;
    fn add_generator(&mut self, new_generator: Self::Element);

    fn new_empty()->impl Group;

}