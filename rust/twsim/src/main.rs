//! twsim: the drafter timewarp simulation compiled, a port of timewarp.py and
//! scale_drafters.py that must reproduce their receipts byte for byte. Integer-only, json=0.
//!
//! usage:
//!   twsim scale ENWIK8 TW_CACHE OUT D L MODE     one scale_drafters.py job (SC_LATS=L SC_MODES=MODE)
//!   twsim tw    ENWIK8 TW_CACHE OUT [SEG]        timewarp.py run (D = 3, 9, 27; M = 81)
#![deny(clippy::float_arithmetic)]

mod model;
mod npy;
mod sha256;
mod sim;

use model::{Agent, Ctx, VER_N};
use sim::{sequential, simulate, Mode, Stats, World};
use std::collections::BTreeMap;
use std::fs;

const ENWIK8_SHA256: &str = "2b49720ec4d78c3c9fabaee6e4179a5e997302b3a70029f30f2d582218c024a8";
const HELD_LO: usize = 95_000_000;
const HELD_HI: usize = 99_998_976;
const ANCHOR_MOD: u64 = 65536;
const POOL: usize = 27;
const BASE_SEGMENTS: usize = 83 * 81;
const MODES: [&str; 4] = ["optimistic", "optimistic_v", "conservative", "oracle"];
const LATS: [u64; 2] = [8, 64];

fn corpus(path: &str) -> Vec<u8> {
    let data = fs::read(path).expect("corpus");
    if sha256::hex(&sha256::digest(&data)) != ENWIK8_SHA256 {
        eprintln!("CORPUS|sha256_mismatch|json=0");
        std::process::exit(2);
    }
    data
}

fn anchors() -> Vec<usize> {
    (HELD_LO..HELD_HI)
        .filter(|p| {
            let h = sha256::digest(format!("TW-V1|{p}").as_bytes());
            u64::from_be_bytes(h[..8].try_into().unwrap()) % ANCHOR_MOD == 0
        })
        .collect()
}

fn speed(i: usize) -> u64 {
    1 + (sha256::digest(format!("TW-SPEED|{i}").as_bytes())[0] % 4) as u64
}

fn ctx_at(data: &[u8], p: usize) -> Ctx {
    let mut c = [0u8; VER_N];
    c.copy_from_slice(&data[p - VER_N..p]);
    c
}

struct Tot {
    st: Stats,
    exact_fail: u64,
    done: u64,
    hist: BTreeMap<u64, u64>,
}

fn run_mode(w: &mut World, data: &[u8], an: &[usize], l: u64, mode: Mode) -> Tot {
    let mut t = Tot { st: Stats::default(), exact_fail: 0, done: 0, hist: BTreeMap::new() };
    for &p in an {
        let ctx0 = ctx_at(data, p);
        let truth = sequential(w, &ctx0);
        let (fin, st) = simulate(w, l, &ctx0, mode, &truth);
        if fin.iter().zip(truth.iter()).any(|(a, b)| a[..] != b[..]) || fin.len() != truth.len() {
            t.exact_fail += 1;
        }
        t.st.computations += st.computations;
        t.st.rollbacks += st.rollbacks;
        t.st.mispredicts += st.mispredicts;
        t.st.messages += st.messages;
        t.st.cascades += st.cascades;
        t.st.end += st.end;
        t.st.makespan += st.makespan;
        t.st.absorbed += st.absorbed;
        t.st.in_time += st.in_time;
        t.st.merged += st.merged;
        for r in st.reach {
            *t.hist.entry(r).or_insert(0) += 1;
        }
        t.done += 1;
    }
    t
}

fn hist_str(h: &BTreeMap<u64, u64>) -> String {
    if h.is_empty() {
        "-".to_string()
    } else {
        h.iter().map(|(k, v)| format!("{k}:{v}")).collect::<Vec<_>>().join(",")
    }
}

