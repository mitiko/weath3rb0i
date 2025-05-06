use weath3rb0i::history::{History, RawHistory};

fn main() -> std::io::Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;

    // calculate entropy at context for different level 1 histories
    let mut entropy_calculator = EntropyCalculator::new();
    buf.iter().for_each(|&byte| entropy_calculator.add(byte));
    println!("Initial entropy: {}", entropy_calculator.entropy());

    for bits in 0..=16 {
        let mask = (1 << bits) - 1;
        let history = MaskedHistory::new(RawHistory::new(), mask);
        let entropy_counts = get_entropy(&buf, history);
        let weighted_entropy = weighted_sum(entropy_counts);
        println!("Entropy for last {} bits: {}", bits, weighted_entropy);
    }

    Ok(())
}

// Calculates history at each byte boundary, then calculates entropy for the level 2 history of the context
fn get_entropy(buf: &[u8], mut history: impl History) -> Vec<(f64, u32)> {
    let mut calculators = Vec::new();

    for (&curr, &next) in buf.iter().zip(buf.iter().skip(1)) {
        for i in (0..8).rev() {
            let bit = (curr >> i) & 1;
            history.update(bit);
        }
        let hash = history.hash();
        if hash >= calculators.len() as u32 {
            let new_size = if hash.is_power_of_two() { hash << 1 } else { hash.next_power_of_two() };
            calculators.resize(new_size as usize, EntropyCalculator::new());
        }
        calculators[hash as usize].add(next);
    }

    calculators
        .into_iter()
        .map(|ec| (ec.entropy(), ec.counts.len() as u32))
        .collect()
}

fn weighted_sum(entropy_counts: Vec<(f64, u32)>) -> f64 {
    let total_count = entropy_counts.iter().map(|&(_, count)| count).sum::<u32>() as f64;
    entropy_counts
        .iter()
        .map(|&(entropy, count)| entropy * (count as f64 / total_count))
        .sum()
}

struct MaskedHistory<H: History> {
    history: H,
    mask: u32,
}

impl<H: History> MaskedHistory<H> {
    fn new(history: H, mask: u32) -> Self {
        Self { history, mask }
    }
}

impl<H: History> History for MaskedHistory<H> {
    fn update(&mut self, bit: u8) {
        self.history.update(bit);
    }

    fn hash(&mut self) -> u32 {
        self.history.hash() & self.mask
    }
}

#[derive(Clone)]
struct EntropyCalculator {
    counts: Vec<u32>,
}

impl EntropyCalculator {
    fn new() -> Self {
        Self { counts: vec![0; 256] }
    }

    fn add(&mut self, byte: u8) {
        self.counts[byte as usize] += 1;
    }

    fn entropy(&self) -> f64 {
        let total = self.counts.iter().sum::<u32>() as f64;
        self.counts
            .iter()
            .filter(|&&count| count > 0)
            .map(|&count| {
                let p = count as f64 / total;
                -p * p.log2()
            })
            .sum()
    }
}
