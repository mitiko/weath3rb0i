use std::{fs, io};
use weath3rb0i::{
    history::{ACHistory, History, HistoryBitOrder},
    models::{counters::*, AdaptiveModel, CtxModel, CtxPrefixModel8, FreezeModel, PrefixModel8},
    unroll_for,
};

fn main() -> io::Result<()> {
    let buf = fs::read("/Users/mitiko/_data/calgary/book1")?;

    let mut ctx_stats_raw = [0; 256];
    let mut ctx_stats_ac = [0; 256];

    let mut model = PrefixModel8::new(Counter4::new());
    for byte in buf.iter() {
        unroll_for!(bit in byte, {
            ctx_stats_raw[model.ctx] += 1;
            model.update(bit);
        });
    }

    let mut inner = PrefixModel8::new(Counter4::new());
    inner.train(&buf);
    let inner = inner.freeze();

    let mut history = ACHistory::new(inner, HistoryBitOrder::LSB);
    let mut model = CtxPrefixModel8::new(Counter4::new());
    for byte in buf.iter() {
        unroll_for!(bit in byte, {
            history.update(bit);
            model.set_ctx(history.hash(7));
            ctx_stats_ac[model.ctx] += 1;
        });
    }

    let used_raw = ctx_stats_raw.iter().filter(|&&x| x > 0).count();
    let used_ac = ctx_stats_ac.iter().filter(|&&x| x > 0).count();
    println!("raw: {used_raw} / 256");
    println!("ac: {used_ac} / 256");

    ctx_stats_raw.sort();
    ctx_stats_ac.sort();
    ctx_stats_raw.reverse();
    ctx_stats_ac.reverse();

    let top = nice_top(ctx_stats_raw[0].max(ctx_stats_ac[0]));
    let lin = Scale { top, log: false };
    let log = Scale { top: 10u64.pow(DECADES), log: true };

    let html = TEMPLATE
        .replace("{{RAW_LIN}}", &lin.area(&ctx_stats_raw))
        .replace("{{AC_LIN}}", &lin.area(&ctx_stats_ac))
        .replace("{{GRID_LIN}}", &lin.grid())
        .replace("{{RAW_LOG}}", &log.area(&ctx_stats_raw))
        .replace("{{AC_LOG}}", &log.area(&ctx_stats_ac))
        .replace("{{GRID_LOG}}", &log.grid())
        .replace("{{USED_RAW}}", &used_raw.to_string())
        .replace("{{USED_AC}}", &used_ac.to_string());
    fs::write("artifacts/eh-ctx-distribution.html", html)?;

    Ok(())
}

/* AI slop below */

