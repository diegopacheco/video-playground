use std::collections::BTreeMap;

use serde::Serialize;

use crate::ass::{Event, Script, Style};
use crate::color::Color;
use crate::overrides::{Tag, Token, paren_args, tokenize};
use crate::time::format_vtt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Kept,
    Approximated,
    Dropped,
}

pub fn classify(tag: &str) -> Outcome {
    match tag {
        "b" | "i" | "u" | "s" | "c" | "1c" | "an" | "a" | "pos" | "k" | "r" => Outcome::Kept,
        "K" | "kf" | "ko" | "move" => Outcome::Approximated,
        _ => Outcome::Dropped,
    }
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LossReport {
    pub cues_in: usize,
    pub cues_out: usize,
    pub kept: BTreeMap<String, usize>,
    pub approximated: BTreeMap<String, usize>,
    pub dropped: BTreeMap<String, usize>,
    pub style_dropped: BTreeMap<String, Vec<String>>,
    pub fidelity: f64,
}

impl LossReport {
    fn add(&mut self, outcome: Outcome, tag: &str, count: usize) {
        let bucket = match outcome {
            Outcome::Kept => &mut self.kept,
            Outcome::Approximated => &mut self.approximated,
            Outcome::Dropped => &mut self.dropped,
        };
        *bucket.entry(tag.to_string()).or_insert(0) += count;
    }

    fn finish(&mut self) {
        let sum = |m: &BTreeMap<String, usize>| m.values().sum::<usize>() as f64;
        let total = sum(&self.kept) + sum(&self.approximated) + sum(&self.dropped);
        self.fidelity = if total == 0.0 {
            1.0
        } else {
            ((sum(&self.kept) + 0.5 * sum(&self.approximated)) / total * 1000.0).round() / 1000.0
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conversion {
    pub vtt: String,
    pub report: LossReport,
}

#[derive(Debug, Clone, PartialEq)]
struct RunState {
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
    color: Option<Color>,
}

impl RunState {
    fn from_style(style: &Style) -> Self {
        RunState {
            bold: style.bold,
            italic: style.italic,
            underline: style.underline,
            strike: style.strike,
            color: None,
        }
    }
}

struct Cue {
    settings: String,
    payload: String,
}

pub fn style_class(name: &str) -> String {
    let clean: String = name.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    format!("s{clean}")
}

pub fn convert(script: &Script) -> Conversion {
    let mut report = LossReport {
        cues_in: script.events.len(),
        ..LossReport::default()
    };
    let mut colors: BTreeMap<String, ()> = BTreeMap::new();
    let mut cues = Vec::new();
    let mut ordered: Vec<&Event> = script.events.iter().collect();
    ordered.sort_by_key(|e| (e.start, e.end, e.line));
    for event in ordered {
        let mut tally = LossReport::default();
        let cue = convert_event(script, event, &mut tally, &mut colors);
        let outcome_override = cue.is_none().then_some(Outcome::Dropped);
        for (outcome, map) in [
            (Outcome::Kept, tally.kept),
            (Outcome::Approximated, tally.approximated),
            (Outcome::Dropped, tally.dropped),
        ] {
            for (tag, n) in map {
                report.add(outcome_override.unwrap_or(outcome), &tag, n);
            }
        }
        if let Some(cue) = cue {
            cues.push((event.start, event.end, cue));
        }
    }
    report.cues_out = cues.len();
    for style in &script.styles {
        let lost = style_losses(style);
        if !lost.is_empty() {
            report.style_dropped.insert(style.name.clone(), lost);
        }
    }
    report.finish();

    let mut vtt = String::from("WEBVTT\n\nSTYLE\n");
    for style in &script.styles {
        vtt.push_str(&format!(
            "::cue(.{}) {{ color: #{}; font-family: \"{}\", sans-serif; }}\n",
            style_class(&style.name),
            style.primary.css_hex(),
            style.font_name
        ));
    }
    for hex in colors.keys() {
        vtt.push_str(&format!("::cue(.c{hex}) {{ color: #{hex}; }}\n"));
    }
    vtt.push_str("::cue(.strike) { text-decoration: line-through; }\n");
    for (i, (start, end, cue)) in cues.iter().enumerate() {
        vtt.push_str(&format!(
            "\n{}\n{} --> {}{}\n{}\n",
            i + 1,
            format_vtt(*start),
            format_vtt(*end),
            cue.settings,
            cue.payload
        ));
    }
    Conversion { vtt, report }
}

fn style_losses(style: &Style) -> Vec<String> {
    let mut lost = Vec::new();
    if style.outline > 0.0 {
        lost.push("outline".to_string());
    }
    if style.shadow > 0.0 {
        lost.push("shadow".to_string());
    }
    if style.angle != 0.0 {
        lost.push("angle".to_string());
    }
    if style.scale_x != 100.0 || style.scale_y != 100.0 {
        lost.push("scale".to_string());
    }
    lost.push("font-size".to_string());
    lost
}

fn convert_event(
    script: &Script,
    event: &Event,
    tally: &mut LossReport,
    colors: &mut BTreeMap<String, ()>,
) -> Option<Cue> {
    let mut style = script.style(&event.style);
    let base = style;
    let mut state = RunState::from_style(style);
    let mut alignment: Option<u8> = None;
    let mut position: Option<(f64, f64)> = None;
    let mut drawing = false;
    let mut karaoke_ms = event.start;
    let mut payload = String::new();
    let mut has_text = false;

    for token in tokenize(&event.text, script.info.wrap_style) {
        match token {
            Token::Text(text) => {
                if drawing {
                    continue;
                }
                if !text.trim().is_empty() {
                    has_text = true;
                }
                payload.push_str(&render_run(&text, &state, colors));
            }
            Token::LineBreak => {
                if !drawing {
                    payload.push('\n');
                }
            }
            Token::Tags(tags) => {
                for tag in tags {
                    let outcome = apply_tag(
                        &tag,
                        script,
                        &mut style,
                        &mut state,
                        &mut alignment,
                        &mut position,
                        &mut drawing,
                    );
                    if karaoke(&tag.name) {
                        if karaoke_ms > event.start && karaoke_ms < event.end {
                            payload.push_str(&format!("<{}>", format_vtt(karaoke_ms)));
                        }
                        karaoke_ms += tag.arg.parse::<u64>().unwrap_or(0) * 10;
                    }
                    tally.add(outcome, &tag.name, 1);
                }
            }
        }
    }
    let payload = payload.trim_matches('\n').to_string();
    if !has_text {
        return None;
    }
    let align = alignment.unwrap_or(base.alignment);
    let settings = cue_settings(align, position, script.info.play_res_x, script.info.play_res_y);
    Some(Cue {
        settings,
        payload: format!("<c.{}>{}</c>", style_class(&base.name), payload),
    })
}

fn karaoke(name: &str) -> bool {
    matches!(name, "k" | "K" | "kf" | "ko")
}

fn apply_tag<'a>(
    tag: &Tag,
    script: &'a Script,
    style: &mut &'a Style,
    state: &mut RunState,
    alignment: &mut Option<u8>,
    position: &mut Option<(f64, f64)>,
    drawing: &mut bool,
) -> Outcome {
    let toggle = |arg: &str, from_style: bool| -> bool {
        arg.parse::<i32>().map(|n| n != 0).unwrap_or(from_style)
    };
    match tag.name.as_str() {
        "b" => state.bold = toggle(&tag.arg, style.bold),
        "i" => state.italic = toggle(&tag.arg, style.italic),
        "u" => state.underline = toggle(&tag.arg, style.underline),
        "s" => state.strike = toggle(&tag.arg, style.strike),
        "c" | "1c" => {
            state.color = if tag.arg.is_empty() {
                None
            } else {
                match Color::parse(&tag.arg) {
                    Ok(c) if c == style.primary => None,
                    Ok(c) => Some(c),
                    Err(_) => return Outcome::Dropped,
                }
            }
        }
        "an" => {
            if alignment.is_none() {
                match tag.arg.parse::<u8>() {
                    Ok(n) if (1..=9).contains(&n) => *alignment = Some(n),
                    _ => return Outcome::Dropped,
                }
            }
        }
        "a" => {
            if alignment.is_none() {
                match tag.arg.parse::<u8>() {
                    Ok(n) => *alignment = Some(crate::ass::legacy_alignment(n)),
                    Err(_) => return Outcome::Dropped,
                }
            }
        }
        "pos" | "move" => {
            let args = paren_args(&tag.arg);
            if args.len() < 2 {
                return Outcome::Dropped;
            }
            if position.is_none() {
                *position = Some((args[0], args[1]));
            }
        }
        "r" => {
            let target = if tag.arg.is_empty() {
                *style
            } else {
                script.style(&tag.arg)
            };
            *style = target;
            *state = RunState::from_style(target);
        }
        "p" => *drawing = tag.arg.parse::<u32>().is_ok_and(|n| n > 0),
        _ => {}
    }
    classify(&tag.name)
}

fn render_run(text: &str, state: &RunState, colors: &mut BTreeMap<String, ()>) -> String {
    let mut out = escape(text);
    if state.strike {
        out = format!("<c.strike>{out}</c>");
    }
    if state.underline {
        out = format!("<u>{out}</u>");
    }
    if state.italic {
        out = format!("<i>{out}</i>");
    }
    if state.bold {
        out = format!("<b>{out}</b>");
    }
    if let Some(color) = state.color {
        let hex = color.css_hex();
        colors.insert(hex.clone(), ());
        out = format!("<c.c{hex}>{out}</c>");
    }
    out
}

pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn pct(value: f64) -> String {
    let rounded = (value.clamp(0.0, 100.0) * 100.0).round() / 100.0;
    format!("{rounded}%")
}

pub fn cue_settings(alignment: u8, position: Option<(f64, f64)>, res_x: u32, res_y: u32) -> String {
    let column = (alignment - 1) % 3;
    let row = (alignment - 1) / 3;
    let (h_anchor, align) = match column {
        0 => ("line-left", "start"),
        1 => ("center", "center"),
        _ => ("line-right", "end"),
    };
    let v_anchor = match row {
        0 => "end",
        1 => "center",
        _ => "start",
    };
    match position {
        Some((x, y)) => format!(
            " line:{},{v_anchor} position:{},{h_anchor} align:{align}",
            pct(y / res_y as f64 * 100.0),
            pct(x / res_x as f64 * 100.0)
        ),
        None => {
            let line = match row {
                0 => String::new(),
                1 => " line:50%,center".to_string(),
                _ => " line:5%,start".to_string(),
            };
            let horizontal = match column {
                0 => " position:5%,line-left align:start",
                1 => "",
                _ => " position:95%,line-right align:end",
            };
            format!("{line}{horizontal}")
        }
    }
}
