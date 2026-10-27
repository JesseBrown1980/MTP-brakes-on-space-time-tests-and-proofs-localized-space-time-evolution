//! The discrete-event run, a line-for-line port of timewarp.simulate (integer ticks).

use crate::model::{tail_of, Agent, Ctx};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Optimistic,
    OptimisticV,
    Conservative,
    Oracle,
}

impl Mode {
    pub fn parse(s: &str) -> Mode {
        match s {
            "optimistic" => Mode::Optimistic,
            "optimistic_v" => Mode::OptimisticV,
            "conservative" => Mode::Conservative,
            "oracle" => Mode::Oracle,
            _ => panic!("mode {s}"),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Mode::Optimistic => "optimistic",
            Mode::OptimisticV => "optimistic_v",
            Mode::Conservative => "conservative",
            Mode::Oracle => "oracle",
        }
    }
}

#[derive(Default)]
pub struct Stats {
    pub computations: u64,
    pub rollbacks: u64,
    pub mispredicts: u64,
    pub messages: u64,
    pub cascades: u64,
    pub absorbed: u64,
    pub in_time: u64,
    pub merged: u64,
    pub end: u64,
    pub makespan: u64,
    pub reach: Vec<u64>,
}

enum Ev {
    Done(usize),
    Msg(usize, Ctx, Option<u64>),
}

pub struct World<'a> {
    pub agents: &'a mut [Agent],
    pub agent_of: Vec<usize>,
    pub speed: Vec<u64>,
    pub seg: usize,
    pub m: usize,
}

pub fn sequential(w: &mut World, ctx0: &Ctx) -> Vec<Rc<[u8]>> {
    let d = w.agent_of.len();
    let mut ctx = *ctx0;
    let mut outs = Vec::with_capacity(w.m);
    for s in 0..w.m {
        let a = w.agent_of[s % d];
        let seg = w.agents[a].segment(&ctx, w.seg);
        ctx = tail_of(&ctx, &seg);
        outs.push(seg);
    }
    outs
}

