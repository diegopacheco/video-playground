use crate::blocking::{BlockRequest, BlockStatus};
use crate::error::{Error, Result};
use axum::body::Bytes;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct Part {
    pub duration: f64,
    pub independent: bool,
    pub data: Bytes,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub msn: u64,
    pub program_date_ms: i64,
    pub parts: Vec<Part>,
    pub complete: bool,
}

impl Segment {
    pub fn duration(&self) -> f64 {
        self.parts.iter().map(|p| p.duration).sum()
    }

    pub fn data(&self) -> Bytes {
        Bytes::from(self.parts.iter().flat_map(|p| p.data.iter().copied()).collect::<Vec<u8>>())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WindowConfig {
    pub max_segments: usize,
    pub part_target: f64,
    pub segment_target: f64,
}

#[derive(Debug)]
pub struct Window {
    pub config: WindowConfig,
    pub init: Option<Bytes>,
    pub segments: VecDeque<Segment>,
    next_msn: u64,
}

impl Window {
    pub fn new(config: WindowConfig) -> Self {
        Self { config, init: None, segments: VecDeque::new(), next_msn: 0 }
    }

    pub fn push_part(&mut self, part: Part, program_date_ms: i64) {
        let open = self.segments.back_mut().filter(|s| !s.complete);
        match (open, part.independent) {
            (Some(segment), false) => segment.parts.push(part),
            (open, true) => {
                if let Some(segment) = open {
                    segment.complete = true;
                }
                self.segments.push_back(Segment {
                    msn: self.next_msn,
                    program_date_ms,
                    parts: vec![part],
                    complete: false,
                });
                self.next_msn += 1;
            }
            (None, false) => {}
        }
        while self.segments.len() > self.config.max_segments {
            self.segments.pop_front();
        }
    }

    pub fn segment(&self, msn: u64) -> Option<&Segment> {
        self.segments.iter().find(|s| s.msn == msn)
    }

    pub fn part(&self, msn: u64, index: usize) -> Option<&Part> {
        self.segment(msn).and_then(|s| s.parts.get(index))
    }

    pub fn complete_segments(&self) -> impl Iterator<Item = &Segment> {
        self.segments.iter().filter(|s| s.complete)
    }

    pub fn check(&self, req: &BlockRequest) -> BlockStatus {
        let Some(last) = self.segments.back() else {
            return BlockStatus::Pending;
        };
        if req.msn > last.msn + 2 {
            return BlockStatus::TooFar;
        }
        if last.msn > req.msn {
            return BlockStatus::Ready;
        }
        let ready = self.segment(req.msn).is_some_and(|s| match req.part {
            None => s.complete,
            Some(p) => s.parts.len() > p,
        });
        if ready { BlockStatus::Ready } else { BlockStatus::Pending }
    }
}

pub struct LiveStream {
    window: Mutex<Window>,
    version: watch::Sender<u64>,
}

impl LiveStream {
    pub fn new(config: WindowConfig) -> Self {
        Self { window: Mutex::new(Window::new(config)), version: watch::Sender::new(0) }
    }

    pub fn update(&self, change: impl FnOnce(&mut Window)) {
        change(&mut self.window.lock().unwrap_or_else(|e| e.into_inner()));
        self.version.send_modify(|v| *v += 1);
    }

    pub fn read<T>(&self, view: impl FnOnce(&Window) -> T) -> T {
        view(&self.window.lock().unwrap_or_else(|e| e.into_inner()))
    }

    pub async fn wait_for(&self, req: BlockRequest, timeout: Duration) -> Result<()> {
        let mut changes = self.version.subscribe();
        let deadline = Instant::now() + timeout;
        loop {
            match self.read(|w| w.check(&req)) {
                BlockStatus::Ready => return Ok(()),
                BlockStatus::TooFar => {
                    return Err(Error::BadRequest(format!("_HLS_msn {} is too far ahead", req.msn)));
                }
                BlockStatus::Pending => {}
            }
            match tokio::time::timeout_at(deadline, changes.changed()).await {
                Err(_) => return Err(Error::Timeout),
                Ok(Err(_)) => return Err(Error::Source("live stream closed".into())),
                Ok(Ok(())) => {}
            }
        }
    }
}

#[cfg(test)]
pub mod fixture {
    use super::*;

    pub fn config() -> WindowConfig {
        WindowConfig { max_segments: 6, part_target: 0.2, segment_target: 2.0 }
    }

    pub fn part(independent: bool) -> Part {
        Part { duration: 0.2, independent, data: Bytes::from_static(b"p") }
    }

    pub fn feed(window: &mut Window, parts: usize) {
        for i in 0..parts {
            window.push_part(part(i % 10 == 0), 1_000_000 + (i as i64) * 200);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixture::*;
    use super::*;
    use std::sync::Arc;

    #[test]
    fn segments_open_only_on_independent_parts_so_each_one_is_decodable_alone() {
        let mut w = Window::new(config());
        w.push_part(part(false), 0);
        assert!(w.segments.is_empty());
        feed(&mut w, 25);
        assert_eq!(w.segments.len(), 3);
        assert!(w.segments.iter().all(|s| s.parts[0].independent));
        assert_eq!(w.segments[0].parts.len(), 10);
        assert!(w.segments[0].complete && w.segments[1].complete && !w.segments[2].complete);
    }

    #[test]
    fn segment_program_date_comes_from_its_first_part() {
        let mut w = Window::new(config());
        feed(&mut w, 15);
        assert_eq!(w.segments[1].program_date_ms, 1_000_000 + 10 * 200);
    }

    #[test]
    fn sliding_window_bounds_memory_for_an_endless_live_stream() {
        let mut w = Window::new(config());
        feed(&mut w, 200);
        assert_eq!(w.segments.len(), 6);
        assert_eq!(w.segments.back().unwrap().msn, 19);
    }

    #[test]
    fn block_check_follows_msn_and_part_semantics() {
        let mut w = Window::new(config());
        feed(&mut w, 13);
        let at = |msn, part| w.check(&BlockRequest { msn, part });
        assert_eq!(at(1, Some(2)), BlockStatus::Ready);
        assert_eq!(at(1, Some(3)), BlockStatus::Pending);
        assert_eq!(at(0, None), BlockStatus::Ready);
        assert_eq!(at(1, None), BlockStatus::Pending);
        assert_eq!(at(2, Some(0)), BlockStatus::Pending);
        assert_eq!(at(4, Some(0)), BlockStatus::TooFar);
    }

    #[tokio::test]
    async fn blocking_reload_returns_as_soon_as_the_requested_part_lands() {
        let live = Arc::new(LiveStream::new(config()));
        live.update(|w| feed(w, 3));
        let waiter = {
            let live = live.clone();
            tokio::spawn(async move {
                live.wait_for(BlockRequest { msn: 0, part: Some(4) }, Duration::from_secs(5)).await
            })
        };
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(!waiter.is_finished());
        live.update(|w| w.push_part(part(false), 0));
        tokio::time::sleep(Duration::from_millis(5)).await;
        assert!(!waiter.is_finished());
        live.update(|w| w.push_part(part(false), 0));
        assert!(waiter.await.unwrap().is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn blocking_reload_gives_up_after_the_timeout_instead_of_hanging_clients() {
        let live = LiveStream::new(config());
        live.update(|w| feed(w, 3));
        let res = live.wait_for(BlockRequest { msn: 0, part: Some(9) }, Duration::from_secs(6)).await;
        assert!(matches!(res, Err(Error::Timeout)));
    }
}
