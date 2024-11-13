use std::collections::HashMap;
use std::vec;

pub trait Group{
    type Element;
    type Target;
    fn operate(element_a: &Self::Element, element_b: &Self::Element) -> Self::Element;
    fn get_inverse(&self) -> Self::Element;

    fn get_generators(&self) -> &[&[Self::Element]];

    fn apply_to(&self, element: &Self::Element, target: &Self::Target) -> Option<Self::Target>;



    fn get_new_elements_in_orbit(){

    }
}
