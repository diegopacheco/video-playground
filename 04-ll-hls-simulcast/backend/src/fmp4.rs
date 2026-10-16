use crate::error::{Error, Result};

const NON_SYNC_SAMPLE: u32 = 0x0001_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackInfo {
    pub timescale: u32,
    pub default_duration: u32,
    pub default_flags: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FragmentInfo {
    pub decode_time: u64,
    pub duration: u64,
    pub independent: bool,
}

pub struct Mp4Box<'a> {
    pub kind: [u8; 4],
    pub body: &'a [u8],
}

fn u32_at(data: &[u8], at: usize) -> Result<u32> {
    data.get(at..at + 4)
        .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| Error::Mp4(format!("truncated u32 at {at}")))
}

fn u64_at(data: &[u8], at: usize) -> Result<u64> {
    Ok(((u32_at(data, at)? as u64) << 32) | u32_at(data, at + 4)? as u64)
}

pub fn box_size(header: &[u8]) -> Result<Option<usize>> {
    if header.len() < 8 {
        return Ok(None);
    }
    let size = u32_at(header, 0)? as usize;
    if size == 1 {
        if header.len() < 16 {
            return Ok(None);
        }
        return Ok(Some(u64_at(header, 8)? as usize));
    }
    if size < 8 {
        return Err(Error::Mp4(format!("unsupported box size {size}")));
    }
    Ok(Some(size))
}

pub fn children(data: &[u8]) -> Result<Vec<Mp4Box<'_>>> {
    let mut boxes = Vec::new();
    let mut at = 0;
    while at < data.len() {
        let size = box_size(&data[at..])?
            .ok_or_else(|| Error::Mp4("truncated box header".into()))?;
        let header = if u32_at(data, at)? == 1 { 16 } else { 8 };
        let end = at + size;
        if end > data.len() {
            return Err(Error::Mp4("box exceeds parent".into()));
        }
        let kind = [data[at + 4], data[at + 5], data[at + 6], data[at + 7]];
        boxes.push(Mp4Box { kind, body: &data[at + header..end] });
        at = end;
    }
    Ok(boxes)
}

fn find<'a>(data: &'a [u8], kind: &[u8; 4]) -> Result<&'a [u8]> {
    children(data)?
        .into_iter()
        .find(|b| &b.kind == kind)
        .map(|b| b.body)
        .ok_or_else(|| Error::Mp4(format!("missing {}", String::from_utf8_lossy(kind))))
}

fn find_path<'a>(data: &'a [u8], path: &[&[u8; 4]]) -> Result<&'a [u8]> {
    path.iter().try_fold(data, |cur, kind| find(cur, kind))
}

pub fn parse_init(moov: &[u8]) -> Result<TrackInfo> {
    let mdhd = find_path(moov, &[b"trak", b"mdia", b"mdhd"])?;
    let timescale = if mdhd.first() == Some(&1) { u32_at(mdhd, 20)? } else { u32_at(mdhd, 12)? };
    let trex = find_path(moov, &[b"mvex", b"trex"])?;
    Ok(TrackInfo {
        timescale,
        default_duration: u32_at(trex, 12)?,
        default_flags: u32_at(trex, 20)?,
    })
}

pub fn parse_fragment(moof: &[u8], track: &TrackInfo) -> Result<FragmentInfo> {
    let traf = find(moof, b"traf")?;
    let (default_duration, default_flags) = parse_tfhd(find(traf, b"tfhd")?, track)?;
    let tfdt = find(traf, b"tfdt")?;
    let decode_time = if tfdt.first() == Some(&1) { u64_at(tfdt, 4)? } else { u32_at(tfdt, 4)? as u64 };
    let trun = find(traf, b"trun")?;
    let flags = u32_at(trun, 0)? & 0x00ff_ffff;
    let count = u32_at(trun, 4)? as usize;
    let mut at = 8;
    if flags & 0x1 != 0 {
        at += 4;
    }
    let mut first_flags = None;
    if flags & 0x4 != 0 {
        first_flags = Some(u32_at(trun, at)?);
        at += 4;
    }
    let mut duration = 0u64;
    for i in 0..count {
        if flags & 0x100 != 0 {
            duration += u32_at(trun, at)? as u64;
            at += 4;
        } else {
            duration += default_duration as u64;
        }
        if flags & 0x200 != 0 {
            at += 4;
        }
        if flags & 0x400 != 0 {
            if i == 0 && first_flags.is_none() {
                first_flags = Some(u32_at(trun, at)?);
            }
            at += 4;
        }
        if flags & 0x800 != 0 {
            at += 4;
        }
    }
    let sample_flags = first_flags.unwrap_or(default_flags);
    Ok(FragmentInfo {
        decode_time,
        duration,
        independent: count > 0 && sample_flags & NON_SYNC_SAMPLE == 0,
    })
}

