//! Diagram - the main logic for this package
//! 
//! 
//! Allows verification of commutativity on diagrams with given sets of elements and maps
//! Example:
//! 
//! ```
//! use itertools::Itertools;
//! use commuter::diagram::{Diagram, Set, Map, diagram_commutes, CommutativeDiagramResult};
//!
//! let generating_integers: Vec<i32> = (0..20).collect();
//!
//! let left_add = |(a, b, c): &(i32, i32, i32)| (a + b, *c);
//! let right_add = |(a, b, c): &(i32, i32, i32)| (*a, b + c);
//!
//! // Build the sets (triplets, pairs, integers) so that summing operations can be well-defined later
//! let triplets: Vec<(i32, i32, i32)> = (generating_integers)
//!     .clone()
//!     .iter()
//!     .cartesian_product(generating_integers.clone())
//!     .cartesian_product(generating_integers.clone().iter())
//!     .map(|((a, b), c)| (*a, b, *c))
//!     .collect();
//!
//! // Construct the diagram
//! let diagram = Diagram::new(
//!     vec![
//!         Set::new(triplets), // This set gets some generating elements, on which the diagram will be tested
//!         Set::<(i32, i32), _, _>::new_no_generating_set(), // We only test on pairs that are generated by the given maps from the triplets, so no explicit elements are added here
//!         Set::new(vec![(5, 3), (100, 100)]), // We can also insert some additional generating elements into an intermediate set
//!         Set::<i32, _, _>::new_no_generating_set_checked(|x: &i32| *x >= 0),
//!     ],
//!     vec![
//!         Map::new(0, 1, left_add, "(+,id)"),
//!         Map::new(0, 2, right_add, "(id,+)"),
//!         Map::new(2, 3, |(a, b): &(i32, i32)| a + b, "(+)"),
//!         Map::new(1, 3, |(a, b): &(i32, i32)| a + b, "(+)"),
//!     ],
//! );
//!
//! assert!(match diagram_commutes(&diagram).unwrap() {
//!     CommutativeDiagramResult::Commutes => true,
//!     CommutativeDiagramResult::DoesNotCommute(reason) => panic!("{}", reason),
//! });
//!``` 
//! 

pub use crate::graph::CyclicGraphError;
use crate::graph::{all_paths, DiGraph, Edge};
use dyn_clonable::*;
use std::any::Any;
use std::rc::Rc;

#[clonable]
pub trait Element: Clone {
    fn eq(&self, other: &Rc<dyn Element>) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn name(&self) -> String;
}

pub trait SetLike {
    fn elements(&self) -> Box<dyn Iterator<Item = Rc<dyn Element>>>;

    // If false, the set element fails validation
    fn check(&self, element: &Rc<dyn Element>) -> bool;

    // If false, the set element is filtered from validation
    fn filter(&self, element: &Rc<dyn Element>) -> bool;
}

#[derive(Clone)]
pub struct Set<T, P, F>
where
    T: Clone + Element + Sized,
    P: Fn(&T) -> bool,
    F: Fn(&T) -> bool
{
    elements: Vec<T>,
    property: P,
    filter: F
}


impl<T> Set<T, fn(&T) -> bool, fn(&T) -> bool> where T: Clone + Element + Sized  {
    pub fn new(elements: Vec<T>) -> Rc<Set<T, fn(&T) -> bool, fn(&T) -> bool>>{
        Rc::new(Self{elements, property: |_x| true, filter: |_x| true})   
    }

    pub fn new_no_generating_set() -> Rc<Set<T, fn(&T) -> bool, fn(&T) -> bool>> {
        Rc::new(Self {
            elements: Vec::new(), property: |_x| true, filter: |_x| true
        })
    }
}

impl<T, P> Set<T, P, fn(&T) -> bool>
where
    T: Clone + Element + Sized,
    P: Fn(&T) -> bool
{

    pub fn new_checked(elements: Vec<T>, property: P) -> Rc<Set<T, P, fn(&T) -> bool>> {
        Rc::new(Self { elements, property, filter: |_x| true})
    }

    pub fn new_no_generating_set_checked(property: P) -> Rc<Set<T, P, fn(&T) -> bool >> {
        Rc::new(Self {
            elements: Vec::new(), property, filter: |_x| true
        })
    }
}


impl<T, F> Set<T, fn(&T) -> bool, F>
where
    T: Clone + Element + Sized,
    F: Fn(&T) -> bool
{

    pub fn new_filtered(elements: Vec<T>, filter: F) -> Rc<Set<T, fn(&T) -> bool, F>> {
        Rc::new(Self { elements, property: |_x| true, filter})
    }

    pub fn new_no_generating_set_filtered(filter: F) -> Rc<Set<T, fn(&T) -> bool, F>> {
        Rc::new(Self {
            elements: Vec::new(), property: |_x| true, filter
        })
    }
}


