use super::RoomSession;
use crate::ConnectionId;
use leocard_ai::{DevicePreference, ModelId, ModelRegistry, Network};
use leocard_doudizhu::DoudizhuMode;
use leocard_protocol::{GameRules, GameViolation, RejectReason, RoomViolation};
use leocard_texas_holdem::TexasPpoBot;

impl RoomSession {
    pub(crate) fn configure_bot_ai(
        &mut self,
        connection: ConnectionId,
        model: Option<ModelId>,
        device: DevicePreference,
        game_started: bool,
        rules: GameRules,
    ) -> Result<(), RejectReason> {
        if self.host_connection != Some(connection) {
            return Err(RejectReason::Room(RoomViolation::OnlyHostCanConfigure));
        }
        if game_started {
            return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
        }
        if model.is_some()
            && device == DevicePreference::Gpu
            && Network::available_device(device).is_err()
        {
            return Err(RejectReason::Room(RoomViolation::AiModelUnavailable));
        }
        let selected = match model {
            Some(id) if supports(id, rules, self.players.iter().filter(|p| !p.left).count()) => {
                Some(
                    ModelRegistry::get(id, device)
                        .ok_or(RejectReason::Room(RoomViolation::AiModelUnavailable))?,
                )
            }
            Some(_) => return Err(RejectReason::Room(RoomViolation::AiModelUnavailable)),
            None => None,
        };
        if self.bot_ai != selected {
            self.bot_ai = selected;
            self.rename_bots();
            self.bump_revision();
        }
        Ok(())
    }

    pub(crate) fn retain_supported_bot_ai(&mut self, rules: GameRules) {
        if self.bot_ai.as_ref().is_some_and(|model| {
            !supports(
                model.id,
                rules,
                self.players.iter().filter(|p| !p.left).count(),
            )
        }) {
            self.disable_bot_ai();
        }
    }

    pub(crate) fn disable_bot_ai(&mut self) {
        if self.bot_ai.take().is_some() {
            self.rename_bots();
            self.bump_revision();
        }
    }

    pub(super) fn bot_name(&self, ordinal: u8) -> String {
        format!(
            "{}{ordinal}",
            if self.bot_ai.is_some() {
                "绝悟人机"
            } else {
                "机器人"
            }
        )
    }

    fn rename_bots(&mut self) {
        let prefix = if self.bot_ai.is_some() {
            "绝悟人机"
        } else {
            "机器人"
        };
        for player in self.players.iter_mut().filter(|p| p.is_bot) {
            let ordinal = player
                .name
                .strip_prefix("绝悟人机")
                .or_else(|| player.name.strip_prefix("机器人"))
                .unwrap_or("1");
            player.name = format!("{prefix}{ordinal}");
        }
    }
}

fn supports(model: ModelId, rules: GameRules, players: usize) -> bool {
    match (model, rules) {
        (ModelId::Douzero, GameRules::Doudizhu(rules)) => rules.mode == DoudizhuMode::Three,
        (ModelId::Kdens3, GameRules::Mahjong(_))
        | (ModelId::GuandanActor, GameRules::Guandan(_)) => true,
        (ModelId::BlackjackTransformer, GameRules::Blackjack(_)) => true,
        (ModelId::TexasPpo, GameRules::Texas(rules)) => {
            players == 6 && TexasPpoBot::supports_rules(&rules)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::supports;
    use leocard_ai::ModelId;
    use leocard_protocol::GameRules;
    use leocard_texas_holdem::TexasRuleSet;

    #[test]
    fn texas_model_requires_six_players_and_standard_rules() {
        let rules = TexasRuleSet::default();
        for count in 3..=8 {
            assert_eq!(
                supports(ModelId::TexasPpo, GameRules::Texas(rules), count),
                count == 6
            );
        }
        for rules in [
            TexasRuleSet {
                short_deck: true,
                ..rules
            },
            TexasRuleSet {
                ignore_kickers: true,
                ..rules
            },
            TexasRuleSet {
                omaha: true,
                ..rules
            },
            TexasRuleSet {
                fixed_limit: true,
                ..rules
            },
            TexasRuleSet {
                squid_markers: 1,
                ..rules
            },
            TexasRuleSet {
                side_pots: false,
                ..rules
            },
            TexasRuleSet {
                run_it_twice: true,
                ..rules
            },
        ] {
            assert!(!supports(ModelId::TexasPpo, GameRules::Texas(rules), 6));
        }
    }
}
