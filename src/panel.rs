use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PanelModel {
    Waveshare3InG,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelProfile {
    pub model: PanelModel,
    pub name: &'static str,
    pub width_px: u16,
    pub height_px: u16,
    pub colors: PanelColors,
    pub full_refresh_seconds: u16,
    pub partial_refresh: bool,
    pub interface: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelColors {
    BlackWhiteRedYellow,
}

impl PanelModel {
    pub fn profile(self) -> PanelProfile {
        match self {
            Self::Waveshare3InG => PanelProfile {
                model: self,
                name: "Waveshare 3inch e-Paper (G)",
                width_px: 400,
                height_px: 168,
                colors: PanelColors::BlackWhiteRedYellow,
                full_refresh_seconds: 12,
                partial_refresh: false,
                interface: "SPI",
            },
        }
    }
}

impl PanelProfile {
    pub fn terminal_columns(self, cell_width: u16) -> u16 {
        self.width_px / cell_width
    }

    pub fn terminal_rows(self, cell_height: u16) -> u16 {
        self.height_px / cell_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waveshare_3in_g_profile_matches_product_spec() {
        let profile = PanelModel::Waveshare3InG.profile();

        assert_eq!(profile.width_px, 400);
        assert_eq!(profile.height_px, 168);
        assert_eq!(profile.full_refresh_seconds, 12);
        assert!(!profile.partial_refresh);
        assert_eq!(profile.terminal_columns(8), 50);
        assert_eq!(profile.terminal_rows(12), 14);
    }
}
