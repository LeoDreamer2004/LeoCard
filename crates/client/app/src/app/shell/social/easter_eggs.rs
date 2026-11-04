use leocard_game_common::PlayerPersona;
use leocard_protocol::PlayerInteractionKind;

pub(super) fn interaction_notice(
    kind: PlayerInteractionKind,
    sender: &str,
    recipient: &str,
) -> Option<String> {
    (kind == PlayerInteractionKind::Wine
        && PlayerPersona::from(recipient) == PlayerPersona::Yang120)
        .then(|| format!("{sender}又给官儿送了个酒杯，警钟长鸣！"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wine_to_yang_persona_announces_the_actual_sender() {
        assert_eq!(
            interaction_notice(PlayerInteractionKind::Wine, "送礼玩家", "Yang120").as_deref(),
            Some("送礼玩家又给官儿送了个酒杯，警钟长鸣！")
        );
        assert!(interaction_notice(PlayerInteractionKind::Wine, "Yang120", "普通玩家").is_none());
        assert!(interaction_notice(PlayerInteractionKind::Flower, "送礼玩家", "Yang120").is_none());
    }
}
