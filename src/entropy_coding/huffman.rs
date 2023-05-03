#[derive(PartialEq, Eq)]
enum HuffmanTree {
    Leaf(u8, u32), // byte, count
    Node(Box<HuffmanTree>, Box<HuffmanTree>),
}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // inverts max-heap to be min-heap
        self.get_count().cmp(&other.get_count()).reverse()
    }
}

impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl HuffmanTree {
    pub fn from_table(table: &[u32; 256]) -> Self {
        use std::collections::BinaryHeap;
        let mut heap: BinaryHeap<_> = table
            .iter()
            .enumerate()
            .filter(|&(_, &count)| count > 0)
            .map(|(i, count)| (u8::try_from(i).unwrap(), count))
            .map(|(byte, &count)| HuffmanTree::Leaf(byte, count))
            .collect();

        while heap.len() >= 2 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();
            heap.push(HuffmanTree::Node(Box::new(left), Box::new(right)));
        }

        heap.pop().unwrap()
    }

    fn get_count(&self) -> u32 {
        match self {
            HuffmanTree::Leaf(_, count) => *count,
            HuffmanTree::Node(left, right) => left.get_count() + right.get_count(),
        }
    }
}
