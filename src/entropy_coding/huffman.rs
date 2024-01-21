use std::collections::VecDeque;
use HuffmanTree::*;

#[derive(PartialEq, Eq, Debug, Ord)]
pub enum HuffmanTree {
    /// byte, count
    Leaf(u8, u32), // TODO: allow 16-bit symbols
    Node(Box<HuffmanTree>, Box<HuffmanTree>),
}

// TODO: Length limit the codes
impl HuffmanTree {
    pub fn from_histogram(histogram: &[u32; 256]) -> Self {
        use std::collections::BinaryHeap;
        let mut heap: BinaryHeap<_> = histogram
            .iter()
            .enumerate()
            .filter(|&(_, &count)| count > 0)
            .map(|(i, count)| (u8::try_from(i).unwrap(), count))
            .map(|(byte, &count)| HuffmanTree::Leaf(byte, count))
            .collect();

        while heap.len() >= 2 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();
            heap.push(Node(Box::new(left), Box::new(right)));
        }

        heap.pop().unwrap()
    }

    fn get_count(&self) -> u32 {
        // does DFS sumation O(log n) depth, can be cached
        match self {
            Leaf(_, count) => *count,
            Node(left, right) => left.get_count() + right.get_count(),
        }
    }

    // TODO: optimize table creation?
    pub fn to_table_encoder(&self) -> HuffmanTableEncoder {
        let mut codes = [0; 256];
        let mut lengths = [0; 256];
        let mut bfs = VecDeque::new();
        bfs.push_back((self, 0, 0));

        while let Some((node, len, code)) = bfs.pop_front() {
            match node {
                Leaf(byte, _) => {
                    codes[usize::from(*byte)] = code;
                    lengths[usize::from(*byte)] = len;
                }
                Node(left, right) => {
                    bfs.push_back((left, len + 1, code << 1));
                    bfs.push_back((right, len + 1, (code << 1) | 1));
                }
            }
        }
        HuffmanTableEncoder { codes, lengths }
    }

    // TODO: Table decoder (can do up to 5 iterations with 64-bit buffer)
    pub fn to_tree_decoder(&self) -> HuffmanTreeDecoder {
        HuffmanTreeDecoder { root: self, node: self }
    }
}

impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // inverts max-heap to be min-heap
        Some(self.get_count().cmp(&other.get_count()).reverse())
    }
}

// TODO: store LSB version of code if needed
pub struct HuffmanTableEncoder {
    codes: [u32; 256],
    lengths: [u8; 256],
}

impl HuffmanTableEncoder {
    pub fn encode(&self, byte: u8) -> (u32, u8) {
        let byte = usize::from(byte);
        (self.codes[byte], self.lengths[byte])
    }
}

pub struct HuffmanTreeDecoder<'a> {
    root: &'a HuffmanTree,
    node: &'a HuffmanTree,
}

impl<'a> HuffmanTreeDecoder<'a> {
    pub fn update(&mut self, bit: u8) {
        self.node = match (self.node, self.root, bit) {
            (Node(left, __), _, 0) | (_, Node(left, __), 0) => left,
            (Node(_, right), _, 1) | (_, Node(_, right), 1) => right,
            _ => self.root, // single byte streams, with 0-length codes
        };
    }

    pub fn decode(&mut self) -> Option<u8> {
        match self.node {
            Leaf(byte, _) => Some(*byte),
            Node(..) => None,
        }
    }
}