impl<T, P, F> Set<T, P, F>
where
    T: Clone + Element + Sized,
    P: Fn(&T) -> bool,
    F: Fn(&T) -> bool
{

    pub fn new_checked_filtered(elements: Vec<T>, property: P, filter: F) -> Rc<Set<T, P, F>> {
        Rc::new(Self { elements, property, filter})
    }

    pub fn new_no_generating_set_checked_filtered(property: P, filter: F) -> Rc<Set<T, P, F>> {
        Rc::new(Self {
            elements: Vec::new(), property, filter
        })
    }
}

impl<T, P, F> SetLike for Set<T, P, F>
where
    T: Clone + Element + Sized + 'static,
    P: Fn(&T) -> bool,
    F: Fn(&T) -> bool
{
    fn elements(&self) -> Box<dyn Iterator<Item = Rc<dyn Element>>> {
        let owned_els = self.elements.clone();
        Box::new(
            owned_els
                .into_iter()
                .map(|e| Rc::new(e.clone()) as Rc<dyn Element>),
        )
    }
    
    fn check(&self, element: &Rc<dyn Element>) -> bool {
        (self.property)(element.as_any().downcast_ref::<T>().unwrap())
    }

    fn filter(&self, element: &Rc<dyn Element>) -> bool {
        (self.filter)(element.as_any().downcast_ref::<T>().unwrap())
    }
}

trait Mappable {
    fn map(&self, key: &Rc<dyn Element>) -> Option<Rc<dyn Element>>;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DiEdge {
    from: usize,
    to: usize,
    ix: usize,
}

impl Edge for DiEdge {
    type Node = usize;

    fn from(&self) -> &Self::Node {
        &self.from
    }

    fn to(&self) -> &Self::Node {
        &self.to
    }
}

impl DiGraph for Diagram {
    type Node = usize;
    type Edge = DiEdge;

    fn nodes(&self) -> Box<dyn Iterator<Item = Self::Node>> {
        let result: Vec<usize> = (0..self.sets.len()).collect();
        Box::new(result.into_iter())
    }

    fn outbounds(&self, node: &Self::Node) -> Box<dyn Iterator<Item = Self::Edge>> {
        let result: Vec<DiEdge> = self
            .maps
            .iter()
            .enumerate()
            .filter(
                move |(
                    _ix,
                    Map {
                        from,
                        to: _,
                        map: _,
                        name: _,
                    },
                )| from == node,
            )
            .map(
                |(
                    ix,
                    Map {
                        from,
                        to,
                        map: _,
                        name: _,
                    },
                )| DiEdge {
                    from: *from,
                    to: *to,
                    ix: ix,
                },
            )
            .collect();

        Box::new(result.into_iter())
    }
}

struct ValueMap<T, U> {
    map: Rc<dyn Fn(&T) -> U>,
}

impl<T, U> Mappable for ValueMap<T, U>
where
    T: Element + 'static,
    U: Element + 'static,
{
    fn map(&self, key: &Rc<dyn Element>) -> Option<Rc<dyn Element>> {
        if let Some(key) = key.as_any().downcast_ref::<T>() {
            Some(Rc::new((self.map)(key)))
        } else {
            None
        }
    }
}

impl<T, U> ValueMap<T, U>
where
    T: Element + 'static,
    U: Element + 'static,
{
    pub fn new<M>(map: M) -> Rc<ValueMap<T, U>>
    where
        M: Fn(&T) -> U + 'static,
    {
        Rc::new(ValueMap {
            map: Rc::new(move |x| map(x)),
        })
    }
}

pub struct Map {
    from: usize,
    to: usize,
    map: Rc<dyn Mappable>,
    name: String,
}

impl Map {
    pub fn new<F, U, V>(from: usize, to: usize, map: F, name: &str) -> Map
    where
        F: Fn(&U) -> V + 'static + Clone,
        U: Clone + PartialEq + core::fmt::Debug + 'static,
        V: Clone + PartialEq + core::fmt::Debug + 'static,
    {
        Map {
            from: from,
            to: to,
            map: ValueMap::new(map.clone()),
            name: name.to_owned(),
        }
    }
}

pub struct Diagram {
    sets: Vec<Rc<dyn SetLike>>,
    maps: Vec<Map>, // (usize, usize, Rc<dyn Mappable>, String)>,
}

#[derive(Clone, Debug)]
pub enum CommutativeDiagramResult {
    Commutes,
    DoesNotCommute(String),
}

impl Diagram {
    pub fn new(sets: Vec<Rc<dyn SetLike>>, maps: Vec<Map>) -> Diagram {
        Diagram { sets, maps }
    }
}

#[derive(Clone, Debug)]
pub enum CommutativeDiagramError {
    CyclicGraphError,
    PropertyCheckError(String)
}

pub fn diagram_commutes(diagram: &Diagram) -> Result<CommutativeDiagramResult, CommutativeDiagramError> {
    // Find all paths through the diagram
    let all_possible_paths = all_paths(diagram).map_err(|_err| CommutativeDiagramError::CyclicGraphError)?;

    // For each pair of paths...
    for path_a in &all_possible_paths.clone() {
        for path_b in &all_possible_paths.clone() {
            // Check if these two paths match
            if path_a.first().map(|e| e.from()) == path_b.first().map(|e| e.from())
                && path_a.last().map(|e| e.to()) == path_b.last().map(|e| e.to())
            {
                // The paths line up, let's look at every elemnt of their common source
                let common_source: &<Diagram as DiGraph>::Node =
                    path_a.first().map(|e| e.from()).unwrap();

                // Find each element of the common source
                let source_set = &diagram.sets[*common_source];

                // Now, map this source set through both of the paths
                let source_elements = source_set.elements();
                'outer: for element in source_elements {

                    // Check if this element should be filtered
                    if !source_set.filter(&element) {
                        continue 'outer; // Next!
                    }

                    // Verify the element in the source set.
                    if !source_set.check(&element) {
                        return Err(CommutativeDiagramError::PropertyCheckError(format!("Element does not satisfy source set property: {:?}", element.name().clone())));
                    }

                    let mut path_a_element = element.clone();

                    for edge in path_a {
                        let map = &diagram.maps[edge.ix].map;
                        let set = &diagram.sets[*edge.to()];

                        path_a_element = map.map(&path_a_element).unwrap();
                        
                        // Check if this element/path should be filtered
                        if !set.filter(&path_a_element) {
                            continue 'outer;
                        }

                        // Check if this element passes validation
                        if !set.check(&path_a_element) {
                            return Err(CommutativeDiagramError::PropertyCheckError(format!("Element does not satisfy target set property: {:?}", path_a_element.name().clone())));
                        }
                    }

                    let mut path_b_element = element.clone();

                    for edge in path_b {
                        let map = &diagram.maps[edge.ix].map;
                        let set = &diagram.sets[*edge.to()];

                        path_b_element = map.map(&path_b_element).unwrap();

                        // Check if this element/path should be filtered
                        if !set.filter(&path_b_element) {
                            continue 'outer;
                        }

                        // Check if this element passes validation
                        if !set.check(&path_b_element) {
                            return Err(CommutativeDiagramError::PropertyCheckError(format!("Element does not satisfy target set property: {:?}", path_a_element.name().clone())));
                        }
                    }

                    // Now, check if the two elements are equal
                    if !path_a_element.eq(&path_b_element) {
                        // Get descriptions for both paths
                        let path_a_description = path_a
                            .iter()
                            .map(|edge| diagram.maps[edge.ix].name.clone())
                            .collect::<Vec<String>>()
                            .join(" -> ");

                        let path_b_description = path_b
                            .iter()
                            .map(|edge| diagram.maps[edge.ix].name.clone())
                            .collect::<Vec<String>>()
                            .join(" -> ");

                        let element_name = element.name().clone();
                        let left_final_element_name = path_a_element.name();
                        let right_final_element_name = path_b_element.name();

                        let reason = format!(
                            "{} and {} don't agree on {}. Left gets {} while right gets {}",
                            path_a_description,
                            path_b_description,
                            element_name,
                            left_final_element_name,
                            right_final_element_name
                        );

                        return Ok(CommutativeDiagramResult::DoesNotCommute(reason.to_owned()));
                    }
                }
            }
        }
    }

