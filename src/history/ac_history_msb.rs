use super::History;
use crate::{
    entropy_coding::arithmetic_coder::{ACWrite, ArithmeticCoder},
    helpers::RotatingBuffer,
    models::Model,
    u8, Analytics,
};

/// An entropy-coded history whose bits are arranged in MSB-first byte order.
///
/// `ACHistory` visits the history one bit at a time, starting with the most
/// recently received bit.  That makes the bits within every byte appear in
/// reverse order.  This variant starts with the partial (most recent) byte,
/// but visits the bits in that byte in their original order.  It then visits
/// each preceding byte, also MSB first.
#[derive(Clone)]
pub struct ACHistoryMSB<M: Model> {
    pos: usize,
    bits: u64,
    probs: RotatingBuffer<u16, 64>,
    max_bits: u8,
    compressed_bits_count: usize,
    model: M,
}

impl<M: Model> ACHistoryMSB<M> {
    pub fn new(max_bits: u8, model: M) -> Self {
        const HALF: u16 = 1 << 15;
        Self {
            pos: 0,
            bits: 0,
            probs: RotatingBuffer::init(HALF),
            max_bits,
            compressed_bits_count: 0,
            model,
        }
    }
}

/// Return the index in the rotating history for the `ordinal`th bit in the
/// MSB-first view of the history.
///
/// The rotating buffer is indexed from the newest bit: index zero is the
/// last bit received.  The first group is therefore the partial current byte
/// in descending buffer-index order.  Full preceding bytes are also
/// descending, with the groups themselves ordered from newest to oldest.
#[inline]
fn history_index(ordinal: usize, partial_bits: usize) -> usize {
    if ordinal < partial_bits {
        partial_bits - ordinal - 1
    } else {
        let previous = ordinal - partial_bits;
        let byte = previous / 8;
        let bit = previous % 8;
        partial_bits + byte * 8 + 7 - bit
    }
}

impl<M: Model> History for ACHistoryMSB<M> {
    fn update(&mut self, bit: u8) {
        let p = self.model.predict();
        self.model.update(bit);
        self.probs.push(p);

        self.bits = (self.bits << 1) | u64::from(bit);
        self.pos += 1;
        self.compressed_bits_count = 0;
    }

    fn hash(&mut self) -> u32 {
        let mut ac = ArithmeticCoder::new_coder();
        let mut writer = EntropyWriter {
            state: 0,
            rev_bits: 0,
            idx: 0,
            max_bits: self.max_bits,
        };

        // A zero remainder means that the most recent byte is complete, so
        // its size in the first group is eight rather than zero.
        let partial_bits = match self.pos % 8 {
            0 => 8,
            bits => bits,
        };

        for ordinal in 0..self.pos.min(64) {
            let index = history_index(ordinal, partial_bits);
            // With an unaligned history, the next complete MSB-first byte
            // can begin before the 64-bit raw window.  Do not read beyond
            // the rotating buffer just to fill the output to 64 bits.
            if index >= 64 {
                break;
            }
            let bit = u8!((self.bits >> index) & 1);
            let p = self.probs[index];
            if ac.encode(bit, p, &mut writer).is_err() {
                self.compressed_bits_count = ordinal + 1;
                break;
            }
        }

        writer.state.wrapping_shr(32 - writer.idx as u32)
    }
}

#[derive(Clone, Debug)]
struct EntropyWriter {
    state: u32,
    max_bits: u8,
    rev_bits: u16,
    idx: u8,
}

impl ACWrite for EntropyWriter {
    fn write_bit(&mut self, bit: impl TryInto<u8>) -> std::io::Result<()> {
        debug_assert!(self.idx <= self.max_bits);
        use std::io::{Error, ErrorKind};
        let bit = bit.try_into().unwrap_or_default();

        let mut write_bit_raw = |bit: u8| -> std::io::Result<()> {
            if self.idx == self.max_bits {
                return Err(Error::from(ErrorKind::Other));
            }

            self.state = (self.state >> 1) | (u32::from(bit) << 31);
            self.idx += 1;
            Ok(())
        };

        write_bit_raw(bit)?;
        while self.rev_bits > 0 {
            self.rev_bits -= 1;
            write_bit_raw(bit ^ 1)?;
        }

        Ok(())
    }

    fn inc_parity(&mut self) {
        self.rev_bits += 1;
    }

    fn flush(&mut self, _padding: u32) -> std::io::Result<()> {
        self.write_bit(1)?;
        debug_assert!(self.rev_bits == 0);
        Ok(())
    }
}

impl<M: Model + Analytics> Analytics for ACHistoryMSB<M> {
    fn log(&mut self) -> serde_json::Value {
        serde_json::json!({
            "h": self.hash(),
            "bits": self.compressed_bits_count,
            "model": self.model.log(),
        })
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "history/ACHistoryMSB",
            "children": {
                "model": self.model.metadata(),
            },
            "vars": { "bits": "usize", },
            "description": "Compressed history with arithmetic coding using a static model in MSB-first byte order",
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn history_indices_are_msb_first_by_byte() {
        // One bit in the current byte, followed by two complete preceding
        // bytes: current, then 8..1, then 16..9.
        let indices = (0..17)
            .map(|i| super::history_index(i, 1))
            .collect::<Vec<_>>();
        assert_eq!(
            indices,
            vec![0, 8, 7, 6, 5, 4, 3, 2, 1, 16, 15, 14, 13, 12, 11, 10, 9]
        );

        // Two bits in the current byte: current bits are 1,0, followed by
        // the preceding byte 9..2.
        let indices = (0..10)
            .map(|i| super::history_index(i, 2))
            .collect::<Vec<_>>();
        assert_eq!(indices, vec![1, 0, 9, 8, 7, 6, 5, 4, 3, 2]);
    }
}
