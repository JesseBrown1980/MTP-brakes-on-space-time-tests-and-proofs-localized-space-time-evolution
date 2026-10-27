//! Frozen n-gram models over exact sorted address tables, and the agents built from them.
//! Mirrors spec_brake.Model (greedy only) and timewarp.Agent / greedy_run exactly.

use crate::npy;
use std::collections::HashMap;
use std::rc::Rc;

pub const VER_N: usize = 5;
pub const DRAFT_N: usize = 2;
pub const KSPEC: usize = 4;
pub type Ctx = [u8; VER_N];

pub struct Model {
    n: usize,
    k: Vec<Vec<u64>>,
    c: Vec<Vec<i64>>,
    memo: HashMap<(usize, u64), Option<u8>>,
}

impl Model {
    pub fn load(dir: &str, n: usize) -> Result<Model, String> {
        let mut k = Vec::new();
        let mut c = Vec::new();
        for o in 0..=n {
            k.push(npy::read_u64(&format!("{dir}/K{o}.npy"))?);
            c.push(npy::read_i64(&format!("{dir}/C{o}.npy"))?);
        }
        Ok(Model { n, k, c, memo: HashMap::new() })
    }

    /// Greedy next byte after `hist` (its last 5 bytes): back off order n .. 0 to the first
    /// context seen in training; largest count, ties to the smallest byte.
    pub fn greedy(&mut self, hist: &[u8]) -> u8 {
        for o in (0..=self.n).rev() {
            let ctx: u64 = if o == 0 { 0 } else { hist[hist.len() - o..].iter().fold(0u64, |a, &b| (a << 8) | b as u64) };
            let key = (o, ctx);
            let g = match self.memo.get(&key) {
                Some(g) => *g,
                None => {
                    let lo = ctx << 8;
                    let ks = &self.k[o];
                    let s = ks.partition_point(|&x| x < lo);
                    let e = ks.partition_point(|&x| x < lo + 256);
                    let g = if e > s {
                        let mut best = (i64::MIN, 0u8);
                        for (key, &cnt) in ks[s..e].iter().zip(self.c[o][s..e].iter()) {
                            if cnt > best.0 {
                                best = (cnt, (key & 255) as u8);
                            }
                        }
                        Some(best.1)
                    } else {
                        None
                    };
                    self.memo.insert(key, g);
                    g
                }
            };
            if let Some(b) = g {
                return b;
            }
        }
        panic!("order-0 table empty")
    }
}

fn last5(v: &[u8]) -> Ctx {
    let mut c = [0u8; VER_N];
    c.copy_from_slice(&v[v.len() - VER_N..]);
    c
}

/// n bytes of greedy text from a 5-byte context, exactly; the next byte depends only on the
/// last 5, so the text enters a cycle and is indexed ahead. With tail_only, the last <= 5 bytes.
pub fn greedy_run(m: &mut Model, ctx: &Ctx, n: usize, tail_only: bool) -> Vec<u8> {
    let mut state: Ctx = *ctx;
    let mut seen: HashMap<Ctx, usize> = HashMap::new();
    let mut seq: Vec<u8> = Vec::new();
    while seq.len() < n {
        if let Some(&c) = seen.get(&state) {
            let per = seq.len() - c;
            let at = |j: usize| if j < c { seq[j] } else { seq[c + (j - c) % per] };
            let from = if tail_only { n.saturating_sub(VER_N) } else { 0 };
            return (from..n).map(at).collect();
        }
        seen.insert(state, seq.len());
        let b = m.greedy(&state);
        seq.push(b);
        state.rotate_left(1);
        state[VER_N - 1] = b;
    }
    if tail_only {
        let k = seq.len().saturating_sub(VER_N);
        seq[k..].to_vec()
    } else {
        seq
    }
}

pub struct Agent {
    pub draft: Model,
    pub ver: Model,
    seg_memo: HashMap<Ctx, Rc<[u8]>>,
    roll_memo: HashMap<(Ctx, usize, bool), Ctx>,
}

impl Agent {
    pub fn load(dir: &str) -> Result<Agent, String> {
        Ok(Agent {
            draft: Model::load(dir, DRAFT_N)?,
            ver: Model::load(dir, VER_N)?,
            seg_memo: HashMap::new(),
            roll_memo: HashMap::new(),
        })
    }

    /// The verified segment: the verifier's greedy text (what speculative greedy commits).
    pub fn segment(&mut self, ctx: &Ctx, seg: usize) -> Rc<[u8]> {
        if let Some(s) = self.seg_memo.get(ctx) {
            return s.clone();
        }
        let s: Rc<[u8]> = if seg > 64 { greedy_run(&mut self.ver, ctx, seg, false).into() } else { self.segment_direct(ctx, seg).into() };
        self.seg_memo.insert(*ctx, s.clone());
        s
    }

    /// Speculative greedy, draft depth KSPEC, byte by byte (timewarp.Agent.segment_direct).
    pub fn segment_direct(&mut self, ctx: &Ctx, seg: usize) -> Vec<u8> {
        let mut out: Vec<u8> = ctx.to_vec();
        while out.len() - VER_N < seg {
            let mut h: Vec<u8> = out[out.len() - VER_N..].to_vec();
            let mut xs = [0u8; KSPEC];
            for x in xs.iter_mut() {
                *x = self.draft.greedy(&h);
                h.push(*x);
            }
            let mut h: Vec<u8> = out[out.len() - VER_N..].to_vec();
            let mut all = true;
            for x in xs.iter() {
                let g = self.ver.greedy(&h);
                out.push(g);
                h.push(g);
                if g != *x {
                    all = false;
                    break;
                }
            }
            if all {
                let g = self.ver.greedy(&h);
                out.push(g);
            }
        }
        out[VER_N..VER_N + seg].to_vec()
    }

    /// The drafter's guess: roll its draft (or verifier) forward steps segments; last 5 bytes.
    pub fn guess(&mut self, ctx: &Ctx, steps: usize, use_verifier: bool, seg: usize) -> Ctx {
        let key = (*ctx, steps, use_verifier);
        if let Some(g) = self.roll_memo.get(&key) {
            return *g;
        }
        let m = if use_verifier { &mut self.ver } else { &mut self.draft };
        let mut v = ctx.to_vec();
        v.extend(greedy_run(m, ctx, steps * seg, true));
        let g = last5(&v);
        self.roll_memo.insert(key, g);
        g
    }
}

pub fn tail_of(ctx: &Ctx, seg: &[u8]) -> Ctx {
    if seg.len() >= VER_N {
        last5(seg)
    } else {
        let mut v = ctx.to_vec();
        v.extend_from_slice(seg);
        last5(&v)
    }
}
