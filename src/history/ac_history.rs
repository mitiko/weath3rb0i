use crate::{
    entropy_coding::arithmetic_coder::ArithmeticCoder,
    history::{EntropyWriter, History},
    models::Model,
    u8,
};

pub struct ACHistory<M: Model> {
    ac: ArithmeticCoder<EntropyWriter>,
    writer: EntropyWriter,
    model: M,
}

impl<M: Model> ACHistory<M> {
    pub fn new(model: M) -> Self {
        Self {
            model,
            ac: ArithmeticCoder::new_coder(),
            writer: EntropyWriter::new(),
        }
    }
}

impl<M: Model> History for ACHistory<M> {
    fn update(&mut self, bit: u8) {
        let p = self.model.predict();
        self.ac.encode(bit, p, &mut self.writer).unwrap();
        self.model.update(bit);
    }

    fn hash(&mut self) -> u32 {
        self.writer.state
    }
}