const TEMPLATE: &str = r#"<div class="ctxd">
  <style>
    .ctxd { --raw: #2a78d6; --ac: #eb6834; --ink: #575652; --line: #dededa;
            font: 13px/1.5 system-ui, sans-serif; color: var(--ink); max-width: 680px; }
    @media (prefers-color-scheme: dark) {
      .ctxd { --raw: #3987e5; --ac: #d95926; --ink: #a8a79f; --line: #2c2c2a; }
    }
    .ctxd svg { display: block; width: 100%; height: auto; }
    .ctxd .lbl { margin: 10px 0 0; font-size: 11px; letter-spacing: .06em; text-transform: uppercase; }
    .ctxd .grid line { stroke: var(--line); }
    .ctxd .grid text, .ctxd .xax text { fill: var(--ink); font-size: 10px; text-anchor: end; }
    .ctxd .xax text { text-anchor: middle; }
    .ctxd .raw { fill: var(--raw); fill-opacity: .22; stroke: var(--raw); }
    .ctxd .ac  { fill: var(--ac);  fill-opacity: .22; stroke: var(--ac); }
    .ctxd p { margin: 8px 0 0; }
    .ctxd .k { display: inline-block; width: 9px; height: 9px; border-radius: 50%; }
  </style>

  <p class="lbl">linear</p>
  <svg viewBox="0 0 640 170" role="img"
       aria-label="L1 history length per context, sorted, linear scale">
    <g class="grid">{{GRID_LIN}}</g>
    <path class="ac" d="{{AC_LIN}}"/>
    <path class="raw" d="{{RAW_LIN}}"/>
    <g class="xax">
      <text x="38" y="167">0</text><text x="187" y="167">64</text>
      <text x="335" y="167">128</text><text x="484" y="167">192</text>
      <text x="632" y="167">256</text>
    </g>
  </svg>

  <p class="lbl">log</p>
  <svg viewBox="0 0 640 170" role="img"
       aria-label="L1 history length per context, sorted, log scale">
    <g class="grid">{{GRID_LOG}}</g>
    <path class="ac" d="{{AC_LOG}}"/>
    <path class="raw" d="{{RAW_LOG}}"/>
    <g class="xax">
      <text x="38" y="167">0</text><text x="187" y="167">64</text>
      <text x="335" y="167">128</text><text x="484" y="167">192</text>
      <text x="632" y="167">256</text>
    </g>
  </svg>

  <p><span class="k" style="background:var(--raw)"></span> raw history &nbsp;
     <span class="k" style="background:var(--ac)"></span> ac history &mdash;
     contexts sorted by the length of the L1 history they see, book1.</p>
  <p>Raw history lands on <b>{{USED_RAW}}</b> of 256 contexts, ac history on <b>{{USED_AC}}</b>.</p>
</div>
"#;

// chart box — 256 ranks on x, L1 history length on y
const W: f64 = 640.0;
const H: f64 = 170.0;
const PL: f64 = 38.0;
const PB: f64 = 18.0;
const PW: f64 = W - PL - 8.0;
const PH: f64 = H - PB - 8.0;
const DECADES: u32 = 6; // 1 .. 1M

struct Scale {
    top: u64,
    log: bool,
}

impl Scale {
    fn y(&self, v: u64) -> f64 {
        let frac = match self.log {
            true => (v.max(1) as f64).log10() / f64::from(DECADES),
            false => v as f64 / self.top as f64,
        };
        H - PB - PH * frac
    }

    /// filled step area, one column per rank
    fn area(&self, stats: &[u64; 256]) -> String {
        let pitch = PW / 256.0;
        let mut d = format!("M{PL} {}", H - PB);
        for (i, &v) in stats.iter().enumerate() {
            let x = PL + i as f64 * pitch;
            d += &format!("L{x:.1} {0:.1}L{1:.1} {0:.1}", self.y(v), x + pitch);
        }
        d + &format!("L{} {}Z", PL + PW, H - PB)
    }

    fn grid(&self) -> String {
        let ticks: Vec<u64> = match self.log {
            true => (0..=DECADES).map(|e| 10u64.pow(e)).collect(),
            false => (0..=4).map(|i| i * self.top / 4).collect(),
        };
        ticks
            .iter()
            .map(|&v| {
                let y = self.y(v);
                format!(
                    "<line x1='{PL}' x2='{}' y1='{y:.1}' y2='{y:.1}'/>\
                     <text x='{}' y='{:.1}'>{}</text>",
                    PL + PW,
                    PL - 6.0,
                    y + 3.5,
                    fmt(v)
                )
            })
            .collect()
    }
}

fn fmt(v: u64) -> String {
    match v {
        1_000_000.. => format!("{}M", v / 1_000_000),
        1_000.. => format!("{}k", v / 1_000),
        _ => v.to_string(),
    }
}

/// round up to a multiple of 4 nice steps, so the linear axis gets round labels
fn nice_top(max: u64) -> u64 {
    let mag = 10f64.powf((max as f64 / 4.0).log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0]
        .iter()
        .map(|m| m * mag)
        .find(|s| s * 4.0 >= max as f64)
        .unwrap() as u64;
    (max + step - 1) / step * step
}
