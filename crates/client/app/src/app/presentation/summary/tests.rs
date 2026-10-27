use super::{
    AnimatedSummaryScore, GameSummaryAnimation, SummaryScore, animate_summary_scores,
    animated_summary_score, animated_summary_value, sorted_summary_scores,
};
use bevy::prelude::*;
use leocard_protocol::{PlayerId, PlayerScore};

#[test]
fn signed_summary_scores_sort_and_animate_without_losing_negative_values() {
    let scores = [
        SummaryScore::signed(PlayerId(0), -6),
        SummaryScore::signed(PlayerId(2), 3),
        SummaryScore::signed(PlayerId(1), 3),
    ];
    let ranked = sorted_summary_scores(&scores);
    assert_eq!(
        ranked.iter().map(|score| score.player).collect::<Vec<_>>(),
        vec![PlayerId(1), PlayerId(2), PlayerId(0)]
    );
    assert_eq!(animated_summary_value(0.0, -6, 0.4), 0);
    assert_eq!(animated_summary_value(2.0, -6, 0.4), -6);
    let unsigned: SummaryScore = PlayerScore {
        player: PlayerId(0),
        score: u32::MAX,
    }
    .into();
    assert!(!unsigned.signed);
    assert_eq!(animated_summary_score(2.0, u32::MAX, 0.4), u32::MAX);

    let mut app = App::new();
    app.insert_resource(GameSummaryAnimation {
        elapsed: 2.0,
        ..default()
    });
    app.add_systems(Update, animate_summary_scores);
    let text = app
        .world_mut()
        .spawn((
            Text::new(""),
            AnimatedSummaryScore {
                target: -6,
                signed: true,
                delay: 0.4,
                unit: "分",
            },
        ))
        .id();
    app.update();
    assert_eq!(app.world().get::<Text>(text).unwrap().0, "-6 分");
}
