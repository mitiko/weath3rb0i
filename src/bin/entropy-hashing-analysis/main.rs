use weath3rb0i::history::{History, RawHistory};

fn main() -> std::io::Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;

    // calculate entropy at context for different level 1 histories
    let mut raw_history_stats_json = serde_json::Value::Array(Vec::new());
    let max_stats = 32;
    for bits in 0..=24 {
        let mask = (1 << bits) - 1;
        let history = MaskedHistory::new(RawHistory::new(), mask);
        // for each context, this stores entropy & bit count of level 2 history
        let entropy_counts = get_entropy(&buf, history);

        // meta structure to store everything
        let mut v = entropy_counts
            .iter()
            .enumerate()
            .map(|(ctx, &(entropy, count))| (ctx, entropy, count, entropy * count as f64))
            .collect::<Vec<_>>();
        v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let most_predictable = v
            .iter()
            .filter(|x| x.1 != 0.0)
            .take(max_stats)
            .copied()
            .collect::<Vec<_>>();
        v.sort_by(|a, b| a.2.cmp(&b.2).reverse());
        let most_common = v.iter().take(max_stats).copied().collect::<Vec<_>>();
        v.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap().reverse());
        let most_expensive = v.iter().take(max_stats).copied().collect::<Vec<_>>();
        v.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());

        let obj = serde_json::json!({
            "mask": mask,
            "most_predictable": most_predictable,
            "most_common": most_common,
            "most_expensive": most_expensive,
        });
        raw_history_stats_json.as_array_mut().unwrap().push(obj);

        let weighted_entropy = weighted_sum(entropy_counts);
        println!(
            "Bitwise order-{} has entropy: {} bits/bit",
            bits, weighted_entropy
        );
    }
    let json = serde_json::json!({
        "raw_history": raw_history_stats_json,
    });
    let json = serde_json::to_string_pretty(&json).unwrap();
    std::fs::write("stats.json", json.as_bytes())?;

    Ok(())
}

// Calculates history at each byte boundary, then calculates entropy for the level 2 history of the context
fn get_entropy(buf: &[u8], mut history: impl History) -> Vec<(f64, u32)> {
    let mut calculators = Vec::new();

    for &byte in buf.iter() {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;

            let hash = history.hash();
            if hash >= calculators.len() as u32 {
                let new_size = if hash.is_power_of_two() {
                    hash << 1
                } else {
                    hash.next_power_of_two()
                };
                calculators.resize(new_size as usize, EntropyCalculator::new());
            }
            calculators[hash as usize].update(bit);

            history.update(bit);
        }
    }

    calculators
        .into_iter()
        .map(|ec| (ec.entropy(), ec.total()))
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
    data: [u16; 2],
}

impl EntropyCalculator {
    pub fn new() -> Self {
        Self { data: [0; 2] }
    }

    pub fn entropy(&self) -> f64 {
        let c0 = f64::from(self.data[0]);
        let c1 = f64::from(self.data[1]);
        if c0 + c1 == 0.0 {
            return 0.0;
        }
        if c0 == 0.0 || c1 == 0.0 {
            return 2.0 / (c0 + c1);
        }
        let p = c1 / (c0 + c1);
        -p * p.log2() - (1.0 - p) * (1.0 - p).log2()
    }

    pub fn total(&self) -> u32 {
        u32::from(self.data[0]) + u32::from(self.data[1])
    }

    pub fn update(&mut self, bit: u8) {
        self.data[usize::from(bit)] += 1;
        if self.data[usize::from(bit)] == u16::MAX {
            self.data[0] = (self.data[0] >> 1) + (self.data[0] & 1);
            self.data[1] = (self.data[1] >> 1) + (self.data[1] & 1);
        }
    }
}
