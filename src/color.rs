use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(format!("expected 6 hex characters, got {}", hex.len()));
        }
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
        Ok(Self { r, g, b })
    }

    pub fn to_ratatui_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::Rgb(self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_hex_black() {
        let c = Color::new(0, 0, 0);
        assert_eq!(c.to_hex(), "#000000");
    }

    #[test]
    fn test_to_hex_white() {
        let c = Color::new(255, 255, 255);
        assert_eq!(c.to_hex(), "#FFFFFF");
    }

    #[test]
    fn test_to_hex_color() {
        let c = Color::new(255, 87, 51);
        assert_eq!(c.to_hex(), "#FF5733");
    }

    #[test]
    fn test_from_hex_with_hash() {
        let c = Color::from_hex("#FF5733").unwrap();
        assert_eq!(c, Color::new(255, 87, 51));
    }

    #[test]
    fn test_from_hex_without_hash() {
        let c = Color::from_hex("3498DB").unwrap();
        assert_eq!(c, Color::new(52, 152, 219));
    }

    #[test]
    fn test_from_hex_lowercase() {
        let c = Color::from_hex("#ff5733").unwrap();
        assert_eq!(c, Color::new(255, 87, 51));
    }

    #[test]
    fn test_from_hex_invalid_length() {
        assert!(Color::from_hex("#FFF").is_err());
    }

    #[test]
    fn test_from_hex_invalid_chars() {
        assert!(Color::from_hex("#GGGGGG").is_err());
    }

    #[test]
    fn test_roundtrip() {
        let original = Color::new(123, 45, 67);
        let hex = original.to_hex();
        let parsed = Color::from_hex(&hex).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_to_ratatui_color() {
        let c = Color::new(255, 87, 51);
        assert_eq!(c.to_ratatui_color(), ratatui::style::Color::Rgb(255, 87, 51));
    }
}
