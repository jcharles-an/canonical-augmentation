use std::fmt::Debug;
use std::hash::Hash;

pub trait Group {
    type Element: Debug;
    type Target:Debug + Eq + Hash + Clone + Copy;

    fn operate(element_a: &Self::Element, element_b: &Self::Element) -> Self::Element;
    fn get_inverse(element: &Self::Element) -> Self::Element;

    fn apply_to(element: &Self::Element, target: &Self::Target) -> Option<Self::Target>;

    fn get_identity(&self) -> Self::Element;

    fn get_generators(&self) -> &Vec<Self::Element>;

}