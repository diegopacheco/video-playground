use crate::color::Color;
use crate::error::SubError;
use crate::time::parse_ass_time;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptKind {
    Ass,
    Ssa,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptInfo {
    pub title: String,
    pub kind: ScriptKind,
    pub play_res_x: u32,
    pub play_res_y: u32,
    pub wrap_style: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub name: String,
    pub font_name: String,
    pub font_size: f32,
    pub primary: Color,
    pub outline_color: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub scale_x: f32,
    pub scale_y: f32,
    pub angle: f32,
    pub outline: f32,
    pub shadow: f32,
    pub alignment: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub line: usize,
    pub layer: i32,
    pub start: u64,
    pub end: u64,
    pub style: String,
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    pub info: ScriptInfo,
    pub styles: Vec<Style>,
    pub events: Vec<Event>,
}

impl Script {
    pub fn style(&self, name: &str) -> &Style {
        let wanted = name.trim_start_matches('*');
        self.styles
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(wanted))
            .or_else(|| self.styles.iter().find(|s| s.name.eq_ignore_ascii_case("Default")))
            .unwrap_or(&FALLBACK_STYLE)
    }
}

pub static FALLBACK_STYLE: Style = Style {
    name: String::new(),
    font_name: String::new(),
    font_size: 20.0,
    primary: Color::WHITE,
    outline_color: Color {
        r: 0,
        g: 0,
        b: 0,
        alpha: 0,
    },
    bold: false,
    italic: false,
    underline: false,
    strike: false,
    scale_x: 100.0,
    scale_y: 100.0,
    angle: 0.0,
    outline: 2.0,
    shadow: 2.0,
    alignment: 2,
};

pub trait SubtitleParser: Send + Sync {
    fn parse(&self, input: &str) -> Result<Script, SubError>;
}

pub struct AssParser;

#[derive(PartialEq)]
enum Section {
    None,
    Info,
    Styles,
    Events,
    Other,
}

const DEFAULT_STYLE_FORMAT: &str = "Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding";
const DEFAULT_EVENT_FORMAT: &str =
    "Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text";

impl SubtitleParser for AssParser {
    fn parse(&self, input: &str) -> Result<Script, SubError> {
        let mut info = ScriptInfo {
            title: String::new(),
            kind: ScriptKind::Ass,
            play_res_x: 0,
            play_res_y: 0,
            wrap_style: 0,
        };
        let mut styles = Vec::new();
        let mut events = Vec::new();
        let mut section = Section::None;
        let mut style_format = split_format(DEFAULT_STYLE_FORMAT);
        let mut event_format = split_format(DEFAULT_EVENT_FORMAT);
        let mut saw_info = false;

        for (idx, raw) in input.trim_start_matches('\u{feff}').lines().enumerate() {
            let line_no = idx + 1;
            let line = raw.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with("!:") {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = match line.to_ascii_lowercase().as_str() {
                    "[script info]" => {
                        saw_info = true;
                        Section::Info
                    }
                    "[v4+ styles]" => Section::Styles,
                    "[v4 styles]" => {
                        info.kind = ScriptKind::Ssa;
                        Section::Styles
                    }
                    "[events]" => Section::Events,
                    _ => Section::Other,
                };
                continue;
            }
            let Some((key, value)) = line.split_once(':') else {
                if section == Section::Events || section == Section::Styles {
                    return Err(SubError::parse(line_no, "expected 'Key: value'"));
                }
                continue;
            };
            let key = key.trim();
            let value = value.trim_start();
            match section {
                Section::Info => apply_info(&mut info, key, value, line_no)?,
                Section::Styles if key.eq_ignore_ascii_case("Format") => {
                    style_format = split_format(value)
                }
                Section::Styles if key.eq_ignore_ascii_case("Style") => {
                    styles.push(parse_style(&style_format, value, info.kind, line_no)?)
                }
                Section::Events if key.eq_ignore_ascii_case("Format") => {
                    event_format = split_format(value)
                }
                Section::Events if key.eq_ignore_ascii_case("Dialogue") => {
                    events.push(parse_event(&event_format, value, line_no)?)
                }
                Section::None => {
                    return Err(SubError::parse(line_no, "content before any section"));
                }
                _ => {}
            }
        }

        if !saw_info {
            return Err(SubError::parse(1, "missing [Script Info] section"));
        }
        if info.play_res_x == 0 && info.play_res_y == 0 {
            info.play_res_x = 384;
            info.play_res_y = 288;
        } else if info.play_res_x == 0 {
            info.play_res_x = info.play_res_y * 4 / 3;
        } else if info.play_res_y == 0 {
            info.play_res_y = info.play_res_x * 3 / 4;
        }
        Ok(Script {
            info,
            styles,
            events,
        })
    }
}

