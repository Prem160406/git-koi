//! Sun Wukong's permanent evolution, derived from lifetime GitHub contributions.
//!
//! This module deliberately contains no rendering code. It turns contribution totals
//! into a stable character stage; sprite selection and recent-activity animation belong
//! to later pipeline stages. The default thresholds are prototype values and should be
//! tuned against real account histories before they become a public compatibility promise.

/// Prototype contribution milestones for the five Wukong forms.
///
/// Values are inclusive lower bounds. These are intentionally configurable because
/// account age and contribution distribution may justify adaptive thresholds later.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvolutionThresholds {
    /// First contribution count that unlocks Stone-born Monkey.
    pub stone_born: u64,
    /// First contribution count that unlocks Handsome Monkey King.
    pub monkey_king: u64,
    /// First contribution count that unlocks Great Sage, Heaven's Equal.
    pub great_sage: u64,
    /// First contribution count that unlocks Victorious Fighting Buddha.
    pub fighting_buddha: u64,
}

impl Default for EvolutionThresholds {
    fn default() -> Self {
        Self {
            stone_born: 50,
            monkey_king: 1_000,
            great_sage: 5_000,
            fighting_buddha: 10_000,
        }
    }
}

/// The five permanent forms in Wukong's evolution line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvolutionStage {
    /// The dormant celestial stone, before the monkey emerges.
    CelestialStone,
    /// The newly emerged, small and curious monkey.
    StoneBornMonkey,
    /// The confident Monkey King, with his first staff and headband.
    HandsomeMonkeyKing,
    /// The armoured Great Sage, Heaven's Equal.
    GreatSage,
    /// The final, celestial Victorious Fighting Buddha form.
    VictoriousFightingBuddha,
}

impl EvolutionStage {
    /// Human-readable name used in captions and previews.
    pub const fn name(self) -> &'static str {
        match self {
            Self::CelestialStone => "The Celestial Stone",
            Self::StoneBornMonkey => "Stone-born Monkey",
            Self::HandsomeMonkeyKing => "Handsome Monkey King",
            Self::GreatSage => "Great Sage, Heaven's Equal",
            Self::VictoriousFightingBuddha => "Victorious Fighting Buddha",
        }
    }

    /// Zero-based position in the evolution line.
    pub const fn index(self) -> u8 {
        match self {
            Self::CelestialStone => 0,
            Self::StoneBornMonkey => 1,
            Self::HandsomeMonkeyKing => 2,
            Self::GreatSage => 3,
            Self::VictoriousFightingBuddha => 4,
        }
    }
}

/// Select the permanent form for a lifetime contribution total.
///
/// The default milestones are provisional prototype values: 0–49 stone,
/// 50–999 Stone-born Monkey, 1,000–4,999 Monkey King, 5,000–9,999 Great Sage,
/// and 10,000+ Victorious Fighting Buddha. Inactivity never reverses evolution.
pub fn stage_for_contributions(
    total_contributions: u64,
    thresholds: EvolutionThresholds,
) -> EvolutionStage {
    assert!(
        thresholds.stone_born < thresholds.monkey_king
            && thresholds.monkey_king < thresholds.great_sage
            && thresholds.great_sage < thresholds.fighting_buddha,
        "evolution thresholds must be strictly increasing"
    );

    if total_contributions >= thresholds.fighting_buddha {
        EvolutionStage::VictoriousFightingBuddha
    } else if total_contributions >= thresholds.great_sage {
        EvolutionStage::GreatSage
    } else if total_contributions >= thresholds.monkey_king {
        EvolutionStage::HandsomeMonkeyKing
    } else if total_contributions >= thresholds.stone_born {
        EvolutionStage::StoneBornMonkey
    } else {
        EvolutionStage::CelestialStone
    }
}

/// Return the contribution count needed to unlock the next form, if one remains.
pub fn next_stage_threshold(
    stage: EvolutionStage,
    thresholds: EvolutionThresholds,
) -> Option<u64> {
    match stage {
        EvolutionStage::CelestialStone => Some(thresholds.stone_born),
        EvolutionStage::StoneBornMonkey => Some(thresholds.monkey_king),
        EvolutionStage::HandsomeMonkeyKing => Some(thresholds.great_sage),
        EvolutionStage::GreatSage => Some(thresholds.fighting_buddha),
        EvolutionStage::VictoriousFightingBuddha => None,
    }
}
