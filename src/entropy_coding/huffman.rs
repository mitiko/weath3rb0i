use std::collections::VecDeque;
use HuffmanTree::*;

#[derive(PartialEq, Eq, Debug)]
pub enum HuffmanTree {
    /// byte, count
    Leaf(u8, u32),
    Node(Box<HuffmanTree>, Box<HuffmanTree>),
}

pub struct HuffmanEncoder {
    codes: [u32; 256],
    lengths: [u8; 256],
}

pub struct HuffmanDecoder<'a> {
    root: &'a HuffmanTree,
    node: &'a HuffmanTree
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

    pub fn to_encode_table(&self) -> HuffmanEncoder {
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
        HuffmanEncoder { codes, lengths }
    }

    pub fn to_decode_table(&self) -> HuffmanDecoder {
        HuffmanDecoder { root: self, node: self }
    }
}

impl HuffmanEncoder {
    pub fn encode(&self, byte: u8) -> (u32, u8) {
        let byte = usize::from(byte);
        (self.codes[byte], self.lengths[byte])
    }
}

impl<'a> HuffmanDecoder<'a> {
    pub fn update(&mut self, bit: u8) {
        // println!("decoding bit {bit}");
        self.node = match (self.node, self.root, bit) {
            (Node(left, _right), _, 0) => {
                // println!("node left");
                left
            },
            (Node(_left, right), _, _) => {
                // println!("node right");
                right
            },
            (_, Node(left, _right), 0) => {
                // println!("leaf left");
                left
            },
            (_, Node(_left, right), _) => {
                // println!("leaf right");
                right
            },
            _ => self.root, // single byte streams
        };
        // println!("moved to {:?}", self.node);
    }

    pub fn decode(&mut self) -> Option<u8> {
        match self.node {
            Leaf(byte, _) => Some(*byte),
            Node(..) => None,
        }
    }
}