    Ok(CommutativeDiagramResult::Commutes)
}

impl<T> Element for T
where
    T: Clone + PartialEq + Sized + 'static + core::fmt::Debug,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &Rc<dyn Element>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            self == other
        } else {
            false
        }
    }

    fn name(&self) -> String {
        format!("{:?}", self)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_addition_is_associative_on_integers() {
        use itertools::Itertools;

        let generating_integers: Vec<i32> = (0..20).collect();

        let left_add = |(a, b, c): &(i32, i32, i32)| (a + b, *c);
        let right_add = |(a, b, c): &(i32, i32, i32)| (*a, b + c);

        // Build the sets (triplets, pairs, integers) so that summing operations can be well-defined later
        let triplets: Vec<(i32, i32, i32)> = (generating_integers)
            .clone()
            .iter()
            .cartesian_product(generating_integers.clone())
            .cartesian_product(generating_integers.clone().iter())
            .map(|((a, b), c)| (*a, b, *c))
            .collect();

        let diagram = Diagram {
            sets: vec![
                Set::new(triplets),
                Set::<(i32, i32), _, _>::new_no_generating_set_filtered(|(a, b): &(i32, i32)| a + b >= 6),
                Set::<(i32, i32), _, _>::new_no_generating_set_filtered(|(a, b): &(i32, i32)| a + b >= 5),
                Set::<i32, _, _>::new_no_generating_set_checked(|x: &i32| *x >= 5),
            ],
            maps: vec![
                Map::new(0, 1, left_add, "(+,id)"),
                Map::new(0, 2, right_add, "(id,+)"),
                Map::new(2, 3, |(a, b): &(i32, i32)| a + b, "(+)"),
                Map::new(1, 3, |(a, b): &(i32, i32)| a + b, "(+)"),
            ],
        };

        assert!(match diagram_commutes(&diagram).unwrap() {
            CommutativeDiagramResult::Commutes => true,
            CommutativeDiagramResult::DoesNotCommute(reason) => panic!("{}", reason),
        });
    }

}