fn cons_bound(d: usize, m: usize, seg: usize, l: u64) -> u64 {
    (0..m).map(|s| (seg as u64) * speed(s % d)).sum::<u64>() + (m as u64 - 1) * l
}

fn body(t: &Tot, m: usize, n_an: usize, cb: u64) -> String {
    format!(
        "exact_fail={}|computations={}|segments={}|rollbacks={}|mispredicts={}|messages={}|cascades={}|absorbed={}|in_time={}|merged={}|unabsorbed={}|makespan={}|conservative_bound={}|reach_hist={}|json=0",
        t.exact_fail, t.st.computations, m * n_an, t.st.rollbacks, t.st.mispredicts, t.st.messages, t.st.cascades,
        t.st.absorbed, t.st.in_time, t.st.merged, t.st.end, t.st.makespan, cb * n_an as u64, hist_str(&t.hist)
    )
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("scale") => {
            let (enwik, cache, out) = (&a[2], &a[3], &a[4]);
            let d: usize = a[5].parse().unwrap();
            let l: u64 = a[6].parse().unwrap();
            let mode = Mode::parse(&a[7]);
            let seg: usize = std::env::var("TW_SEG").ok().map_or(16, |s| s.parse().unwrap());
            let data = corpus(enwik);
            let mut pool: Vec<Agent> = (0..POOL).map(|j| Agent::load(&format!("{cache}/D27/s{j}")).unwrap()).collect();
            let base = anchors();
            let m = 3 * d;
            let n = BASE_SEGMENTS.checked_div(m).unwrap_or(0).clamp(1, base.len());
            let an = &base[..n];
            let mut w = World { agents: &mut pool, agent_of: (0..d).map(|i| i % POOL).collect(), speed: (0..d).map(speed).collect(), seg, m };
            let t = run_mode(&mut w, &data, an, l, mode);
            let text = format!(
                "SCHDR|base_anchors={}|pool={POOL}|SEG={seg}|json=0\nSC|D={d}|M={m}|L={l}|mode={}|anchors={}|{}\n",
                base.len(), mode.name(), t.done, body(&t, m, n, cons_bound(d, m, seg, l))
            );
            fs::write(out, text).unwrap();
        }
        Some("tw") => {
            let (enwik, cache, out) = (&a[2], &a[3], &a[4]);
            let seg: usize = a.get(5).map_or(16, |s| s.parse().unwrap());
            let data = corpus(enwik);
            let an = anchors();
            let m = 81;
            let mut lines = vec![format!("TWHDR|anchors={}|M={m}|SEG={seg}|KSPEC=4|D=3,9,27|L=8,64|json=0", an.len())];
            let mut c5 = 0u64;
            for d in [3usize, 9, 27] {
                let mut agents: Vec<Agent> = (0..d).map(|i| Agent::load(&format!("{cache}/D{d}/s{i}")).unwrap()).collect();
                if seg > 64 {
                    for &p in &an {
                        let c = ctx_at(&data, p);
                        if agents[0].segment(&c, seg)[..] != agents[0].segment_direct(&c, seg)[..] {
                            c5 += 1;
                        }
                    }
                    lines.push(format!("TWC5|D={d}|checked={}|fast_path_mismatch={c5}|json=0", an.len()));
                }
                let mut w = World { agents: &mut agents, agent_of: (0..d).collect(), speed: (0..d).map(speed).collect(), seg, m };
                for l in LATS {
                    for mname in MODES {
                        let t = run_mode(&mut w, &data, &an, l, Mode::parse(mname));
                        lines.push(format!("TW|D={d}|L={l}|mode={mname}|anchors_done={}|{}", t.done, body(&t, m, an.len(), cons_bound(d, m, seg, l))));
                    }
                }
            }
            fs::write(out, lines.join("\n") + "\n").unwrap();
        }
        _ => {
            eprintln!("usage: twsim scale ENWIK8 TW_CACHE OUT D L MODE | twsim tw ENWIK8 TW_CACHE OUT [SEG]");
            std::process::exit(2);
        }
    }
}
