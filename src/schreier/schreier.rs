use std::collections::HashMap;
use crate::schreier::group::Group;

struct Schreier<'a, T: Group>{
    group: T,
    generators: &'a [&'a [T::Element]]

}

impl<'a, T:Group> Schreier<T>{
    pub fn get_orbit<'a>(&self, origin: &Self::Target) -> &'a[Self::Target]{
        let generators = self.get_generators();
        //TODO: is a hashmap really the best way to do this?
        let mut orbit = HashMap::new();
        orbit.insert(origin, true);
        let mut uncalculated_orbit= vec![origin];

        while !uncalculated_orbit.is_empty(){
            let mut new_uncalculated_orbit = Vec::new();
            for element in uncalculated_orbit{
                self.build_orbit()
            }

        }

        let mut missing_

        Self::operate(&element,&element)
    }

    fn build_orbit(&self, orbit_prefix: &mut HashMap<&T::Element,bool> ,element: &T::Element) {

        for generator in self.generators{
            if let Some(transformed) = self.apply_to(generator, element){
                if let None = orbit_prefix.insert(transformed,true) {
                    self.build_orbit(orbit_prefix,transformed)
                }
            }
        }
    }

}