use std::cell::RefCell;

#[derive(Copy, Clone, Debug)]
enum Set {
    Delegate { index: usize },
    End { size: usize },
}

#[derive(Debug)]
pub struct Sets {
    elements: RefCell<Vec<Set>>,
}

impl Sets {
    pub fn new(n: usize) -> Sets {
        Sets {
            elements: RefCell::new(vec![Set::End { size: 1 }; n]),
        }
    }

    /// Join together sets `a` and `b`, and return the number of
    /// members in the combined set.
    pub fn join(&mut self, a: usize, b: usize) -> usize {
        let sets = self.elements.get_mut();
        let (a, a_size) = compress_path(sets, a);
        let (b, b_size) = compress_path(sets, b);
        if a == b {
            return a_size;
        }

        let new_size = b_size + a_size;
        assert!(matches!(sets[a], Set::End { .. }));
        assert!(matches!(sets[b], Set::End { .. }));
        sets[a] = Set::Delegate { index: b };
        sets[b] = Set::End { size: new_size };
        new_size
    }

    pub fn sets(&self) -> Vec<usize> {
        self.elements
            .borrow()
            .iter()
            .enumerate()
            .filter_map(|(index, set)| match set {
                Set::Delegate { .. } => None,
                Set::End { .. } => Some(index),
            })
            .collect()
    }

    pub fn size(&self, n: usize) -> usize {
        let mut sets = self.elements.borrow_mut();
        let (rep, size) = compress_path(&mut sets, n);
        size
    }
}

fn compress_path(reps: &mut [Set], n: usize) -> (usize, usize) {
    match reps[n] {
        Set::End { size } => (n, size),
        Set::Delegate { index } => {
            let (ultimate, size) = compress_path(reps, index);
            reps[n] = Set::Delegate { index: ultimate };
            (ultimate, size)
        }
    }
}
