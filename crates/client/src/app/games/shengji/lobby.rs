//! 升级房间规则与玩家席位界面。

use super::ShengjiUiAction;
use crate::app::presentation::{
    EditableRuleSet, MUTED, RuleConfigRow, add_rule_config_row, add_section_title, add_text,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{LobbyPage, LobbyPageStyle, LobbyPlayerSection, UiAction};
use bevy::prelude::*;
use leocard_protocol::LobbySnapshot;
use leocard_shengji::{ShengjiRuleSet, ShengjiThrowPenalty};

type ShengjiRuleConfigRow<'a> = RuleConfigRow<'a, ShengjiRuleSet>;

impl EditableRuleSet for ShengjiRuleSet {
    const ROW_HEIGHT: f32 = 38.0;
    const VALUE_WIDTH: f32 = 92.0;

    fn update_action(self) -> UiAction {
        UiAction::Shengji(ShengjiUiAction::UpdateRules(self))
    }
}

pub(crate) fn render_shengji_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules_value = *lobby.rules.shengji().expect("双升大厅应携带对应规则");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        LobbyPageStyle::default(),
    );
    let rules_panel = page.rules;
    let connected_count = page.connected_count;
    let can_configure = page.can_configure;
    add_section_title(commands, rules_panel, "双升配置", assets);
    add_text(
        commands,
        rules_panel,
        format!("当前人数 {connected_count}/4，需要四人开局。"),
        13.0,
        MUTED,
        assets,
    );
    let previous_deck_count = (rules_value.deck_count > 2).then_some(ShengjiRuleSet {
        deck_count: rules_value.deck_count - 1,
        ..rules_value
    });
    let next_deck_count = (rules_value.deck_count < 4).then_some(ShengjiRuleSet {
        deck_count: rules_value.deck_count + 1,
        ..rules_value
    });
    add_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "牌副数",
            value: format!("{} 副", rules_value.deck_count),
            help: "两副牌：25 张手牌、8 张底牌；三副牌：39 张手牌、6 张底牌，加入三同张和泰坦尼克；四副牌：52 张手牌、8 张底牌，再加入炸弹和宇宙飞船。",
            editable: can_configure,
            previous: previous_deck_count.filter(|_| can_configure),
            next: next_deck_count.filter(|_| can_configure),
        },
        assets,
    );
    let throw_toggled = ShengjiRuleSet {
        allow_throw: !rules_value.allow_throw,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "允许甩牌",
            value: if rules_value.allow_throw {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启后首家可以甩出同门的单张、对子和拖拉机组合；甩牌失败会被强制改出最小可失败牌型。",
            editable: can_configure,
            previous: can_configure.then_some(throw_toggled),
            next: can_configure.then_some(throw_toggled),
        },
        assets,
    );
    let bottom_copy_toggled = ShengjiRuleSet {
        bottom_copy: !rules_value.bottom_copy,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "抄底",
            value: if rules_value.bottom_copy {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "庄家埋底后，从庄家下家开始依次询问可反主的玩家；每次抄底都公开反主牌、取得当前底牌并重新埋底，然后继续询问，直到一整轮无人再抄底。抄底永远不改变庄家，但实际通过扳底定庄的对局禁用。",
            editable: can_configure,
            previous: can_configure.then_some(bottom_copy_toggled),
            next: can_configure.then_some(bottom_copy_toggled),
        },
        assets,
    );
    let penalties = [
        ShengjiThrowPenalty::None,
        ShengjiThrowPenalty::FivePerCard,
        ShengjiThrowPenalty::TenPerCard,
    ];
    let penalty_index = penalties
        .iter()
        .position(|penalty| *penalty == rules_value.throw_penalty)
        .unwrap_or(0);
    let previous_penalty = penalties[(penalty_index + penalties.len() - 1) % penalties.len()];
    let next_penalty = penalties[(penalty_index + 1) % penalties.len()];
    add_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "甩牌罚分",
            value: match rules_value.throw_penalty {
                ShengjiThrowPenalty::None => "不罚分",
                ShengjiThrowPenalty::FivePerCard => "每张 5 分",
                ShengjiThrowPenalty::TenPerCard => "每张 10 分",
            }
            .to_owned(),
            help: "庄家方罚分会给闲家加分；闲家方罚分从闲家总分扣除，最低为零。",
            editable: can_configure,
            previous: can_configure.then_some(ShengjiRuleSet {
                throw_penalty: previous_penalty,
                ..rules_value
            }),
            next: can_configure.then_some(ShengjiRuleSet {
                throw_penalty: next_penalty,
                ..rules_value
            }),
        },
        assets,
    );
    let mandatory_toggled = ShengjiRuleSet {
        mandatory_five_ten_king_ace: !rules_value.mandatory_five_ten_king_ace,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "必打 5/10/K/A",
            value: if rules_value.mandatory_five_ten_king_ace {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启后升级跨过 5、10、K、A 时必须先停在对应等级。",
            editable: can_configure,
            previous: can_configure.then_some(mandatory_toggled),
            next: can_configure.then_some(mandatory_toggled),
        },
        assets,
    );
    let bid_with_joker_toggled = ShengjiRuleSet {
        bid_with_joker: !rules_value.bid_with_joker,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "带王亮",
            value: if rules_value.bid_with_joker {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启后红桃/方块必须带一张大王，梅花/黑桃必须带一张小王；本人已经亮过的王可以复用。无主不能首亮，只能用于反主。",
            editable: can_configure,
            previous: can_configure.then_some(bid_with_joker_toggled),
            next: can_configure.then_some(bid_with_joker_toggled),
        },
        assets,
    );
    let power_outage_dealer_toggled = ShengjiRuleSet {
        power_outage_dealer: !rules_value.power_outage_dealer,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "断电换庄",
            value: if rules_value.power_outage_dealer {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "第一次无人亮主时保持手牌不动，由当前庄家的下家直接接庄，改打新庄家一方的级牌并重新亮主 10 秒；不产生上台积分。",
            editable: can_configure,
            previous: can_configure.then_some(power_outage_dealer_toggled),
            next: can_configure.then_some(power_outage_dealer_toggled),
        },
        assets,
    );
    let bottom_flip_toggled = ShengjiRuleSet {
        bottom_flip: !rules_value.bottom_flip,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "扳底",
            value: if rules_value.bottom_flip {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "最终无人亮主时逐张翻开底牌，公开各玩家持有的同牌并按两队总数、队内数量和座次确定庄家；王定无主，其他牌定其花色。",
            editable: can_configure,
            previous: can_configure.then_some(bottom_flip_toggled),
            next: can_configure.then_some(bottom_flip_toggled),
        },
        assets,
    );
    let five_trump_crossing_toggled = ShengjiRuleSet {
        five_trump_crossing: !rules_value.five_trump_crossing,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "五主过江",
            value: if rules_value.five_trump_crossing {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "有主局埋底后，主牌不多于五张的玩家可将全部主牌补足五张交给对家，再由对家任选五张归还；每人每局仅可进行一次。",
            editable: can_configure,
            previous: can_configure.then_some(five_trump_crossing_toggled),
            next: can_configure.then_some(five_trump_crossing_toggled),
        },
        assets,
    );
    let constant_trump_toggled = ShengjiRuleSet {
        constant_trump: !rules_value.constant_trump,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "常主 2",
            value: if rules_value.constant_trump {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启后双方从 3 开始，2 永远为主牌；牌力位于本局级牌和主牌 A 之间，并区分主 2 与副 2。",
            editable: can_configure,
            previous: can_configure.then_some(constant_trump_toggled),
            next: can_configure.then_some(constant_trump_toggled),
        },
        assets,
    );

    let can_start = connected_count == ShengjiRuleSet::PLAYER_COUNT
        && lobby
            .players
            .iter()
            .filter(|player| player.connected)
            .all(|player| player.seat.is_some() && player.ready);
    page.render_players(
        commands,
        LobbyPlayerSection::new(
            client,
            lobby,
            assets,
            avatars,
            ShengjiRuleSet::PLAYER_COUNT as u8,
            can_start,
            "等待四名玩家",
        ),
    );
}
