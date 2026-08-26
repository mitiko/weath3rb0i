use std::{
    io::Result,
    time::{Duration, Instant},
};
use weath3rb0i::{
    entropy_coding::arithmetic_coder::ArithmeticCoder, helpers::ACStats, history::*, models::*,
    unroll_for,
};

/// Best of 3 runs, so the timing reflects a warm cache. The model expression
/// is re-evaluated per run to start each one from a clean counter.
macro_rules! best_of3 {
    ($buf:expr, $label:expr, $history:expr, $model:expr) => {
        let additional_bytes = if $label.contains("huff") {
            4 * 256
        } else if $label.contains("ac") {
            2 * 256
        } else {
            0
        };
        let (csize, time) = (0..3)
            .map(|_| compress_ctx($buf, $history, $model).unwrap())
            .min_by(|(_, t1), (_, t2)| t1.cmp(t2))
            .unwrap();
        println!("{:<22} csize: {:>9}, time: {:?}", $label, csize + additional_bytes, time);
    };
    ($buf:expr, $label:expr, $model:expr) => {
        let (csize, time) = (0..3)
            .map(|_| compress($buf, $model).unwrap())
            .min_by(|(_, t1), (_, t2)| t1.cmp(t2))
            .unwrap();
        println!("{:<22} csize: {:>9}, time: {:?}", $label, csize, time);
    };
}

