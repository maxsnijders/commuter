//! Provides the `Diagram` struct, which represents diagrams we'll test for commutativity
//! A diagram is a directed graph where the nodes are sets of elements, and the edges are
//! maps between the sets. The diagram is commutative if the following holds:

use crate::graph::{all_paths, CyclicGraphError, DiGraph, Edge};

struct Set {
    elements: Vec<usize>,
    eq: Box<dyn Fn(usize, usize) -> bool>,
    name: &'static str,
    element_name: Box<dyn Fn(usize) -> String>,
}

struct Map {
    from: usize,
    to: usize,
    map: Box<dyn Fn(usize) -> usize>,
    name: &'static str,
}

pub struct Diagram {
    sets: Vec<Set>,
    maps: Vec<Map>,
}

#[derive(Clone, Copy, Debug)]
pub struct DiagramEdge {
    from: usize,
    to: usize,
    map: usize,
}

impl Edge for DiagramEdge {
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
    type Edge = DiagramEdge;

    fn nodes(&self) -> Box<dyn Iterator<Item = Self::Node>> {
        let result: Vec<usize> = (0..self.sets.len()).collect();
        Box::new(result.into_iter())
    }

    fn outbounds(&self, node: &Self::Node) -> Box<dyn Iterator<Item = Self::Edge>> {
        let result: Vec<DiagramEdge> = self
            .maps
            .iter()
            .enumerate()
            .map(|(map_ix, map)| DiagramEdge {
                from: map.from,
                to: map.to,
                map: map_ix,
            })
            .filter(|edge| edge.from == *node)
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
                let common_destination: &<Diagram as DiGraph>::Node =
                    path_a.last().map(|e| e.to()).unwrap();

                // Find each element of the common source
                let source_set = &diagram.sets[*common_source];
                let target_set = &diagram.sets[*common_destination];

                // Now, map this source set through both of the paths
                for element in source_set.elements.clone() {
                    let mut path_a_element = element;

                    for edge in path_a {
                        let map = &diagram.maps[edge.map];
                        path_a_element = (map.map)(path_a_element);
                    }

                    let mut path_b_element = element;

                    for edge in path_b {
                        let map = &diagram.maps[edge.map];
                        path_b_element = (map.map)(path_b_element);
                    }

                    // Now, check if the two elements are equal
                    if !(target_set.eq)(path_a_element, path_b_element) {
                        // Get descriptions for both paths
                        let path_a_description = path_a
                            .iter()
                            .map(|edge| diagram.maps[edge.map].name)
                            .collect::<Vec<&str>>()
                            .join(" -> ");

                        let path_b_description = path_b
                            .iter()
                            .map(|edge| diagram.maps[edge.map].name)
                            .collect::<Vec<&str>>()
                            .join(" -> ");

                        let element_name = (source_set.element_name)(element);
                        let left_final_element_name = (target_set.element_name)(path_a_element);
                        let right_final_element_name = (target_set.element_name)(path_b_element);

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
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_addition_is_associative_on_integers() {
        use itertools::Itertools;

        // Build the full diagram
        let diagram_factory = |left_offset: i32| {
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

            // Map each downstream map back to their positions in the sets
            let pair_positions: HashMap<(i32, i32), usize> = pairs
                .clone()
                .iter()
                .enumerate()
                .map(|(ix, i)| (*i, ix))
                .collect();
            let integer_positions: HashMap<i32, usize> = integers
                .clone()
                .iter()
                .enumerate()
                .map(|(ix, i)| (*i, ix))
                .collect();

            // Build moveable copies so we can use them in the diagram
            let triplets_1 = triplets.clone();
            let pair_positions_1 = pair_positions.clone();
            let pairs_1 = pairs.clone();
            let integer_positions_1 = integer_positions.clone();

            let triplets_2 = triplets.clone();
            let pairs_2 = pairs.clone();
            let pairs_3 = pairs.clone();
            let integers_1 = integers.clone();

            Diagram {
                sets: vec![
                    Set {
                        elements: (0..triplets.len()).collect(),
                        eq: Box::new(|a, b| a == b),
                        name: "triplets",
                        element_name: Box::new(move |triplet_ix| {
                            let (a, b, c) = triplets_2[triplet_ix];
                            format!("({}, {}, {})", a, b, c)
                        }),
                    },
                    Set {
                        elements: (0..pairs.len()).collect(),
                        eq: Box::new(|a, b| a == b),
                        name: "pairs",
                        element_name: Box::new(move |pair_ix| {
                            let (a, b) = pairs_2[pair_ix];
                            format!("({}, {})", a, b)
                        }),
                    },
                    Set {
                        elements: (0..pairs.len()).collect(),
                        eq: Box::new(|a, b| a == b),
                        name: "pairs",
                        element_name: Box::new(move |pair_ix| {
                            let (a, b) = pairs_3[pair_ix];
                            format!("({}, {})", a, b)
                        }),
                    },
                    Set {
                        elements: (0..integers.len()).collect(),
                        eq: Box::new(|a, b| a == b),
                        name: "pairs",
                        element_name: Box::new(move |integer_ix| {
                            let a = integers_1[integer_ix];
                            format!("{}", a)
                        }),
                    },
                ],
                maps: vec![
                    Map {
                        from: 0,
                        to: 1,
                        map: Box::new(move |triplet_ix| {
                            let (a, b, c) = triplets_1[triplet_ix];
                            let mut pair = (a + b, c);

                            if c == 4 {
                                pair = (a + b, c + left_offset)
                            }

                            pair_positions_1[&pair]
                        }),
                        name: "(+,)",
                    },
                    Map {
                        from: 0,
                        to: 2,
                        map: Box::new(move |triplet_ix| {
                            let (a, b, c) = triplets[triplet_ix];
                            let pair = (a, b + c);

                            pair_positions[&pair]
                        }),
                        name: "(,+)",
                    },
                    Map {
                        from: 1,
                        to: 3,
                        map: Box::new(move |pair_ix| {
                            let (a, b) = pairs_1[pair_ix];

                            integer_positions_1[&(a + b)]
                        }),
                        name: "+",
                    },
                    Map {
                        from: 2,
                        to: 3,
                        map: Box::new(move |pair_ix| {
                            let (a, b) = pairs[pair_ix];

                            integer_positions[&(a + b)]
                        }),
                        name: "+",
                    },
                ],
            }
        };

        let commutative_diagram = diagram_factory(0);

        // And confirm that integer addition is associative
        assert!(match diagram_commutes(&commutative_diagram).unwrap() {
            CommutativeDiagramResult::Commutes => true,
            CommutativeDiagramResult::DoesNotCommute(reason) => panic!("{}", reason),
        });

        let non_commutative_diagram = diagram_factory(1);
        assert!(match diagram_commutes(&non_commutative_diagram).unwrap() {
            CommutativeDiagramResult::Commutes => false,
            CommutativeDiagramResult::DoesNotCommute(..) => true,
        });
    }

    #[test]
    fn test_diagram_build() {
        let d = Diagram {
            sets: vec![
                Set {
                    elements: vec![0, 1, 2],
                    eq: Box::new(|a, b| a == b),
                    name: "[0, 1, 2]",
                    element_name: Box::new(|a| a.to_string()),
                },
                Set {
                    elements: vec![0, 2],
                    eq: Box::new(|a, b| a == b),
                    name: "[0, 2]",
                    element_name: Box::new(|a| a.to_string()),
                },
                Set {
                    elements: vec![0],
                    eq: Box::new(|a, b| a == b),
                    name: "[0]",
                    element_name: Box::new(|a| a.to_string()),
                },
            ],
            maps: vec![
                Map {
                    from: 0,
                    to: 1,
                    map: Box::new(|a| a / 2),
                    name: "/2",
                },
                Map {
                    from: 1,
                    to: 2,
                    map: Box::new(|a| a / 2),
                    name: "/2",
                },
                Map {
                    from: 0,
                    to: 2,
                    map: Box::new(|_| 0),
                    name: "0",
                },
            ],
        };

        assert!(matches!(
            diagram_commutes(&d).unwrap(),
            CommutativeDiagramResult::Commutes
        ));
    }
}
