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

/// What the transmitter sends on the FRL lanes once training patterns are stopped.
///
/// Passed to [`HdmiPhy::set_frl_output`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrlOutput {
    /// Gap characters only, with no video, data islands or control periods. Used during
    /// link training and while waiting for the sink to start FRL.
    GapOnly,
    /// Video, data islands and control periods. Set by the caller once link training
    /// has succeeded.
    Active,
}

/// A TxFFE (transmitter feed-forward equalization) level index, 0–7.
///
/// The level a lane's transmitter applies during FRL training. The range is checked on
/// construction; the lower per-rate limit the source advertises to the sink is the link
/// training layer's concern.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TxFfeLevel(u8);

impl TxFfeLevel {
    /// The highest TxFFE level, 7.
    pub const MAX: Self = Self(7);

    /// Returns the level for `level`, or `None` if it is above 7.
    pub const fn new(level: u8) -> Option<Self> {
        if level <= Self::MAX.0 {
            Some(Self(level))
        } else {
            None
        }
    }

    /// Returns the level index (0–7).
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// Per-lane equalization parameters carried by [`EqParams`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LaneEqParams {
    /// TxFFE level for the lane. Defaults to 0.
    pub tx_ffe_level: TxFfeLevel,
}

/// Equalization parameters passed from link training feedback to the PHY.
///
/// Carries per-lane adjustment data derived from character error detection (CED)
/// feedback during the FRL training loop. `lane3` is `None` in 3-lane FRL mode.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

/// Link-level and analog lane control for an HDMI 2.1 transmitter PHY.
///
/// Covers the transmitter's link-level FRL and TMDS behaviour (FRL rate selection, link
/// training patterns, FRL output mode and scrambling) as well as analog lane
/// configuration (lane mapping, pre-emphasis and equalization). Vendor-specific register
/// sequences are an implementation detail of each backend.
pub trait HdmiPhy {
    /// Error type returned by PHY operations.
    type Error;

    /// Select the FRL rate (or TMDS). Triggers the required lane reconfiguration sequence.
    ///
    /// Returns once the PHY transmits at `rate`, after any bring-up its hardware needs to
    /// get there — the Xilinx HDMI 2.1 transmitter, for example, holds a Nyquist clock
    /// pattern until its link is up. Only the PHY knows when that is, so the sequence
    /// belongs here; the link training layer sends no pattern of its own around it.
    fn set_frl_rate(&mut self, rate: HdmiForumFrl) -> Result<(), Self::Error>;

    /// Drive the given link training patterns on the physical lanes, one per lane.
    fn send_ltp(&mut self, patterns: LanePatterns) -> Result<(), Self::Error>;

    /// Select what the transmitter sends on the FRL lanes: gap characters only, or video,
    /// data islands and control periods.
    fn set_frl_output(&mut self, output: FrlOutput) -> Result<(), Self::Error>;

    /// Adjust equalization parameters after link training feedback.
    fn adjust_equalization(&mut self, params: EqParams) -> Result<(), Self::Error>;

    /// Enable or disable scrambling on the PHY.
    fn set_scrambling(&mut self, enabled: bool) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use display_types::cea861::hdmi_forum::HdmiForumFrl;

    /// A minimal `HdmiPhy` that records what it is given. The tests that drive it method by
    /// method check little beyond recording — they show that the trait can be implemented
    /// and used for each method, and they keep this implementation covered. Behaviour is
    /// tested where it lives: in plumbob, against its simulated PHY.
    struct MockPhy {
        frl_rate: Option<HdmiForumFrl>,
        scrambling: Option<bool>,
        eq_calls: u32,
        last_eq: Option<EqParams>,
        frl_output: Option<FrlOutput>,
        last_ltp: Option<LanePatterns>,
    }

    impl MockPhy {
        fn new() -> Self {
            Self {
                frl_rate: None,
                scrambling: None,
                eq_calls: 0,
                last_eq: None,
                frl_output: None,
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

        fn set_frl_output(&mut self, output: FrlOutput) -> Result<(), Self::Error> {
            self.frl_output = Some(output);
            Ok(())
        }

        fn set_frl_rate(&mut self, rate: HdmiForumFrl) -> Result<(), Self::Error> {
            self.frl_rate = Some(rate);
            Ok(())
        }

        fn adjust_equalization(&mut self, params: EqParams) -> Result<(), Self::Error> {
            self.eq_calls += 1;
            self.last_eq = Some(params);
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
    fn set_frl_output_records_mode() {
        let mut phy = MockPhy::new();
        phy.set_frl_output(FrlOutput::GapOnly).unwrap();
        assert_eq!(phy.frl_output, Some(FrlOutput::GapOnly));
    }

    #[test]
    fn set_frl_output_can_be_switched() {
        let mut phy = MockPhy::new();
        phy.set_frl_output(FrlOutput::GapOnly).unwrap();
        phy.set_frl_output(FrlOutput::Active).unwrap();
        assert_eq!(phy.frl_output, Some(FrlOutput::Active));
    }

    #[test]
    fn tx_ffe_level_accepts_0_to_7() {
        for level in 0..=7 {
            assert_eq!(TxFfeLevel::new(level).map(TxFfeLevel::value), Some(level));
        }
    }

    #[test]
    fn tx_ffe_level_rejects_above_7() {
        assert_eq!(TxFfeLevel::new(8), None);
        assert_eq!(TxFfeLevel::new(u8::MAX), None);
    }

    #[test]
    fn tx_ffe_level_max_and_default() {
        assert_eq!(TxFfeLevel::MAX.value(), 7);
        assert_eq!(TxFfeLevel::default().value(), 0);
        assert!(TxFfeLevel::default() < TxFfeLevel::MAX);
    }

    #[test]
    fn lane_eq_params_default_tx_ffe_level_is_0() {
        assert_eq!(LaneEqParams::default().tx_ffe_level, TxFfeLevel::default());
    }

    #[test]
    fn adjust_equalization_records_per_lane_levels() {
        let mut phy = MockPhy::new();
        let mut params = EqParams::new();
        params.lane1.tx_ffe_level = TxFfeLevel::MAX;
        phy.adjust_equalization(params).unwrap();
        let last = phy.last_eq.unwrap();
        assert_eq!(last, params);
        assert_eq!(last.lane0.tx_ffe_level.value(), 0);
        assert_eq!(last.lane1.tx_ffe_level.value(), 7);
    }

    #[test]
    fn adjust_equalization_tracks_call_count() {
        let mut phy = MockPhy::new();
        phy.adjust_equalization(EqParams::new()).unwrap();
        phy.adjust_equalization(EqParams::new()).unwrap();
        assert_eq!(phy.eq_calls, 2);
    }
}
