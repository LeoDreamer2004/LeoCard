use super::MahjongUiAction;
use crate::app::presentation::{
    EditableRuleSet, MUTED, RuleConfigRow, add_rule_config_row, add_section_title, add_text,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{LobbyPage, LobbyPageStyle, LobbyPlayerSection, UiAction};
use bevy::prelude::*;
use leocard_mahjong::{MahjongMatchLength, MahjongRuleSet};
use leocard_protocol::LobbySnapshot;

type MahjongRuleConfigRow<'a> = RuleConfigRow<'a, MahjongRuleSet>;

impl EditableRuleSet for MahjongRuleSet {
    const ROW_HEIGHT: f32 = 38.0;
    const VALUE_WIDTH: f32 = 92.0;

    fn update_action(self) -> UiAction {
        UiAction::Mahjong(MahjongUiAction::UpdateRules(self))
    }
}

pub(crate) fn render_mahjong_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules = *lobby.rules.mahjong().expect("麻将大厅应携带对应规则");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        LobbyPageStyle {
            rules_min_width: 340.0,
            rules_basis: 390.0,
            rules_gap: 15.0,
            players_min_width: 500.0,
            players_basis: 650.0,
            players_gap: 11.0,
            wrap: false,
        },
    );
    let rules_panel = page.rules;
    let connected = page.connected_count;
    let can_configure = page.can_configure;
    add_section_title(commands, rules_panel, "国标麻将配置", assets);
    add_text(
        commands,
        rules_panel,
        format!("当前人数 {connected}/4，需要四人开局。采用 2014 版国标规则。"),
        13.0,
        MUTED,
        assets,
    );
    let lengths = [
        MahjongMatchLength::SingleHand,
        MahjongMatchLength::EastRound,
        MahjongMatchLength::HalfGame,
        MahjongMatchLength::FullGame,
    ];
    let index = lengths
        .iter()
        .position(|length| *length == rules.match_length)
        .unwrap_or_default();
    add_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "场次",
            value: match rules.match_length {
                MahjongMatchLength::SingleHand => "单局结算",
                MahjongMatchLength::EastRound => "东风场（4 局）",
                MahjongMatchLength::HalfGame => "半庄场（8 局）",
                MahjongMatchLength::FullGame => "全庄场（16 局）",
            }
            .to_owned(),
            help: "每盘结算后全员可直接准备下一盘，不会返回大厅。单局模式仍会继续轮庄。",
            editable: can_configure,
            previous: can_configure.then_some(MahjongRuleSet {
                match_length: lengths[(index + lengths.len() - 1) % lengths.len()],
                ..rules
            }),
            next: can_configure.then_some(MahjongRuleSet {
                match_length: lengths[(index + 1) % lengths.len()],
                ..rules
            }),
        },
        assets,
    );
    let minimum_toggled = MahjongRuleSet {
        minimum_eight_points: !rules.minimum_eight_points,
        false_win: rules.false_win && !rules.minimum_eight_points,
        ..rules
    };
    add_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "8 番起和",
            value: if rules.minimum_eight_points {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启时严格按 2014 国标要求至少 8 番（花牌不计入起和）；关闭后只按实际番数结算，不另加 8 分。",
            editable: can_configure,
            previous: can_configure.then_some(minimum_toggled),
            next: can_configure.then_some(minimum_toggled),
        },
        assets,
    );
    let winners_toggled = MahjongRuleSet {
        multiple_winners: !rules.multiple_winners,
        ..rules
    };
    add_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "一炮多响",
            value: if rules.multiple_winners {
                "允许"
            } else {
                "截和"
            }
            .to_owned(),
            help: "关闭时按出牌者之后的座次由最近一家截和；开启时所有合法和牌同时结算。",
            editable: can_configure,
            previous: can_configure.then_some(winners_toggled),
            next: can_configure.then_some(winners_toggled),
        },
        assets,
    );
    let false_win_toggled = MahjongRuleSet {
        false_win: !rules.false_win,
        ..rules
    };
    add_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "允许错和",
            value: if !rules.minimum_eight_points {
                "不适用"
            } else if rules.false_win {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启后牌型已经完整便显示和牌按钮；不足 8 番属于错和，向其余三家各付 10 分并公开手牌，本盘继续。",
            editable: can_configure,
            previous: (can_configure && rules.minimum_eight_points).then_some(false_win_toggled),
            next: (can_configure && rules.minimum_eight_points).then_some(false_win_toggled),
        },
        assets,
    );

    let can_start = connected == 4
        && lobby
            .players
            .iter()
            .filter(|player| player.connected)
            .all(|player| player.seat.is_some() && player.ready);
    page.render_players(
        commands,
        LobbyPlayerSection::new(client, lobby, assets, avatars, 4, can_start, "等待四名玩家"),
    );
}
