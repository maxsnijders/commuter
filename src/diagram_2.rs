use std::any::Any;

trait Element {
    fn eq(&self, other: &Box<dyn Element>) -> bool;
    fn as_any(&self) -> &dyn Any;
}

trait Set<'a> {
    fn elements(&self) -> Box<dyn Iterator<Item=Box<dyn Element +'a>> + 'a>;
}

trait Map {
    fn map(&self, key: &Box<dyn Element>) -> Option<Box<dyn Element>>;
}

// struct Diagram {
//     sets: Vec<Box<dyn Set<>>,
//     maps: Vec<(usize, usize, Box<dyn Map>)>
// }

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

    #[derive(Clone, Copy, PartialEq)]
    struct IntElement {
        value: i32
    }

    impl Element for IntElement {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn eq(&self, other: &Box<dyn Element>) -> bool {
            if let Some(other) = other.as_any().downcast_ref::<IntElement>() {
                self.value == other.value
            } else {
                false
            }
        }
    }

    struct TypeSet<T> where T: Clone + Element + Sized{
        elements: Vec<T>
    }

    impl<'a, T> Set<'a> for TypeSet<T> where T : Clone + Element + Sized + 'a{
        fn elements(&self) -> Box<dyn Iterator<Item=Box<dyn Element + 'a>> + 'a>{
            let owned_els = self.elements.clone();
            Box::new(owned_els.into_iter().map(|e| Box::new(e.clone()) as Box<dyn Element>))
        }
    }

    

    #[test]
    fn test_addition_is_associative_on_integers() {
        use itertools::Itertools;

        let generating_integers: Vec<i32> = (0..20).collect();

        

        // Build the sets (triplets, pairs, integers) so that summing operations can be well-defined later
        let triplets: Vec<(i32, i32, i32)> = generating_integers.clone().iter().cartesian_product(generating_integers.clone()).cartesian_product(generating_integers.clone().iter()).map(|((a, b), c)| (*a, b, *c)).collect();
        let pairs: Vec<(i32, i32)> = (0..40).cartesian_product(0..40).collect();
        let integers: Vec<i32> = (0..80).collect();    
    
    }
}