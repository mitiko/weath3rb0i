use std::collections::VecDeque;

#[derive(PartialEq, Eq)]
pub enum HuffmanTree {
    /// byte, count
    Leaf(u8, u32),
    Node(Box<HuffmanTree>, Box<HuffmanTree>),
}

pub struct HuffmanEncodeTable {
    codes: [u32; 256],
    lengths: [u8; 256],
}

pub struct HuffmanDecodeTable {
    bytes: [u8; 256],
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
        // does DFS sumation O(log n) depth, can be cached
        match self {
            HuffmanTree::Leaf(_, count) => *count,
            HuffmanTree::Node(left, right) => left.get_count() + right.get_count(),
        }
    }

    pub fn to_encode_table(&self) -> HuffmanEncodeTable {
        let mut codes = [0; 256];
        let mut lengths = [0; 256];
        let mut bfs = VecDeque::new();
        bfs.push_back((self, 0, 0));

        while let Some((node, len, code)) = bfs.pop_front() {
            match node {
                HuffmanTree::Leaf(byte, _) => {
                    codes[usize::from(*byte)] = code;
                    lengths[usize::from(*byte)] = len;
                }
                HuffmanTree::Node(left, right) => {
                    bfs.push_back((left, len + 1, code << 1));
                    bfs.push_back((right, len + 1, (code << 1) | 1));
                }
            }
        }
        HuffmanEncodeTable { codes, lengths }
    }

    pub fn to_decode_table(&self) -> HuffmanDecodeTable {
        // also how do we read it from stream?
        // encode the frequencies or the table?
        todo!();
    }
}

impl HuffmanEncodeTable {
    pub fn encode(&self, byte: u8) -> (u32, u8) {
        let byte = usize::from(byte);
        (self.codes[byte], self.lengths[byte])
    }
}
