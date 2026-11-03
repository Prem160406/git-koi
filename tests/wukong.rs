use git_koi::wukong::{
    EvolutionStage, EvolutionThresholds, next_stage_threshold, stage_for_contributions,
};

#[test]
fn default_thresholds_select_each_form_at_its_lower_bound() {
    let thresholds = EvolutionThresholds::default();
    let cases = [
        (0, EvolutionStage::CelestialStone),
        (49, EvolutionStage::CelestialStone),
        (50, EvolutionStage::StoneBornMonkey),
        (999, EvolutionStage::StoneBornMonkey),
        (1_000, EvolutionStage::HandsomeMonkeyKing),
        (4_999, EvolutionStage::HandsomeMonkeyKing),
        (5_000, EvolutionStage::GreatSage),
        (9_999, EvolutionStage::GreatSage),
        (10_000, EvolutionStage::VictoriousFightingBuddha),
        (100_000, EvolutionStage::VictoriousFightingBuddha),
    ];

    for (contributions, expected_stage) in cases {
        assert_eq!(
            stage_for_contributions(contributions, thresholds),
            expected_stage,
            "unexpected stage at {contributions} contributions"
        );
    }
}

#[test]
fn custom_thresholds_can_be_used_without_changing_the_stage_model() {
    let thresholds = EvolutionThresholds {
        stone_born: 10,
        monkey_king: 20,
        great_sage: 30,
        fighting_buddha: 40,
    };

    assert_eq!(
        stage_for_contributions(35, thresholds),
        EvolutionStage::GreatSage
    );
}

#[test]
fn next_stage_threshold_stops_at_the_final_form() {
    let thresholds = EvolutionThresholds::default();

    assert_eq!(
        next_stage_threshold(EvolutionStage::CelestialStone, thresholds),
        Some(50)
    );
    assert_eq!(
        next_stage_threshold(EvolutionStage::GreatSage, thresholds),
        Some(10_000)
    );
    assert_eq!(
        next_stage_threshold(EvolutionStage::VictoriousFightingBuddha, thresholds),
        None
    );
}

#[test]
fn stage_identity_is_stable_for_labels_and_ordering() {
    assert_eq!(EvolutionStage::StoneBornMonkey.name(), "Stone-born Monkey");
    assert_eq!(EvolutionStage::VictoriousFightingBuddha.index(), 4);
}