fn apply_info(info: &mut ScriptInfo, key: &str, value: &str, line: usize) -> Result<(), SubError> {
    let number = |v: &str| -> Result<u32, SubError> {
        v.trim()
            .parse()
            .map_err(|_| SubError::parse(line, format!("{key} must be a number")))
    };
    match key.to_ascii_lowercase().as_str() {
        "title" => info.title = value.trim().to_string(),
        "scripttype" => {
            if value.trim().eq_ignore_ascii_case("v4.00") {
                info.kind = ScriptKind::Ssa;
            }
        }
        "playresx" => info.play_res_x = number(value)?,
        "playresy" => info.play_res_y = number(value)?,
        "wrapstyle" => info.wrap_style = number(value)? as u8,
        _ => {}
    }
    Ok(())
}

fn split_format(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|f| f.trim().to_ascii_lowercase())
        .collect()
}

fn fields<'a>(format: &[String], value: &'a str, line: usize) -> Result<Vec<&'a str>, SubError> {
    let parts: Vec<&str> = value.splitn(format.len(), ',').collect();
    if parts.len() < format.len() {
        return Err(SubError::parse(
            line,
            format!("expected {} fields, found {}", format.len(), parts.len()),
        ));
    }
    Ok(parts)
}

fn field<'a>(format: &[String], parts: &[&'a str], name: &str) -> Option<&'a str> {
    format.iter().position(|f| f == name).map(|i| parts[i].trim())
}

fn flag(value: Option<&str>) -> bool {
    value
        .and_then(|v| v.parse::<i32>().ok())
        .is_some_and(|n| n != 0)
}

fn float(value: Option<&str>, default: f32, line: usize, name: &str) -> Result<f32, SubError> {
    match value {
        None | Some("") => Ok(default),
        Some(v) => v
            .parse()
            .map_err(|_| SubError::parse(line, format!("{name} must be a number, got '{v}'"))),
    }
}

pub fn legacy_alignment(value: u8) -> u8 {
    match value {
        1..=3 => value,
        5..=7 => value + 2,
        9..=11 => value - 5,
        _ => 2,
    }
}

fn parse_style(format: &[String], value: &str, kind: ScriptKind, line: usize) -> Result<Style, SubError> {
    let parts = fields(format, value, line)?;
    let get = |name: &str| field(format, &parts, name);
    let color = |name: &str, default: Color| -> Result<Color, SubError> {
        match get(name) {
            None | Some("") => Ok(default),
            Some(v) => Color::parse(v).map_err(|e| SubError::parse(line, e.to_string())),
        }
    };
    let raw_alignment = float(get("alignment"), 2.0, line, "Alignment")? as u8;
    let alignment = match kind {
        ScriptKind::Ssa => legacy_alignment(raw_alignment),
        ScriptKind::Ass if (1..=9).contains(&raw_alignment) => raw_alignment,
        ScriptKind::Ass => {
            return Err(SubError::parse(line, format!("alignment {raw_alignment} out of 1..9")));
        }
    };
    Ok(Style {
        name: get("name").unwrap_or("Default").to_string(),
        font_name: get("fontname").unwrap_or("").to_string(),
        font_size: float(get("fontsize"), 20.0, line, "Fontsize")?,
        primary: color("primarycolour", Color::WHITE)?,
        outline_color: color("outlinecolour", FALLBACK_STYLE.outline_color)?,
        bold: flag(get("bold")),
        italic: flag(get("italic")),
        underline: flag(get("underline")),
        strike: flag(get("strikeout")),
        scale_x: float(get("scalex"), 100.0, line, "ScaleX")?,
        scale_y: float(get("scaley"), 100.0, line, "ScaleY")?,
        angle: float(get("angle"), 0.0, line, "Angle")?,
        outline: float(get("outline"), 2.0, line, "Outline")?,
        shadow: float(get("shadow"), 2.0, line, "Shadow")?,
        alignment,
    })
}

fn parse_event(format: &[String], value: &str, line: usize) -> Result<Event, SubError> {
    if format.last().map(String::as_str) != Some("text") {
        return Err(SubError::parse(line, "Format must end with Text"));
    }
    let parts = fields(format, value, line)?;
    let get = |name: &str| field(format, &parts, name);
    let time = |name: &str| -> Result<u64, SubError> {
        let raw = get(name).ok_or_else(|| SubError::parse(line, format!("missing {name}")))?;
        parse_ass_time(raw).map_err(|e| SubError::parse(line, e.to_string()))
    };
    let start = time("start")?;
    let end = time("end")?;
    if end <= start {
        return Err(SubError::parse(line, "end must be after start"));
    }
    let layer = get("layer")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    Ok(Event {
        line,
        layer,
        start,
        end,
        style: get("style").unwrap_or("Default").to_string(),
        name: get("name").unwrap_or("").to_string(),
        text: parts[parts.len() - 1].to_string(),
    })
}