fn parse_tfhd(tfhd: &[u8], track: &TrackInfo) -> Result<(u32, u32)> {
    let flags = u32_at(tfhd, 0)? & 0x00ff_ffff;
    let mut at = 8;
    if flags & 0x1 != 0 {
        at += 8;
    }
    if flags & 0x2 != 0 {
        at += 4;
    }
    let mut duration = track.default_duration;
    if flags & 0x8 != 0 {
        duration = u32_at(tfhd, at)?;
        at += 4;
    }
    if flags & 0x10 != 0 {
        at += 4;
    }
    let mut sample_flags = track.default_flags;
    if flags & 0x20 != 0 {
        sample_flags = u32_at(tfhd, at)?;
    }
    Ok((duration, sample_flags))
}

pub struct BoxSplitter {
    buffer: Vec<u8>,
}

impl Default for BoxSplitter {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxSplitter {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn push(&mut self, chunk: &[u8]) -> Result<Vec<([u8; 4], Vec<u8>)>> {
        self.buffer.extend_from_slice(chunk);
        let mut out = Vec::new();
        while let Some(size) = box_size(&self.buffer)? {
            if self.buffer.len() < size {
                break;
            }
            let rest = self.buffer.split_off(size);
            let whole = std::mem::replace(&mut self.buffer, rest);
            let kind = [whole[4], whole[5], whole[6], whole[7]];
            out.push((kind, whole));
        }
        Ok(out)
    }
}

#[cfg(test)]
pub mod build {
    pub fn mp4_box(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut out = ((body.len() + 8) as u32).to_be_bytes().to_vec();
        out.extend_from_slice(kind);
        out.extend_from_slice(body);
        out
    }

    pub fn full(version: u8, flags: u32, rest: &[u8]) -> Vec<u8> {
        let mut out = (((version as u32) << 24) | flags).to_be_bytes().to_vec();
        out.extend_from_slice(rest);
        out
    }

    pub fn moov(timescale: u32) -> Vec<u8> {
        let mut mdhd = vec![0u8; 8];
        mdhd.extend_from_slice(&timescale.to_be_bytes());
        mdhd.extend_from_slice(&[0u8; 8]);
        let mdhd = mp4_box(b"mdhd", &full(0, 0, &mdhd));
        let trak = mp4_box(b"trak", &mp4_box(b"mdia", &mdhd));
        let mut trex = Vec::new();
        for v in [1u32, 1, 0, 0, 0] {
            trex.extend_from_slice(&v.to_be_bytes());
        }
        let mvex = mp4_box(b"mvex", &mp4_box(b"trex", &full(0, 0, &trex)));
        [trak, mvex].concat()
    }

    pub fn moof(decode_time: u64, samples: u32, sample_duration: u32, keyframe: bool) -> Vec<u8> {
        let mut tfhd = 1u32.to_be_bytes().to_vec();
        tfhd.extend_from_slice(&sample_duration.to_be_bytes());
        tfhd.extend_from_slice(&0x0101_0000u32.to_be_bytes());
        let tfhd = mp4_box(b"tfhd", &full(0, 0x20028, &tfhd));
        let tfdt = mp4_box(b"tfdt", &full(1, 0, &decode_time.to_be_bytes()));
        let mut trun = samples.to_be_bytes().to_vec();
        trun.extend_from_slice(&0u32.to_be_bytes());
        let mut flags = 0x1;
        if keyframe {
            flags |= 0x4;
            trun.extend_from_slice(&0x0200_0000u32.to_be_bytes());
        }
        let trun = mp4_box(b"trun", &full(0, flags, &trun));
        mp4_box(b"traf", &[tfhd, tfdt, trun].concat())
    }
}

#[cfg(test)]
mod tests {
    use super::build::*;
    use super::*;

    fn track() -> TrackInfo {
        parse_init(&moov(15360)).unwrap()
    }

    #[test]
    fn init_timescale_drives_every_duration_so_it_must_come_from_mdhd() {
        assert_eq!(track().timescale, 15360);
    }

    #[test]
    fn keyframe_fragment_is_independent_because_players_can_start_decoding_there() {
        let info = parse_fragment(&moof(3072, 6, 512, true), &track()).unwrap();
        assert!(info.independent);
        assert_eq!(info.decode_time, 3072);
        assert_eq!(info.duration, 3072);
    }

    #[test]
    fn delta_fragment_is_not_independent_so_it_cannot_open_a_segment() {
        let info = parse_fragment(&moof(0, 6, 512, false), &track()).unwrap();
        assert!(!info.independent);
    }

    #[test]
    fn splitter_only_emits_complete_boxes_because_pipes_deliver_arbitrary_chunks() {
        let a = mp4_box(b"moof", &[1, 2, 3]);
        let b = mp4_box(b"mdat", &[9; 20]);
        let stream = [a.clone(), b.clone()].concat();
        let mut splitter = BoxSplitter::new();
        let first = splitter.push(&stream[..5]).unwrap();
        assert!(first.is_empty());
        let second = splitter.push(&stream[5..15]).unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(&second[0].0, b"moof");
        assert_eq!(second[0].1, a);
        let third = splitter.push(&stream[15..]).unwrap();
        assert_eq!(third[0].1, b);
    }

    #[test]
    fn truncated_moof_is_rejected_instead_of_producing_a_bogus_part() {
        let mut bad = moof(0, 6, 512, true);
        bad.truncate(bad.len() - 6);
        assert!(parse_fragment(&bad, &track()).is_err());
    }
}
