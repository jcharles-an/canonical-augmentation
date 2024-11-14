use std::hash::Hash;

pub trait GroupElement{
    type Element;
    type Target:Eq + Hash + Clone + Copy;
    fn operate(&self, element: &Self::Element) -> Self::Element;
    fn get_inverse(&self) -> Self::Element;

    fn apply_to(&self, target: &Self::Target) -> Option<Self::Target>;
}