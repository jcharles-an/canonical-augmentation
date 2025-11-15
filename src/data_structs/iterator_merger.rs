use crate::n_k_combinatorial::NkCombManager::merge_sorted;

pub struct IteratorMerger<T>
where
    T: Iterator,
    T::Item: Clone + Ord,
{
    iters: Vec<T>,
    current_elements:  Vec<(usize, Option<T::Item>, usize)>
}


impl<T> IteratorMerger<T>
where
    T: Iterator,
    T::Item: Clone + Ord,
{
    pub fn new(mut iters: Vec<T>) -> Self{
        // add a 0 to avoid setting nones, before when ordering
        let mut current_elements: Vec<(usize, Option<T::Item>, usize)> = iters.iter_mut().enumerate()
            .map(|(index, iterator)| {
                let next = iterator.next();
                (if next.is_none() { 1 } else { 0 }, next, index)
            })
            .collect();
        IteratorMerger{
            iters,
            current_elements
        }

    }
}

impl<T:Iterator> Iterator for IteratorMerger<T>
where
    T: Iterator,
    T::Item: Clone + Ord,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let min = self.current_elements.iter().min();
        let (_, min_val, _) = min.unwrap();
        if min_val.is_none() {
            return None;
        }
        let min_val = min_val.clone();
        self.current_elements = self.current_elements.iter().map(
            |(state, val, index)| {
                if val == &min_val {
                    let next = self.iters[*index].next();
                    return (if next.is_none() { 1 } else { 0 }, next, *index);
                }
                (*state, val.clone(), *index)
            }).collect::<Vec<(usize, Option<T::Item>, usize)>>();
        min_val

    }
}



#[test]
fn merge_sorted_test(){

    let numstring = "18446741950994212328";
    let num = numstring.parse::<u128>().unwrap();
    println!("num: {}", num);


    let mut lists = vec![
        vec![0,1,3,7].into_iter(),
        vec![0,4,6,7].into_iter(),
        vec![0,2,8].into_iter(),
        vec![6,8].into_iter(),
        vec![9,10,11].into_iter(),
        vec![4].into_iter(),
    ];
    let iterMerger = IteratorMerger::new(lists);

    let sorted = iterMerger.collect::<Vec<u128>>();
    println!("sorted: {:?}", sorted);

    let mut lists = vec![
        vec![(938463463374607431768211455u128, "my:graph"),(938469463374607431768211455, "my:graph"),(939463463374607431768211455, "my:graph"),(1938463463374607431768211455, "my:graph"),].into_iter(),
        vec![(463463374607431768211455, "my:graph"),(998463463374607431768211455, "my:graph"),(999463463374607431768211455, "my:graph")].into_iter(),
        vec![(1938463463374607431768211455, "my:graph"),(1938463463374607431768211459, "my:graph"),(1998463463374607431768211455, "my:graph"),(1999463463374607431768211455, "my:graph"),(2999463463374607431768211455, "my:graph"),(9999463463374607431768211455, "my:graph"),].into_iter(),
    ];
    let iterMerger = IteratorMerger::new(lists);

    let sorted: Vec<_> = iterMerger.collect();
    println!("sorted: {:?}", sorted);
}
