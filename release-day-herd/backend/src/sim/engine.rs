use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::HerdError;
use crate::admission::AdmissionGate;
use crate::rng::Rng;
use crate::sim::bucket::TokenBucket;
use crate::sim::report::Recorder;
use crate::sim::{SimConfig, SimReport, Strategy};

const MICROS_PER_MS: u64 = 1_000;
const NOT_CACHED: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Pressed,
    Api,
    Manifest,
    Segment,
}

#[derive(Debug, Clone, Copy)]
struct Client {
    press_us: u64,
    edge: u32,
    rendition: u32,
    stage: Stage,
    failures: u32,
    retrying: bool,
}

struct World<'a> {
    strategy: &'a dyn Strategy,
    cfg: &'a SimConfig,
    rng: Rng,
    clients: Vec<Client>,
    queue: BinaryHeap<Reverse<(u64, u32)>>,
    api: TokenBucket,
    origin: TokenBucket,
    gate: Option<AdmissionGate>,
    filled_at: Vec<u64>,
    recorder: Recorder,
}

pub fn simulate(strategy: &dyn Strategy, cfg: &SimConfig) -> Result<SimReport, HerdError> {
    cfg.validate()?;
    let mut world = World::new(strategy, cfg)?;
    world.run();
    Ok(world.finish())
}

impl<'a> World<'a> {
    fn new(strategy: &'a dyn Strategy, cfg: &'a SimConfig) -> Result<Self, HerdError> {
        let mut rng = Rng::new(cfg.seed);
        let clients: Vec<Client> = (0..cfg.users)
            .map(|_| Client {
                press_us: rng.below(cfg.press_window_ms * MICROS_PER_MS),
                edge: rng.below(u64::from(cfg.edges)) as u32,
                rendition: rng.below(u64::from(cfg.renditions)) as u32,
                stage: Stage::Pressed,
                failures: 0,
                retrying: false,
            })
            .collect();
        let queue = clients.iter().enumerate().map(|(i, c)| Reverse((c.press_us, i as u32))).collect();
        let objects = (cfg.edges * cfg.objects_per_edge()) as usize;
        let warm = if strategy.prewarm_cdn() { 0 } else { NOT_CACHED };
        let gate = match strategy.uses_waiting_room() {
            true => Some(AdmissionGate::new(cfg.admission_rps)?),
            false => None,
        };
        Ok(Self {
            strategy,
            cfg,
            rng,
            clients,
            queue,
            api: TokenBucket::for_rate(cfg.api_rps),
            origin: TokenBucket::for_rate(cfg.origin_rps),
            gate,
            filled_at: vec![warm; objects],
            recorder: Recorder::default(),
        })
    }

    fn run(&mut self) {
        while let Some(Reverse((t, id))) = self.queue.pop() {
            self.step(id, t);
        }
    }

    fn finish(self) -> SimReport {
        let prewarmed = if self.strategy.prewarm_cdn() { self.filled_at.len() as u64 } else { 0 };
        self.recorder.finish(self.strategy.name(), u64::from(self.cfg.users), prewarmed)
    }

    fn latency(&mut self, base_ms: u64) -> u64 {
        (base_ms + self.rng.below(self.cfg.jitter_ms + 1)) * MICROS_PER_MS
    }

    fn step(&mut self, id: u32, t: u64) {
        match self.clients[id as usize].stage {
            Stage::Pressed => self.enter(id, t),
            Stage::Api => self.call_api(id, t),
            Stage::Manifest | Stage::Segment => self.fetch_from_cdn(id, t),
        }
    }

    fn enter(&mut self, id: u32, t: u64) {
        self.clients[id as usize].stage = Stage::Api;
        match self.gate.as_mut() {
            Some(gate) => {
                let ticket = gate.join(t);
                self.recorder.admitted(ticket.admit_at_us);
                self.queue.push(Reverse((ticket.admit_at_us, id)));
            }
            None => self.call_api(id, t),
        }
    }

    fn call_api(&mut self, id: u32, t: u64) {
        let accepted = self.api.try_take(t);
        self.recorder.api(t, accepted);
        if accepted {
            let done = t + self.latency(self.cfg.api_latency_ms);
            self.succeed(id, done);
        } else {
            let done = t + self.latency(self.cfg.api_reject_ms);
            self.fail(id, done);
        }
    }

    fn fetch_from_cdn(&mut self, id: u32, t: u64) {
        let c = self.clients[id as usize];
        let object = match c.stage {
            Stage::Manifest => 0,
            _ => 1 + c.rendition,
        };
        let slot = (c.edge * self.cfg.objects_per_edge() + object) as usize;
        let edge_us = self.latency(self.cfg.edge_latency_ms);
        if self.filled_at[slot] <= t {
            self.succeed(id, t + edge_us);
            return;
        }
        let accepted = self.origin.try_take(t);
        self.recorder.origin(t, accepted);
        if accepted {
            let done = t + edge_us + self.latency(self.cfg.origin_latency_ms);
            self.filled_at[slot] = self.filled_at[slot].min(done);
            self.succeed(id, done);
        } else {
            let done = t + edge_us + self.latency(self.cfg.origin_reject_ms);
            self.fail(id, done);
        }
    }

    fn succeed(&mut self, id: u32, done: u64) {
        let c = &mut self.clients[id as usize];
        c.retrying = false;
        let next = match c.stage {
            Stage::Api => Stage::Manifest,
            Stage::Manifest => Stage::Segment,
            _ => {
                let ttff = done - c.press_us;
                self.recorder.first_frame(done, ttff);
                return;
            }
        };
        c.stage = next;
        self.queue.push(Reverse((done, id)));
    }

    fn fail(&mut self, id: u32, done: u64) {
        let (was_retry, failures, press_us) = {
            let c = &mut self.clients[id as usize];
            let was_retry = c.retrying;
            c.failures += 1;
            (was_retry, c.failures, c.press_us)
        };
        if was_retry {
            self.recorder.wasted_retry();
        }
        if failures >= self.cfg.max_attempts {
            self.recorder.abandon();
            return;
        }
        let at = done + self.strategy.retry_delay_ms(failures, &mut self.rng) * MICROS_PER_MS;
        if at - press_us > self.cfg.give_up_ms * MICROS_PER_MS {
            self.recorder.abandon();
            return;
        }
        self.clients[id as usize].retrying = true;
        self.recorder.retry(at);
        self.queue.push(Reverse((at, id)));
    }
}
