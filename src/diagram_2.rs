pub use crate::graph::CyclicGraphError;
use crate::graph::{all_paths, DiGraph, Edge};
use dyn_clonable::*;
use std::any::Any;
use std::rc::Rc;

#[clonable]
trait Element: Clone {
    fn eq(&self, other: &Rc<dyn Element>) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn name(&self) -> String;
}

trait Set {
    fn elements(&self) -> Box<dyn Iterator<Item = Rc<dyn Element>>>;
}

trait Mappable {
    fn map(&self, key: &Rc<dyn Element>) -> Option<Rc<dyn Element>>;
}

struct Map {
    from: usize, 
    to: usize, 
    map: Rc<dyn Mappable>,
    name: String,
}

impl Map {
    pub fn new<F, U, V> (from: usize, to: usize, map: F, name: String) -> Map where F: Fn(&U) -> V, U: Clone + PartialEq{
        Map {
            from: from,
            to: to,
            map: ValueMap::new(map),
            name: name,
        }
    }
}

pub struct Diagram {
    sets: Vec<Rc<dyn Set>>,
    maps: Vec<Map> // (usize, usize, Rc<dyn Mappable>, String)>,
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
            .filter(move |(_ix, Map{from, to, map, name})| from == node)
            .map(|(ix, Map{from, to, map, name})| DiEdge {
                from: *from,
                to: *to,
                ix: ix,
            })
            .collect();

        Box::new(result.into_iter())
    }
}

pub enum CommutativeDiagramResult {
    Commutes,
    DoesNotCommute(String),
}

pub fn diagram_commutes(diagram: &Diagram) -> Result<CommutativeDiagramResult, CyclicGraphError> {
    // Find all paths through the diagram
    let all_possible_paths = all_paths(diagram)?;

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
                for element in source_elements {
                    let mut path_a_element = element.clone();

                    for edge in path_a {
                        let map = &diagram.maps[edge.ix].map;
                        let x = map.map(&path_a_element).unwrap();
                        path_a_element = x;
                    }

                    let mut path_b_element = element.clone();

                    for edge in path_b {
                        let map = &diagram.maps[edge.ix].map;
                        let y = map.map(&path_b_element).unwrap();
                        path_b_element = y;
                    }

                    // Now, check if the two elements are equal
                    if !path_a_element.eq(&path_b_element) {
                        // Get descriptions for both paths
                        let path_a_description = path_a
                            .iter()
                            .map(|edge| {
                                diagram.maps[edge.ix].name.clone()
                            })
                            .collect::<Vec<String>>()
                            .join(" -> ");

                        let path_b_description = path_b
                            .iter()
                            .map(|edge| {
                                diagram.maps[edge.ix].name.clone()
                            })
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

#[derive(Clone, Copy, PartialEq)]
struct TypeElement<T>
where
    T: Clone + PartialEq + Sized,
{
    value: T,
}

impl<T> Element for TypeElement<T>
where
    T: Clone + PartialEq + Sized + 'static + core::fmt::Debug,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &Rc<dyn Element>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<TypeElement<T>>() {
            self.value == other.value
        } else {
            false
        }
    }

    fn name(&self) -> String {
        format!("{:?}", self.value)
    }
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

#[derive(Clone)]
struct TypeSet<T>
where
    T: Clone + Element + Sized,
{
    elements: Vec<T>,
}

impl<T> TypeSet<T>
where
    T: Clone + Element + Sized,
{
    pub fn new(elements: Vec<T>) -> Rc<TypeSet<T>> {
        Rc::new(Self { elements })
    }
}

impl<T> Set for TypeSet<T>
where
    T: Clone + Element + Sized + 'static,
{
    fn elements(&self) -> Box<dyn Iterator<Item = Rc<dyn Element>>> {
        let owned_els = self.elements.clone();
        Box::new(
            owned_els
                .into_iter()
                .map(|e| Rc::new(e.clone()) as Rc<dyn Element>),
        )
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_addition_is_associative_on_integers() {
        use itertools::Itertools;

        let generating_integers: Vec<i32> = (0..20).collect();

        // Build the sets (triplets, pairs, integers) so that summing operations can be well-defined later
        let triplets: Vec<(i32, i32, i32)> = generating_integers
            .clone()
            .iter()
            .cartesian_product(generating_integers.clone())
            .cartesian_product(generating_integers.clone().iter())
            .map(|((a, b), c)| (*a, b, *c))
            .collect();
        let pairs: Vec<(i32, i32)> = (0..40).cartesian_product(0..40).collect();
        let integers: Vec<i32> = (0..80).collect();

        let triplets = TypeSet::new(triplets);
        let pairs = TypeSet::new(pairs);
        let integers = TypeSet::new(integers);

        let diagram = Diagram {
            sets: vec![triplets, pairs.clone(), pairs, integers],
            maps: vec![
                Map::new(
                    0,
                    1,
                    ValueMap::new(|(a, b, c): &(i32, i32, i32)| (a + b, *c)),
                    "(+,id)".to_owned(),
                ),
                Map::new(
                    0,
                    2,
                    ValueMap::new(|(a, b, c): &(i32, i32, i32)| (*a, b + c)),
                    "(id,+)".to_owned(),
                ),
                Map::new
                (
                    2,
                    3,
                    ValueMap::new(|(a, b): &(i32, i32)| a + b),
                    "(+)".to_owned(),
                ),
                Map::new(
                    1,
                    3,
                    ValueMap::new(|(a, b): &(i32, i32)| a + b),
                    "(+)".to_owned(),
                ),
            ],
        };

        assert!(match diagram_commutes(&diagram).unwrap() {
            CommutativeDiagramResult::Commutes => true,
            CommutativeDiagramResult::DoesNotCommute(reason) => panic!("{}", reason),
        });
    }
}
