use git_koi::wukong::EvolutionStage;
use git_koi::wukong_render::{
    WukongBehavior, render_wukong, render_wukong_environment_frame, render_wukong_frame,
    render_wukong_sprite,
};

const STAGES: [EvolutionStage; 5] = [
    EvolutionStage::CelestialStone,
    EvolutionStage::StoneBornMonkey,
    EvolutionStage::HandsomeMonkeyKing,
    EvolutionStage::GreatSage,
    EvolutionStage::VictoriousFightingBuddha,
];

const BEHAVIORS: [WukongBehavior; 4] = [
    WukongBehavior::Meditating,
    WukongBehavior::Training,
    WukongBehavior::Fighting,
    WukongBehavior::CloudSurfing,
];

#[test]
fn every_evolution_stage_renders_a_full_frame() {
    for stage in STAGES {
        let frame = render_wukong(stage);
        assert_eq!(frame.as_rgba_bytes().len(), 256 * 256 * 4);
        assert!(frame.as_rgba_bytes().chunks_exact(4).any(|pixel| pixel[0..3] != [22, 17, 29]));
    }
}

#[test]
fn each_behavior_loop_is_seamless() {
    for behavior in BEHAVIORS {
        let first = render_wukong_frame(EvolutionStage::HandsomeMonkeyKing, behavior, 0);
        let wrapped = render_wukong_frame(
            EvolutionStage::HandsomeMonkeyKing,
            behavior,
            behavior.frame_count(),
        );
        assert_eq!(
            first.as_rgba_bytes(),
            wrapped.as_rgba_bytes(),
            "{behavior:?} should wrap to its first frame"
        );
    }
}

#[test]
fn fighting_has_a_distinct_frame_from_meditation() {
    let meditating = render_wukong_frame(
        EvolutionStage::GreatSage,
        WukongBehavior::Meditating,
        8,
    );
    let fighting = render_wukong_frame(
        EvolutionStage::GreatSage,
        WukongBehavior::Fighting,
        8,
    );
    assert_ne!(meditating.as_rgba_bytes(), fighting.as_rgba_bytes());
}

#[test]
fn individual_stage_portraits_keep_transparency_and_the_character() {
    for stage in STAGES {
        let portrait = render_wukong_sprite(stage);
        let alpha_channels = portrait
            .as_rgba_bytes()
            .chunks_exact(4)
            .map(|pixel| pixel[3]);
        assert!(alpha_channels.clone().any(|alpha| alpha == 0));
        assert!(alpha_channels.into_iter().any(|alpha| alpha == 255));
    }
}

#[test]
fn environment_preview_omits_the_character_but_keeps_opaque_effects() {
    let frame = render_wukong_environment_frame(
        EvolutionStage::VictoriousFightingBuddha,
        WukongBehavior::CloudSurfing,
        12,
    );
    assert!(
        frame
            .as_rgba_bytes()
            .chunks_exact(4)
            .any(|pixel| pixel[0..3] == [72, 48, 78])
    );
    assert!(
        frame
            .as_rgba_bytes()
            .chunks_exact(4)
            .all(|pixel| pixel[3] == 255)
    );
}