type GC5 = GeometricCounter::<5>;
type GC9 = GeometricCounter::<9>;

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;
    // 4*256 bytes for Huff
    // 2*256 bytes for AC

    // history: raw, huff, ac
    // counter: linear, exp, FSM
    // model: pm5, pm8, pm9, pm13, pm16

    best_of3!(&buf, "[h=raw, m=pm5,  c=lc4]", PrefixModel5::new(Counter4::new()));
    best_of3!(&buf, "[h=raw, m=pm8,  c=lc4]", PrefixModel8::new(Counter4::new()));
    best_of3!(&buf, "[h=raw, m=pm9,  c=lc4]", PrefixModel9::new(Counter4::new()));
    best_of3!(&buf, "[h=raw, m=pm13, c=lc4]", PrefixModel13::new(Counter4::new()));
    best_of3!(&buf, "[h=raw, m=pm16, c=lc4]", PrefixModel16::new(Counter4::new()));
    best_of3!(&buf, "[h=raw, m=pm5,  c=gc5]", PrefixModel5::new(GC5::new()));
    best_of3!(&buf, "[h=raw, m=pm8,  c=gc5]", PrefixModel8::new(GC5::new()));
    best_of3!(&buf, "[h=raw, m=pm9,  c=gc5]", PrefixModel9::new(GC5::new()));
    best_of3!(&buf, "[h=raw, m=pm13, c=gc5]", PrefixModel13::new(GC5::new()));
    best_of3!(&buf, "[h=raw, m=pm16, c=gc5]", PrefixModel16::new(GC5::new()));
    best_of3!(&buf, "[h=raw, m=pm5,  c=gc9]", PrefixModel5::new(GC9::new()));
    best_of3!(&buf, "[h=raw, m=pm8,  c=gc9]", PrefixModel8::new(GC9::new()));
    best_of3!(&buf, "[h=raw, m=pm9,  c=gc9]", PrefixModel9::new(GC9::new()));
    best_of3!(&buf, "[h=raw, m=pm13, c=gc9]", PrefixModel13::new(GC9::new()));
    best_of3!(&buf, "[h=raw, m=pm16, c=gc9]", PrefixModel16::new(GC9::new()));
    best_of3!(&buf, "[h=raw, m=pm5,  c=fsm]", PrefixModel5::new(FSM0::new()));
    best_of3!(&buf, "[h=raw, m=pm8,  c=fsm]", PrefixModel8::new(FSM0::new()));
    best_of3!(&buf, "[h=raw, m=pm9,  c=fsm]", PrefixModel9::new(FSM0::new()));
    best_of3!(&buf, "[h=raw, m=pm13, c=fsm]", PrefixModel13::new(FSM0::new()));
    best_of3!(&buf, "[h=raw, m=pm16, c=fsm]", PrefixModel16::new(FSM0::new()));

    let h = HuffHistory::new(&buf, 16, 16);
    best_of3!(&buf, "[h=huff, m=pm5,  c=lc4]", h.clone(), CtxPrefixModel5::new(Counter4::new()));
    best_of3!(&buf, "[h=huff, m=pm8,  c=lc4]", h.clone(), CtxPrefixModel8::new(Counter4::new()));
    best_of3!(&buf, "[h=huff, m=pm9,  c=lc4]", h.clone(), CtxPrefixModel9::new(Counter4::new()));
    best_of3!(&buf, "[h=huff, m=pm13, c=lc4]", h.clone(), CtxPrefixModel13::new(Counter4::new()));
    best_of3!(&buf, "[h=huff, m=pm16, c=lc4]", h.clone(), CtxPrefixModel16::new(Counter4::new()));
    best_of3!(&buf, "[h=huff, m=pm5,  c=gc5]", h.clone(), CtxPrefixModel5::new(GC5::new()));
    best_of3!(&buf, "[h=huff, m=pm8,  c=gc5]", h.clone(), CtxPrefixModel8::new(GC5::new()));
    best_of3!(&buf, "[h=huff, m=pm9,  c=gc5]", h.clone(), CtxPrefixModel9::new(GC5::new()));
    best_of3!(&buf, "[h=huff, m=pm13, c=gc5]", h.clone(), CtxPrefixModel13::new(GC5::new()));
    best_of3!(&buf, "[h=huff, m=pm16, c=gc5]", h.clone(), CtxPrefixModel16::new(GC5::new()));
    best_of3!(&buf, "[h=huff, m=pm5,  c=gc9]", h.clone(), CtxPrefixModel5::new(GC9::new()));
    best_of3!(&buf, "[h=huff, m=pm8,  c=gc9]", h.clone(), CtxPrefixModel8::new(GC9::new()));
    best_of3!(&buf, "[h=huff, m=pm9,  c=gc9]", h.clone(), CtxPrefixModel9::new(GC9::new()));
    best_of3!(&buf, "[h=huff, m=pm13, c=gc9]", h.clone(), CtxPrefixModel13::new(GC9::new()));
    best_of3!(&buf, "[h=huff, m=pm16, c=gc9]", h.clone(), CtxPrefixModel16::new(GC9::new()));
    best_of3!(&buf, "[h=huff, m=pm5,  c=fsm]", h.clone(), CtxPrefixModel5::new(FSM0::new()));
    best_of3!(&buf, "[h=huff, m=pm8,  c=fsm]", h.clone(), CtxPrefixModel8::new(FSM0::new()));
    best_of3!(&buf, "[h=huff, m=pm9,  c=fsm]", h.clone(), CtxPrefixModel9::new(FSM0::new()));
    best_of3!(&buf, "[h=huff, m=pm13, c=fsm]", h.clone(), CtxPrefixModel13::new(FSM0::new()));
    best_of3!(&buf, "[h=huff, m=pm16, c=fsm]", h.clone(), CtxPrefixModel16::new(FSM0::new()));

    let model = PrefixModel8::new(Counter4::new());
    let model = model.freeze();
    // let mut model = FrozenModel::new(RawModel::new(model));
    // model.train(&buf);
    let ac = ACHistory::new(15, model);

    best_of3!(&buf, "[h=ac, m=pm5,  c=lc4]", ac.clone(), CtxPrefixModel5::new(Counter4::new()));
    best_of3!(&buf, "[h=ac, m=pm8,  c=lc4]", ac.clone(), CtxPrefixModel8::new(Counter4::new()));
    best_of3!(&buf, "[h=ac, m=pm9,  c=lc4]", ac.clone(), CtxPrefixModel9::new(Counter4::new()));
    best_of3!(&buf, "[h=ac, m=pm13, c=lc4]", ac.clone(), CtxPrefixModel13::new(Counter4::new()));
    best_of3!(&buf, "[h=ac, m=pm16, c=lc4]", ac.clone(), CtxPrefixModel16::new(Counter4::new()));
    best_of3!(&buf, "[h=ac, m=pm5,  c=gc5]", ac.clone(), CtxPrefixModel5::new(GC5::new()));
    best_of3!(&buf, "[h=ac, m=pm8,  c=gc5]", ac.clone(), CtxPrefixModel8::new(GC5::new()));
    best_of3!(&buf, "[h=ac, m=pm9,  c=gc5]", ac.clone(), CtxPrefixModel9::new(GC5::new()));
    best_of3!(&buf, "[h=ac, m=pm13, c=gc5]", ac.clone(), CtxPrefixModel13::new(GC5::new()));
    best_of3!(&buf, "[h=ac, m=pm16, c=gc5]", ac.clone(), CtxPrefixModel16::new(GC5::new()));
    best_of3!(&buf, "[h=ac, m=pm5,  c=gc9]", ac.clone(), CtxPrefixModel5::new(GC9::new()));
    best_of3!(&buf, "[h=ac, m=pm8,  c=gc9]", ac.clone(), CtxPrefixModel8::new(GC9::new()));
    best_of3!(&buf, "[h=ac, m=pm9,  c=gc9]", ac.clone(), CtxPrefixModel9::new(GC9::new()));
    best_of3!(&buf, "[h=ac, m=pm13, c=gc9]", ac.clone(), CtxPrefixModel13::new(GC9::new()));
    best_of3!(&buf, "[h=ac, m=pm16, c=gc9]", ac.clone(), CtxPrefixModel16::new(GC9::new()));
    best_of3!(&buf, "[h=ac, m=pm5,  c=fsm]", ac.clone(), CtxPrefixModel5::new(FSM0::new()));
    best_of3!(&buf, "[h=ac, m=pm8,  c=fsm]", ac.clone(), CtxPrefixModel8::new(FSM0::new()));
    best_of3!(&buf, "[h=ac, m=pm9,  c=fsm]", ac.clone(), CtxPrefixModel9::new(FSM0::new()));
    best_of3!(&buf, "[h=ac, m=pm13, c=fsm]", ac.clone(), CtxPrefixModel13::new(FSM0::new()));
    best_of3!(&buf, "[h=ac, m=pm16, c=fsm]", ac.clone(), CtxPrefixModel16::new(FSM0::new()));

    Ok(())
}

fn compress(buf: &[u8], mut model: impl Model) -> Result<(u64, Duration)> {
    let mut ac = ArithmeticCoder::new_coder();
    let mut writer = ACStats::new();

    let timer = Instant::now();
    for byte in buf {
        unroll_for!(bit in byte, {
            let p = model.predict();
            model.update(bit);
            ac.encode(bit, p, &mut writer)?;
        });
    }
    ac.flush(&mut writer)?;
    let time = timer.elapsed();

    Ok((writer.result(), time))
}

fn compress_ctx(
    buf: &[u8],
    mut history: impl History,
    mut model: impl CtxModel,
) -> Result<(u64, Duration)> {
    let mut ac = ArithmeticCoder::new_coder();
    let mut writer = ACStats::new();

    let timer = Instant::now();
    for byte in buf {
        unroll_for!(bit in byte, {
            let p = model.predict();

            model.adapt(bit);
            history.update(bit);
            model.set_ctx(history.hash());

            ac.encode(bit, p, &mut writer)?;
        });
    }
    ac.flush(&mut writer)?;
    let time = timer.elapsed();

    Ok((writer.result(), time))
}
