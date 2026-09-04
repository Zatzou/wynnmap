#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl From<[u8; 3]> for Color {
    #[inline(always)]
    fn from(value: [u8; 3]) -> Self {
        let [r, g, b] = value;
        Self { r, g, b }
    }
}

impl From<Color> for [u8; 3] {
    fn from(Color { r, g, b }: Color) -> Self {
        [r, g, b]
    }
}

impl Color {
    /// Get a color using the wynntils guild color hashing
    #[inline]
    pub fn from_hash(hashable: impl AsRef<[u8]>) -> Self {
        let [_, r, g, b] = crc32fast::hash(hashable.as_ref()).to_be_bytes();

        [r, g, b].into()
    }

    /// Parse a color from a 6 or 8 digit hex code ignoring alpha
    #[inline]
    pub fn from_hex(hex_col: impl AsRef<str>) -> Option<Self> {
        let col = hex_col
            .as_ref()
            .strip_prefix("#")
            .unwrap_or(hex_col.as_ref());

        match col.len() {
            6 => {
                let [_, r, g, b] = u32::from_str_radix(col, 16).unwrap_or(0).to_be_bytes();

                Some([r, g, b].into())
            }
            // for 8 digit values just discard the alpha
            8 => {
                let [r, g, b, _] = u32::from_str_radix(col, 16).unwrap_or(0).to_be_bytes();

                Some([r, g, b].into())
            }
            _ => None,
        }
    }

    /// Format the color as an hex color
    #[inline]
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Format the color as css rgb numbers
    #[inline]
    pub fn to_rgb_values(&self) -> String {
        format!("{} {} {}", self.r, self.g, self.b)
    }

    /// Format the color as css rgb
    #[inline]
    pub fn to_rgb(&self) -> String {
        format!("rgb({} {} {})", self.r, self.g, self.b)
    }

    /// Format the color as css rgb with alpha
    #[inline]
    pub fn to_rgb_with_alpha(&self, alpha: f32) -> String {
        format!("rgb({} {} {} / {})", self.r, self.g, self.b, alpha)
    }
}
