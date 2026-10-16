use crate::error::SubError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: u8,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        alpha: 0,
    };

    pub fn parse(input: &str) -> Result<Color, SubError> {
        let bad = || SubError::Color(input.to_string());
        let trimmed = input.trim().trim_end_matches('&');
        let value = if let Some(hex) = trimmed
            .strip_prefix("&H")
            .or_else(|| trimmed.strip_prefix("&h"))
        {
            if hex.is_empty() || hex.len() > 8 {
                return Err(bad());
            }
            u32::from_str_radix(hex, 16).map_err(|_| bad())?
        } else {
            let n: i64 = trimmed.parse().map_err(|_| bad())?;
            u32::try_from(n).map_err(|_| bad())?
        };
        Ok(Color {
            r: (value & 0xFF) as u8,
            g: ((value >> 8) & 0xFF) as u8,
            b: ((value >> 16) & 0xFF) as u8,
            alpha: ((value >> 24) & 0xFF) as u8,
        })
    }

    pub fn css_hex(&self) -> String {
        format!("{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}
