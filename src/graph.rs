//! Provides graph logic, used for building diagrams
//! Note that currently, only non-cyclic graphs are supported

pub trait Edge {
    type Node: PartialEq + Clone;

    fn from(&self) -> &Self::Node;
    fn to(&self) -> &Self::Node;
}

pub trait DiGraph {
    type Node: PartialEq + Clone + std::fmt::Debug;
    type Edge: Edge<Node = Self::Node> + Clone + std::fmt::Debug;

    fn nodes(&self) -> Box<dyn Iterator<Item = Self::Node>>;
    fn outbounds(&self, node: &Self::Node) -> Box<dyn Iterator<Item = Self::Edge>>;
}

#[derive(Debug, Clone)]
pub struct CyclicGraphError;
impl std::fmt::Display for CyclicGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Graph contains at least one cycle - this is currently unsupported"
        )
    }
}

// Computes *all* paths in this graph
pub fn all_paths<G>(graph: &G) -> Result<Vec<Vec<G::Edge>>, CyclicGraphError>
where
    G: DiGraph,
{
    // Stores a list of all found paths
    let mut paths = Vec::new();

    fn search<G>(
        graph: &G,
        current_path: Vec<G::Edge>,
        paths: &mut Vec<Vec<G::Edge>>,
        initial_vertex: &G::Node,
    ) -> Result<(), CyclicGraphError>
    where
        G: DiGraph,
    {
        let current_destination = current_path.last().unwrap().to().clone();
        if current_destination == *initial_vertex {
            return Err(CyclicGraphError);
        }

        for next in graph.outbounds(&current_destination) {
            // Compute the newly found path
            let mut new_path = current_path.clone();
            new_path.push(next);

            // And store it in the result set
            paths.push(new_path.clone());

            // Now continue the search from our new position
            search(graph, new_path, paths, initial_vertex)?;
        }

        Ok(())
    }

    // Initiate the search from each vertex
    for initial_vertex in graph.nodes() {
        // Initiate the search from this vertex through each outbound
        for outbound in graph.outbounds(&initial_vertex) {
            paths.push(vec![outbound.clone()]);

            // The search will modify the list of discovered paths in-place
            search(graph, vec![outbound], &mut paths, &initial_vertex)?;
        }
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct TestGraphEdge {
        from: u32,
        to: u32,
    }

    impl Edge for TestGraphEdge {
        type Node = u32;

        fn from(&self) -> &Self::Node {
            &self.from
        }

        fn to(&self) -> &Self::Node {
            &self.to
        }
    }

    struct TestGraph {
        nodes: Vec<u32>,
        edges: Vec<TestGraphEdge>,
    }

    impl DiGraph for TestGraph {
        type Node = u32;
        type Edge = TestGraphEdge;

        fn nodes(&self) -> Box<dyn Iterator<Item = Self::Node>> {
            let result: Vec<u32> = self.nodes.clone();
            Box::new(result.into_iter())
        }

        fn outbounds(&self, node: &Self::Node) -> Box<dyn Iterator<Item = Self::Edge>> {
            let result: Vec<TestGraphEdge> = self
                .edges
                .iter()
                .filter(move |TestGraphEdge { from, to: _ }| from == node)
                .cloned()
                .collect();

            Box::new(result.into_iter())
        }
    }

    #[test]
    fn test_all_paths_on_example_graph() {
        let graph = TestGraph {
            nodes: vec![1, 2, 3, 4],
            edges: vec![
                TestGraphEdge { from: 1, to: 2 },
                TestGraphEdge { from: 1, to: 3 },
                TestGraphEdge { from: 2, to: 3 },
                TestGraphEdge { from: 2, to: 4 },
                TestGraphEdge { from: 3, to: 4 },
            ],
        };

        // Fetch all paths through the graph
        let paths = all_paths(&graph).unwrap();

        // Test one: all paths should be valid
        for path in paths.clone() {
            for edge in path {
                assert!(graph.outbounds(edge.from()).any(|e| e.to() == edge.to()));
            }
        }

        // Test two: some sample paths should be present to test various cases
        assert!(paths.contains(&vec![
            TestGraphEdge { from: 1, to: 2 },
            TestGraphEdge { from: 2, to: 3 },
            TestGraphEdge { from: 3, to: 4 },
        ]));
        assert!(paths.contains(&vec![
            TestGraphEdge { from: 1, to: 2 },
            TestGraphEdge { from: 2, to: 4 },
        ]));
        assert!(paths.contains(&vec![
            TestGraphEdge { from: 1, to: 3 },
            TestGraphEdge { from: 3, to: 4 },
        ]));
        assert!(paths.contains(&vec![
            TestGraphEdge { from: 2, to: 3 },
            TestGraphEdge { from: 3, to: 4 },
        ]));
        assert!(paths.contains(&vec![TestGraphEdge { from: 2, to: 4 },]));
    }
}
