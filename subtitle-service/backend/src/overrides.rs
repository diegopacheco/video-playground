#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub name: String,
    pub arg: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    Tags(Vec<Tag>),
    LineBreak,
}

const KNOWN: [&str; 52] = [
    "xbord", "ybord", "xshad", "yshad", "iclip", "alpha", "fscx", "fscy", "blur", "bord", "shad",
    "move", "fade", "clip", "frx", "fry", "frz", "fax", "fay", "fsp", "fad", "org", "pos", "pbo",
    "1c", "2c", "3c", "4c", "1a", "2a", "3a", "4a", "fn", "fs", "fr", "be", "an", "kf", "ko", "b",
    "i", "u", "s", "c", "a", "k", "K", "q", "r", "p", "t", "fe",
];

pub fn tokenize(text: &str, wrap_style: u8) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut buf = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                let mut block = String::new();
                let mut closed = false;
                for c in chars.by_ref() {
                    if c == '}' {
                        closed = true;
                        break;
                    }
                    block.push(c);
                }
                if !closed {
                    buf.push('{');
                    buf.push_str(&block);
                    continue;
                }
                let tags = parse_block(&block);
                if !tags.is_empty() {
                    flush(&mut buf, &mut tokens);
                    tokens.push(Token::Tags(tags));
                }
            }
            '\\' => match chars.peek().copied() {
                Some('N') => {
                    chars.next();
                    flush(&mut buf, &mut tokens);
                    tokens.push(Token::LineBreak);
                }
                Some('n') => {
                    chars.next();
                    if wrap_style == 2 {
                        flush(&mut buf, &mut tokens);
                        tokens.push(Token::LineBreak);
                    } else {
                        buf.push(' ');
                    }
                }
                Some('h') => {
                    chars.next();
                    buf.push('\u{a0}');
                }
                _ => buf.push('\\'),
            },
            _ => buf.push(ch),
        }
    }
    flush(&mut buf, &mut tokens);
    tokens
}

fn flush(buf: &mut String, tokens: &mut Vec<Token>) {
    if !buf.is_empty() {
        tokens.push(Token::Text(std::mem::take(buf)));
    }
}

pub fn parse_block(block: &str) -> Vec<Tag> {
    let mut tags = Vec::new();
    let mut depth = 0usize;
    let mut current: Option<String> = None;
    for c in block.chars() {
        match c {
            '\\' if depth == 0 => {
                if let Some(raw) = current.take() {
                    tags.push(split_tag(&raw));
                }
                current = Some(String::new());
            }
            '(' => {
                depth += 1;
                if let Some(cur) = current.as_mut() {
                    cur.push(c);
                }
            }
            ')' => {
                depth = depth.saturating_sub(1);
                if let Some(cur) = current.as_mut() {
                    cur.push(c);
                }
            }
            _ => {
                if let Some(cur) = current.as_mut() {
                    cur.push(c);
                }
            }
        }
    }
    if let Some(raw) = current {
        tags.push(split_tag(&raw));
    }
    tags.retain(|t| !t.name.is_empty());
    tags
}

fn split_tag(raw: &str) -> Tag {
    let raw = raw.trim();
    let name = KNOWN
        .iter()
        .find(|k| raw.starts_with(*k))
        .map(|k| k.to_string())
        .unwrap_or_else(|| raw.chars().take_while(|c| c.is_ascii_alphanumeric()).collect());
    let arg = raw[name.len()..].trim().to_string();
    let name = if KNOWN.contains(&name.as_str()) || name.is_empty() {
        name
    } else {
        format!("unknown:{name}")
    };
    Tag { name, arg }
}

pub fn paren_args(arg: &str) -> Vec<f64> {
    arg.trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .filter_map(|p| p.trim().parse().ok())
        .collect()
}

pub fn strip_overrides(text: &str, wrap_style: u8) -> String {
    let mut out = String::new();
    let mut drawing = false;
    for token in tokenize(text, wrap_style) {
        match token {
            Token::Text(t) if !drawing => out.push_str(&t),
            Token::Text(_) => {}
            Token::LineBreak => out.push('\n'),
            Token::Tags(tags) => {
                for tag in tags {
                    if tag.name == "p" {
                        drawing = tag.arg.parse::<u32>().is_ok_and(|n| n > 0);
                    }
                }
            }
        }
    }
    out
}
