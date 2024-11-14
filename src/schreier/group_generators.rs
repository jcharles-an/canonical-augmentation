use crate::schreier::group_element::GroupElement;

pub trait GroupGenerators {
    type Element:GroupElement;

    fn get_identity(&self) -> Self::Element;

}