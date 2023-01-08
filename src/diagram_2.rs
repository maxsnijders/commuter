use std::any::Any;
use crate::graph::{DiGraph, Edge, all_paths, CyclicGraphError};
use dyn_clone::DynClone;

trait Element: DynClone{
    fn eq(&self, other: &Box<dyn Element>) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn name(&self) -> String;
}

trait Set<'a> {
    fn elements(&self) -> Box<dyn Iterator<Item = Box<dyn Element + 'a>> + 'a>;
}

trait Map {
    fn map(&self, key: &Box<dyn Element>) -> Option<Box<dyn Element>>;
}

struct Diagram<'a> {
    sets: Vec<Box<dyn Set<'a>>>,
    maps: Vec<(usize, usize, Box<dyn Map>, String)>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct DiEdge {
    from: usize,
    to: usize,
    ix: usize
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

impl<'a> DiGraph for Diagram<'a> {
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
            .filter(move |(ix, (from, to, map, name))| from == node)
            .map(|(ix, (from, to, map, name))| DiEdge{from: *from, to: *to, ix:ix})
            .collect();

        Box::new(result.into_iter())
    }
}

pub enum CommutativeDiagramResult {
    Commutes,
    DoesNotCommute(String),
}

fn diagram_commutes(diagram: &Diagram) ->  Result<CommutativeDiagramResult, CyclicGraphError> {
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
                let common_destination: &<Diagram as DiGraph>::Node =
                    path_a.last().map(|e| e.to()).unwrap();

                // Find each element of the common source
                let source_set = &diagram.sets[*common_source];
                let target_set = &diagram.sets[*common_destination];

                // Now, map this source set through both of the paths
                for element in source_set.elements() {
                    let mut path_a_element = &element; // dyn_clone::clone_box(&*element);

                    for edge in path_a {
                        let (_, _, map, name) = &diagram.maps[edge.ix];
                        let x = map.map(path_a_element);
                        path_a_element = &(x.unwrap());
                    }

                    let mut path_b_element = &element; // dyn_clone::clone_box(&*element);

                    for edge in path_b {
                        let (_, _, map, name) = &diagram.maps[edge.ix];
                        path_b_element = &map.map(path_b_element).unwrap();
                    }

                    // Now, check if the two elements are equal
                    if path_a_element.eq(&path_b_element){

                        // Get descriptions for both paths
                        let path_a_description = path_a
                            .iter()
                            .map(|edge| {
                                let (_, _, _, name) = &diagram.maps[edge.ix];
                                name.clone()
                            })
                            .collect::<Vec<String>>()
                            .join(" -> ");

                        let path_b_description = path_b
                            .iter()
                            .map(|edge| {
                                let (_, _, _, name) = &diagram.maps[edge.ix];
                                name.clone()
                            })
                            .collect::<Vec<String>>()
                            .join(" -> ");

                        let element_name = "x".to_owned(); // element.name().clone();
                        // let element_name = (source_set.element_name)(element);
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

                        return Ok(CommutativeDiagramResult::DoesNotCommute(reason));
                    }
                }
            }
        }
    }

    Ok(CommutativeDiagramResult::Commutes)
}

#[cfg(test)]
mod tests {

    use super::*;

    // #[derive(Clone, Copy, PartialEq)]
    // struct ValueElement<T> where T: Clone + PartialEq + Sized {
    //     value: T
    // }

    // impl<T> Element for ValueElement<T> where T: Clone + PartialEq + Sized{
    //     fn as_any(&self) -> &dyn Any {
    //         self
    //     }

    //     fn eq(&self, other: &Box<dyn Element>) -> bool {
    //         if let Some(other) = other.as_any().downcast_ref::<ValueElement<T>>() {
    //             self.value == other.value
    //         } else {
    //             false
    //         }
    //     }
    // }

    // #[derive(Clone, Copy, PartialEq)]
    // struct IntElement {
    //     value: i32,
    // }

    // impl Element for IntElement {
    //     fn as_any(&self) -> &dyn Any {
    //         self
    //     }

    //     fn eq(&self, other: &Box<dyn Element>) -> bool {
    //         if let Some(other) = other.as_any().downcast_ref::<IntElement>() {
    //             self.value == other.value
    //         } else {
    //             false
    //         }
    //     }
    // }

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

        fn eq(&self, other: &Box<dyn Element>) -> bool {
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

    struct TypeSet<T>
    where
        T: Clone + Element + Sized,
    {
        elements: Vec<T>,
    }

    impl<'a, T> Set<'a> for TypeSet<T>
    where
        T: Clone + Element + Sized + 'a,
    {
        fn elements(&self) -> Box<dyn Iterator<Item = Box<dyn Element + 'a>> + 'a> {
            let owned_els = self.elements.clone();
            Box::new(
                owned_els
                    .into_iter()
                    .map(|e| Box::new(e.clone()) as Box<dyn Element>),
            )
        }
    }

    struct ValueMap<T, U> {
        map: Box<dyn Fn(&T) -> U>,
    }

    impl<T, U> Map for ValueMap<T, U>
    where
        T: Element + 'static,
        U: Element + 'static,
    {
        fn map(&self, key: &Box<dyn Element>) -> Option<Box<dyn Element>> {
            if let Some(key) = key.as_any().downcast_ref::<T>() {
                Some(Box::new((self.map)(key)))
            } else {
                None
            }
        }
    }

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

        type IEl = TypeElement<i32>;
        type IntSet = TypeSet<IEl>;

        type PEl = TypeElement<(i32, i32)>;
        type IntPSet = TypeSet<PEl>;

        type TEl = TypeElement<(i32, i32, i32)>;
        type IntTSet = TypeSet<TEl>;

        let triplets = IntTSet {
            elements: triplets.iter().map(|x| TypeElement { value: *x }).collect(),
        };
        let pairs = IntPSet {
            elements: pairs.iter().map(|x| TypeElement { value: *x }).collect(),
        };
        let integers = IntSet {
            elements: integers.iter().map(|x| TypeElement { value: *x }).collect(),
        };

        let diagram = Diagram {
            sets: vec![Box::new(triplets), Box::new(pairs), Box::new(integers)],
            maps: vec![(
                0,
                1,
                Box::new(ValueMap {
                    map: Box::new(|TEl { value: (a, b, c) }| PEl { value: (a + b, *c) }),
                }),
                "(+,id)".to_owned()
            )],
        };

        assert!(match diagram_commutes(&diagram).unwrap() {
            CommutativeDiagramResult::Commutes => true,
            CommutativeDiagramResult::DoesNotCommute(reason) => panic!("{}", reason),
        });    }
}
