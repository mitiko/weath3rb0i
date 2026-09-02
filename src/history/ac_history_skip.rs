use super::{ac_history::EntropyWriter, History};
use crate::{
    entropy_coding::arithmetic_coder::ArithmeticCoder, helpers::RotatingBuffer, models::Model, u8,
};

/// Selects which history bits are worth encoding into the compressed hash.
#[derive(Clone, Copy, Debug)]
pub enum HistorySkipCondition {
    /// Skips predictions near 0.5.
    Probability { lower: f64, upper: f64 },
    /// Encode when bit cost is within the inclusive range.
    Entropy { lower: f64, upper: f64 },
    /// Encode when bit cost is outside the range.
    EntropyExclusive { lower: f64, upper: f64 },
}

impl HistorySkipCondition {
    /// Returns whether the bit should be encoded.
    #[inline]
    pub fn evaluate(&self, bit: u8, prediction: u16) -> bool {
        let prediction = f64::from(prediction) / 65536.0;
        let probability = if bit == 0 {
            1.0 - prediction
        } else {
            prediction
        };

        match *self {
            Self::Probability { lower, upper } => {
                if bit == 0 {
                    prediction > upper
                } else {
                    prediction < lower
                }
            }
            Self::Entropy { lower, upper } => {
                let entropy = -probability.log2();
                entropy >= lower && entropy <= upper
            }
            Self::EntropyExclusive { lower, upper } => {
                let entropy = -probability.log2();
                entropy < lower || entropy > upper
            }
        }
    }
}

const SKIP_HISTORY_BITS: usize = 1024;
const SKIP_HISTORY_WORDS: usize = SKIP_HISTORY_BITS / u64::BITS as usize;

/// An entropy-coded history that only includes bits selected by a
/// [`HistorySkipCondition`]. Bits are kept in LSB order, with the newest bit
/// at index zero.
#[derive(Clone)]
pub struct ACHistorySkip<M: Model> {
    pos: usize,
    bits: [u64; SKIP_HISTORY_WORDS],
    probs: RotatingBuffer<u16, SKIP_HISTORY_BITS>,
    processed_bits: usize,
    condition: HistorySkipCondition,
    model: M,
}

impl<M: Model> ACHistorySkip<M> {
    pub fn new(model: M, condition: HistorySkipCondition) -> Self {
        const HALF: u16 = 1 << 15;
        Self {
            pos: 0,
            bits: [0; SKIP_HISTORY_WORDS],
            probs: RotatingBuffer::init(HALF),
            processed_bits: 0,
            condition,
            model,
        }
    }
}

impl<M: Model> History for ACHistorySkip<M> {
    fn update(&mut self, bit: u8) {
        let p = self.model.predict();
        self.model.update(bit);
        self.probs.push(p);

        for word in (1..self.bits.len()).rev() {
            self.bits[word] = (self.bits[word] << 1) | (self.bits[word - 1] >> 63);
        }
        self.bits[0] = (self.bits[0] << 1) | u64::from(bit);
        self.pos += 1;
        self.processed_bits = 0;
    }

    fn hash(&mut self, max_bits: u8) -> u32 {
        let mut ac = ArithmeticCoder::new_coder();
        let mut writer = EntropyWriter::new(max_bits);

        for i in 0..self.pos.min(SKIP_HISTORY_BITS) {
            let bit = u8!((self.bits[i / 64] >> (i % 64)) & 1);
            let p = self.probs[i];
            if !self.condition.evaluate(bit, p) {
                continue;
            }
            if ac.encode(bit, p, &mut writer).is_err() {
                self.processed_bits = i + 1;
                break;
            }
        }

        writer.state.wrapping_shr(32 - writer.idx as u32)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn history_skip_conditions_evaluate_observed_bit_cost() {
        let probability = super::HistorySkipCondition::Probability { lower: 0.4, upper: 0.6 };
        assert!(probability.evaluate(1, 16_384));
        assert!(!probability.evaluate(1, 32_768));
        assert!(!probability.evaluate(1, 49_152));
        assert!(probability.evaluate(0, 49_152));
        assert!(!probability.evaluate(0, 32_768));
        assert!(!probability.evaluate(0, 16_384));

        let entropy = super::HistorySkipCondition::Entropy { lower: 1.0, upper: f64::INFINITY };
        assert!(entropy.evaluate(1, 16_384));
        assert!(!entropy.evaluate(1, 49_152));

        let exclusive = super::HistorySkipCondition::EntropyExclusive { lower: 0.5, upper: 1.0 };
        assert!(exclusive.evaluate(1, 49_152));
        assert!(!exclusive.evaluate(1, 32_768));
        assert!(exclusive.evaluate(1, 16_384));
    }
}
