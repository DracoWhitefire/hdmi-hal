use display_types::cea861::hdmi_forum::HdmiForumFrl;

/// A link training pattern to be driven on the physical lanes.
///
/// Produced by the link training state machine and passed to [`HdmiPhy::send_ltp`].
/// The discriminants are the pattern values a sink requests in the SCDC Status_Flags
/// LTP fields. Request values that are not patterns (0 for no pattern, and the TxFFE
/// and rate change requests) are handled by the link training layer and have no variant.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LtpPattern {
    /// All ones.
    AllOnes = 1,
    /// All zeros.
    AllZeros = 2,
    /// Nyquist clock pattern.
    NyquistClock = 3,
    /// DDE (Data Dependent Equalization) compliance pattern.
    DdeCompliance = 4,
    /// LFSR pattern 0.
    Lfsr0 = 5,
    /// LFSR pattern 1.
    Lfsr1 = 6,
    /// LFSR pattern 2.
    Lfsr2 = 7,
    /// LFSR pattern 3.
    Lfsr3 = 8,
}

impl LtpPattern {
    /// Returns the pattern's SCDC value (1–8).
    pub const fn value(self) -> u8 {
        self as u8
    }
}

/// The link training pattern for each lane, passed to [`HdmiPhy::send_ltp`].
///
/// Always the full per-lane set: the PHY applies it as given, and the link training
/// layer tracks which pattern each lane carries. `None` means no training pattern on
/// that lane; `None` on every lane stops the training patterns. `lane3` is `None` in
/// 3-lane FRL mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LanePatterns {
    /// Pattern for lane 0.
    pub lane0: Option<LtpPattern>,
    /// Pattern for lane 1.
    pub lane1: Option<LtpPattern>,
    /// Pattern for lane 2.
    pub lane2: Option<LtpPattern>,
    /// Pattern for lane 3. `None` in 3-lane FRL mode.
    pub lane3: Option<LtpPattern>,
}

/// Per-lane equalization parameters carried by [`EqParams`].
///
/// Fields will be defined as the link training layer is implemented and per-lane
/// hardware requirements become known.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default)]
pub struct LaneEqParams {}

