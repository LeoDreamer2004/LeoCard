use super::*;

pub(super) fn render_action_bar(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    assets: &UiAssets,
) {
    let bar = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(300),
            right: px(300),
            bottom: px(88),
            min_height: px(48),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    if let Some(pending) = &game.pending_claim {
        if pending.your_response.is_some() {
            add_text(commands, bar, "已响应，等待其他玩家", 15.0, MUTED, assets);
            return;
        }
        if pending.your_options.is_empty() {
            add_text(commands, bar, "等待其他玩家响应", 15.0, MUTED, assets);
            return;
        }
        for option in &pending.your_options {
            let (label, claim) = match *option {
                MahjongClaimOption::Chow { start } => (
                    format!("吃 {start}{}{}", start + 1, start + 2),
                    MahjongClaim::Chow { start },
                ),
                MahjongClaimOption::Pung => ("碰".to_owned(), MahjongClaim::Pung),
                MahjongClaimOption::Kong => ("杠".to_owned(), MahjongClaim::Kong),
                MahjongClaimOption::Win => ("和".to_owned(), MahjongClaim::Win),
            };
            add_action_button(
                commands,
                bar,
                &label,
                UiAction::MahjongRespond(claim),
                if matches!(claim, MahjongClaim::Win) {
                    ButtonKind::Warning
                } else {
                    ButtonKind::Primary
                },
                assets,
            );
        }
        add_action_button(
            commands,
            bar,
            "过",
            UiAction::MahjongRespond(MahjongClaim::Pass),
            ButtonKind::Pass,
            assets,
        );
        return;
    }
    if !matches!(game.phase, MahjongPhaseView::Playing) || game.current_player != game.you {
        return;
    }
    if game.can_self_draw {
        add_action_button(
            commands,
            bar,
            "自摸",
            UiAction::MahjongSelfDraw,
            ButtonKind::Warning,
            assets,
        );
    }
    for kind in &game.concealed_kong_options {
        add_action_button(
            commands,
            bar,
            &format!("暗杠 {}", kind),
            UiAction::MahjongConcealedKong(*kind),
            ButtonKind::Secondary,
            assets,
        );
    }
    for tile in &game.added_kong_options {
        add_action_button(
            commands,
            bar,
            &format!("加杠 {}", tile.kind()),
            UiAction::MahjongAddedKong(*tile),
            ButtonKind::Secondary,
            assets,
        );
    }
}
