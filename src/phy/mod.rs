use display_types::cea861::hdmi_forum::HdmiForumFrl;

/// Equalization parameters passed from link training feedback to the PHY.
///
/// Fields will be refined as the link training layer is implemented.
#[non_exhaustive]
#[derive(Default)]
pub struct EqParams {}

impl EqParams {
    /// Create a new `EqParams` with default values.
    pub fn new() -> Self {
        Self::default()
    }
}

/// PHY lane configuration for an HDMI 2.1 transmitter or receiver.
///
/// Abstracts the register sequences required to configure an HDMI 2.1 PHY: lane
/// mapping, pre-emphasis, equalization, scrambling, and FRL rate selection.
/// Vendor-specific register sequences are an implementation detail of each backend.
pub trait HdmiPhy {
    /// Error type returned by PHY operations.
    type Error;

    /// Select the FRL rate (or TMDS). Triggers the required lane reconfiguration sequence.
    fn set_frl_rate(&mut self, rate: HdmiForumFrl) -> Result<(), Self::Error>;

    /// Adjust equalization parameters after link training feedback.
    fn adjust_equalization(&mut self, params: EqParams) -> Result<(), Self::Error>;

    /// Enable or disable scrambling on the PHY.
    fn set_scrambling(&mut self, enabled: bool) -> Result<(), Self::Error>;
}
