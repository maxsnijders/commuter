//! Provides graph logic, used for building diagrams
//! Note that currently, only non-cyclic graphs are supported

trait DiGraph {
    type Node: PartialEq + Clone;

    fn next(&self, node: &Self::Node) -> Box<dyn Iterator<Item=Self::Node>>;

    /// Returns all non-cyclic paths from start to end
    fn all_paths(&self, start: &Self::Node, end: &Self::Node) -> Vec<Vec<Self::Node>> {
        let mut paths = Vec::new();
        let mut stack = Vec::new();
        stack.push(vec![start.clone()]);

        while let Some(path) = stack.pop() {
            let last = path.last().unwrap();
            if *last == *end {
                paths.push(path);
            } else {
                for next in self.next(last) {
                    let mut new_path = path.clone();
                    new_path.push(next);
                    stack.push(new_path);
                }
            }
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestGraph {
        edges: Vec<(usize, usize)>,
    }

    impl DiGraph for TestGraph {
        type Node = usize;

        fn next(&self, node: &usize) -> Box<dyn Iterator<Item=usize>> {
            let response: Vec<usize> = self.edges.iter()
            .filter(move |(from, _)| *from == *node)
            .map(|(_, to)| *to).collect();
            
            Box::new(response.into_iter())
        }
    }

    impl TestGraph {
        fn new(edges: Vec<(usize, usize)>) -> Self {
            Self { edges }
        }
    }

    #[test]
    fn test_all_paths() {
        let graph = TestGraph::new(vec![(1, 2), (1, 3), (2, 4), (3, 4), (4, 5)]);
        let mut paths = graph.all_paths(&1, &5);
        paths.sort_by(|a, b| a.cmp(b));

        assert_eq!(paths, vec![vec![1, 2, 4, 5], vec![1, 3, 4, 5]]);
    }
}