/// Equalization parameters passed from link training feedback to the PHY.
///
/// Carries per-lane adjustment data derived from character error detection (CED)
/// feedback during the FRL training loop. `lane3` is `None` in 3-lane FRL mode.
///
/// Per-lane field contents will be defined as the link training layer is implemented.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default)]
pub struct EqParams {
    /// Equalization parameters for lane 0.
    pub lane0: LaneEqParams,
    /// Equalization parameters for lane 1.
    pub lane1: LaneEqParams,
    /// Equalization parameters for lane 2.
    pub lane2: LaneEqParams,
    /// Equalization parameters for lane 3. `None` in 3-lane FRL mode.
    pub lane3: Option<LaneEqParams>,
}

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

    /// Drive the given link training patterns on the physical lanes, one per lane.
    fn send_ltp(&mut self, patterns: LanePatterns) -> Result<(), Self::Error>;

    /// Adjust equalization parameters after link training feedback.
    fn adjust_equalization(&mut self, params: EqParams) -> Result<(), Self::Error>;

    /// Enable or disable scrambling on the PHY.
    fn set_scrambling(&mut self, enabled: bool) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use display_types::cea861::hdmi_forum::HdmiForumFrl;

    struct MockPhy {
        frl_rate: Option<HdmiForumFrl>,
        scrambling: Option<bool>,
        eq_calls: u32,
        last_ltp: Option<LanePatterns>,
    }

    impl MockPhy {
        fn new() -> Self {
            Self {
                frl_rate: None,
                scrambling: None,
                eq_calls: 0,
                last_ltp: None,
            }
        }
    }

    impl HdmiPhy for MockPhy {
        type Error = core::convert::Infallible;

        fn send_ltp(&mut self, patterns: LanePatterns) -> Result<(), Self::Error> {
            self.last_ltp = Some(patterns);
            Ok(())
        }

        fn set_frl_rate(&mut self, rate: HdmiForumFrl) -> Result<(), Self::Error> {
            self.frl_rate = Some(rate);
            Ok(())
        }

        fn adjust_equalization(&mut self, _params: EqParams) -> Result<(), Self::Error> {
            self.eq_calls += 1;
            Ok(())
        }

        fn set_scrambling(&mut self, enabled: bool) -> Result<(), Self::Error> {
            self.scrambling = Some(enabled);
            Ok(())
        }
    }

    #[test]
    fn ltp_pattern_values_match_scdc_encoding() {
        assert_eq!(LtpPattern::AllOnes.value(), 1);
        assert_eq!(LtpPattern::AllZeros.value(), 2);
        assert_eq!(LtpPattern::NyquistClock.value(), 3);
        assert_eq!(LtpPattern::DdeCompliance.value(), 4);
        assert_eq!(LtpPattern::Lfsr0.value(), 5);
        assert_eq!(LtpPattern::Lfsr1.value(), 6);
        assert_eq!(LtpPattern::Lfsr2.value(), 7);
        assert_eq!(LtpPattern::Lfsr3.value(), 8);
    }

    #[test]
    fn ltp_pattern_clone_eq() {
        let a = LtpPattern::Lfsr1;
        assert_eq!(a, a);
        assert_ne!(LtpPattern::Lfsr0, LtpPattern::Lfsr1);
    }

    #[test]
    fn lane_patterns_default_is_no_pattern() {
        let p = LanePatterns::default();
        assert_eq!(p.lane0, None);
        assert_eq!(p.lane1, None);
        assert_eq!(p.lane2, None);
        assert_eq!(p.lane3, None);
    }

    #[test]
    fn send_ltp_records_patterns() {
        let mut phy = MockPhy::new();
        let patterns = LanePatterns {
            lane0: Some(LtpPattern::Lfsr0),
            lane1: Some(LtpPattern::Lfsr1),
            lane2: Some(LtpPattern::Lfsr2),
            lane3: None,
        };
        phy.send_ltp(patterns).unwrap();
        assert_eq!(phy.last_ltp, Some(patterns));
    }

    #[test]
    fn send_ltp_updates_on_each_call() {
        let mut phy = MockPhy::new();
        let first = LanePatterns {
            lane0: Some(LtpPattern::Lfsr0),
            ..LanePatterns::default()
        };
        let second = LanePatterns {
            lane0: Some(LtpPattern::NyquistClock),
            ..first
        };
        phy.send_ltp(first).unwrap();
        phy.send_ltp(second).unwrap();
        assert_eq!(phy.last_ltp, Some(second));
    }

    #[test]
    fn eq_params_constructors_are_equivalent() {
        let _a = EqParams::new();
        let _b = EqParams::default();
    }

    #[test]
    fn set_frl_rate_records_rate() {
        let mut phy = MockPhy::new();
        phy.set_frl_rate(HdmiForumFrl::Rate6Gbps4Lanes).unwrap();
        assert_eq!(phy.frl_rate, Some(HdmiForumFrl::Rate6Gbps4Lanes));
    }

    #[test]
    fn set_frl_rate_not_supported_is_valid() {
        let mut phy = MockPhy::new();
        phy.set_frl_rate(HdmiForumFrl::NotSupported).unwrap();
        assert_eq!(phy.frl_rate, Some(HdmiForumFrl::NotSupported));
    }

    #[test]
    fn set_frl_rate_can_be_updated() {
        let mut phy = MockPhy::new();
        phy.set_frl_rate(HdmiForumFrl::Rate3Gbps3Lanes).unwrap();
        phy.set_frl_rate(HdmiForumFrl::Rate12Gbps4Lanes).unwrap();
        assert_eq!(phy.frl_rate, Some(HdmiForumFrl::Rate12Gbps4Lanes));
    }

    #[test]
    fn set_scrambling_enable() {
        let mut phy = MockPhy::new();
        phy.set_scrambling(true).unwrap();
        assert_eq!(phy.scrambling, Some(true));
    }

    #[test]
    fn set_scrambling_disable() {
        let mut phy = MockPhy::new();
        phy.set_scrambling(false).unwrap();
        assert_eq!(phy.scrambling, Some(false));
    }

    #[test]
    fn set_scrambling_can_be_toggled() {
        let mut phy = MockPhy::new();
        phy.set_scrambling(true).unwrap();
        phy.set_scrambling(false).unwrap();
        assert_eq!(phy.scrambling, Some(false));
    }

    #[test]
    fn adjust_equalization_tracks_call_count() {
        let mut phy = MockPhy::new();
        phy.adjust_equalization(EqParams::new()).unwrap();
        phy.adjust_equalization(EqParams::new()).unwrap();
        assert_eq!(phy.eq_calls, 2);
    }
}
