use crate::error::Result;
use crate::fmp4::{BoxSplitter, TrackInfo, parse_fragment, parse_init};
use crate::live::{LiveStream, Part};
use axum::body::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt};

const HEADER: usize = 8;

pub async fn ingest<R: AsyncRead + Unpin>(mut reader: R, live: &LiveStream, epoch_ms: i64) -> Result<()> {
    let mut splitter = BoxSplitter::new();
    let mut ftyp = Vec::new();
    let mut track: Option<TrackInfo> = None;
    let mut moof: Option<Vec<u8>> = None;
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            return Ok(());
        }
        for (kind, data) in splitter.push(&buffer[..read])? {
            match &kind {
                b"ftyp" => ftyp = data,
                b"moov" => {
                    track = Some(parse_init(&data[HEADER..])?);
                    let init = Bytes::from([ftyp.as_slice(), &data].concat());
                    live.update(|w| w.init = Some(init));
                }
                b"moof" => moof = Some(data),
                b"mdat" => {
                    let (Some(track), Some(header)) = (track, moof.take()) else {
                        continue;
                    };
                    let info = parse_fragment(&header[HEADER..], &track)?;
                    let timescale = track.timescale.max(1) as u64;
                    let part = Part {
                        duration: info.duration as f64 / timescale as f64,
                        independent: info.independent,
                        data: Bytes::from([header, data].concat()),
                    };
                    let program_date_ms = epoch_ms + (info.decode_time * 1000 / timescale) as i64;
                    live.update(|w| w.push_part(part, program_date_ms));
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fmp4::build::{moof, moov, mp4_box};
    use crate::live::fixture::config;

    fn stream(fragments: usize) -> Vec<u8> {
        let mut out = mp4_box(b"ftyp", b"iso6");
        out.extend(mp4_box(b"moov", &moov(15360)));
        for i in 0..fragments {
            out.extend(mp4_box(b"moof", &moof(i as u64 * 3072, 6, 512, i % 10 == 0)));
            out.extend(mp4_box(b"mdat", &[i as u8; 32]));
        }
        out
    }

    #[tokio::test]
    async fn encoder_stream_becomes_init_plus_timed_parts_grouped_by_keyframe() {
        let live = LiveStream::new(config());
        ingest(stream(23).as_slice(), &live, 5_000).await.unwrap();
        live.read(|w| {
            let init = w.init.as_ref().unwrap();
            assert_eq!(&init[4..8], b"ftyp");
            assert_eq!(w.segments.len(), 3);
            assert_eq!(w.segments[0].parts.len(), 10);
            assert!((w.segments[0].duration() - 2.0).abs() < 1e-9);
            assert_eq!(w.segments[1].program_date_ms, 7_000);
            let part = &w.segments[2].parts[1].data;
            assert_eq!(&part[4..8], b"moof");
        });
    }

    #[tokio::test]
    async fn mdat_before_init_is_ignored_because_it_cannot_be_timed() {
        let live = LiveStream::new(config());
        let mut bytes = mp4_box(b"moof", &moof(0, 6, 512, true));
        bytes.extend(mp4_box(b"mdat", &[1; 8]));
        ingest(bytes.as_slice(), &live, 0).await.unwrap();
        assert!(live.read(|w| w.segments.is_empty()));
    }
}