pub fn simulate(w: &mut World, l: u64, ctx0: &Ctx, mode: Mode, truth: &[Rc<[u8]>]) -> (Vec<Rc<[u8]>>, Stats) {
    let d = w.agent_of.len();
    let m = w.m;
    let mut known: Vec<BTreeMap<usize, Ctx>> = vec![BTreeMap::new(); d];
    let mut proc: Vec<Option<(Ctx, Rc<[u8]>)>> = vec![None; m];
    let mut sent: Vec<Option<Ctx>> = vec![None; m];
    let mut depth_of: HashMap<usize, u64> = HashMap::new();
    let mut busy: Vec<Option<(usize, Ctx)>> = vec![None; d];
    let mut free_at: Vec<u64> = vec![0; d];
    let mut heap: BinaryHeap<Reverse<(u64, u64)>> = BinaryHeap::new();
    let mut payload: HashMap<u64, Ev> = HashMap::new();
    let mut seq: u64 = 0;
    let mut st = Stats::default();

    let true_in = |s: usize| -> Ctx {
        if s == 0 {
            *ctx0
        } else {
            let t = &truth[s - 1];
            let mut c = [0u8; 5];
            c.copy_from_slice(&t[t.len() - 5..]);
            c
        }
    };

    macro_rules! dirty {
        ($i:expr, $s:expr) => {
            match &proc[$s] {
                None => true,
                Some((c, _)) => $s >= 1 && known[$i].get(&($s - 1)).map_or(false, |k| k != c),
            }
        };
    }

    // start(i, t): the drafter's lowest dirty segment, if it has an input
    macro_rules! start {
        ($i:expr, $t:expr) => {{
            let i: usize = $i;
            if busy[i].is_none() {
                let mut s = i;
                while s < m {
                    if dirty!(i, s) {
                        let input: Option<Ctx> = if s == 0 {
                            Some(*ctx0)
                        } else if let Some(k) = known[i].get(&(s - 1)) {
                            Some(*k)
                        } else {
                            match mode {
                                Mode::Conservative => None,
                                Mode::Oracle => Some(true_in(s)),
                                _ => {
                                    let j = known[i].range(..s - 1).next_back().map(|(k, _)| *k);
                                    let (base, steps) = match j {
                                        None => (*ctx0, s),
                                        Some(j) => (known[i][&j], s - 1 - j),
                                    };
                                    let a = w.agent_of[i];
                                    Some(w.agents[a].guess(&base, steps, mode == Mode::OptimisticV, w.seg))
                                }
                            }
                        };
                        if let Some(ctx) = input {
                            busy[i] = Some((s, ctx));
                            st.computations += 1;
                            if proc[s].is_some() {
                                st.rollbacks += 1;
                            }
                            free_at[i] = free_at[i].max($t) + (w.seg as u64) * w.speed[i];
                            seq += 1;
                            heap.push(Reverse((free_at[i], seq)));
                            payload.insert(seq, Ev::Done(i));
                        }
                        break;
                    }
                    s += d;
                }
            }
        }};
    }

    for i in 0..d {
        start!(i, 0u64);
    }
    while let Some(Reverse((t, q))) = heap.pop() {
        st.makespan = st.makespan.max(t);
        match payload.remove(&q).unwrap() {
            Ev::Done(i) => {
                let (s, ctx) = busy[i].take().unwrap();
                let first = proc[s].is_none();
                let a = w.agent_of[i];
                let segv = w.agents[a].segment(&ctx, w.seg);
                if first && ctx != true_in(s) {
                    st.mispredicts += 1;
                }
                let tail = tail_of(&ctx, &segv);
                proc[s] = Some((ctx, segv));
                let stale = s >= 1 && known[i].get(&(s - 1)).map_or(false, |k| *k != ctx);
                let dd = if stale { None } else { depth_of.remove(&s) };
                let changed = sent[s] != Some(tail);
                if let Some(dv) = dd {
                    if !changed {
                        st.absorbed += 1;
                        st.reach.push(dv);
                    } else if s + 1 == m {
                        st.end += 1;
                        st.reach.push(dv);
                    }
                }
                if s + 1 < m && changed {
                    sent[s] = Some(tail);
                    st.messages += 1;
                    seq += 1;
                    heap.push(Reverse((t + l, seq)));
                    payload.insert(seq, Ev::Msg(s, tail, dd));
                }
                start!(i, t);
            }
            Ev::Msg(s, tail, dmsg) => {
                let j = (s + 1) % d;
                known[j].insert(s, tail);
                // hit(s, tail, d)
                let tgt = s + 1;
                let mut used: Option<Ctx> = proc[tgt].as_ref().map(|p| p.0);
                if let Some((bs, bc)) = busy[tgt % d] {
                    if bs == tgt && (proc[tgt].is_none() || Some(bc) != used) {
                        used = Some(bc);
                    }
                }
                let wrong = used.is_some() && used != Some(tail);
                match dmsg {
                    None => {
                        if wrong {
                            if let std::collections::hash_map::Entry::Vacant(e) = depth_of.entry(tgt) {
                                st.cascades += 1;
                                e.insert(1);
                            }
                        }
                    }
                    Some(dv) => {
                        if !wrong {
                            st.in_time += 1;
                            st.reach.push(dv);
                        } else {
                            match depth_of.entry(tgt) {
                                std::collections::hash_map::Entry::Occupied(_) => {
                                    st.merged += 1;
                                    st.reach.push(dv);
                                }
                                std::collections::hash_map::Entry::Vacant(e) => {
                                    e.insert(dv + 1);
                                }
                            }
                        }
                    }
                }
                start!(j, t);
            }
        }
    }
    for i in 0..d {
        let mut s = i;
        while s < m {
            if busy[i].is_some() || dirty!(i, s) {
                panic!("did not settle");
            }
            s += d;
        }
    }
    let final_: Vec<Rc<[u8]>> = proc.into_iter().map(|p| p.unwrap().1).collect();
    (final_, st)
}
