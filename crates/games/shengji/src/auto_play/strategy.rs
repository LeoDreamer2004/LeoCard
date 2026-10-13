use super::error::ShengjiAutoPlayError;
use super::helpers::{card_cmp, choose_short_category_fill, choose_structured_follow};
use crate::play::{category, classify_cards};
use crate::{
    ShengjiCard, ShengjiClassifiedPlay, ShengjiTrump, follow_suggestions, validate_follow,
};

/// 双升最小贪心托管策略作出一次出牌决定所需的信息。
///
/// `lead` 为 `None` 时托管策略领出最小单张；否则严格履行跟门、对子和拖拉机义务。
#[derive(Clone, Copy, Debug)]
pub struct ShengjiAutoPlayRequest<'a> {
    pub hand: &'a [ShengjiCard],
    pub lead: Option<&'a ShengjiClassifiedPlay>,
    pub trump: ShengjiTrump,
}

/// 确定性的最小贪心策略，不保存对局状态，也不读取其他玩家手牌。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ShengjiAutoPlay;

impl ShengjiAutoPlay {
    pub fn choose(
        request: ShengjiAutoPlayRequest<'_>,
    ) -> Result<ShengjiClassifiedPlay, ShengjiAutoPlayError> {
        let Some(lead) = request.lead else {
            let card = request
                .hand
                .iter()
                .copied()
                .min_by(|left, right| card_cmp(*left, *right, request.trump))
                .ok_or(ShengjiAutoPlayError::EmptyHand)?;
            return classify_cards(&[card], request.trump)
                .map_err(ShengjiAutoPlayError::InvalidLead);
        };

        let required = lead.cards.len();
        if request.hand.len() < required {
            return Err(ShengjiAutoPlayError::NotEnoughCards {
                required,
                actual: request.hand.len(),
            });
        }
        if let Some(play) = follow_suggestions(request.hand, lead, request.trump, 1)
            .into_iter()
            .next()
        {
            return Ok(play);
        }
        let lead_category = lead.category;
        let mut same_category = request
            .hand
            .iter()
            .copied()
            .filter(|card| category(*card, request.trump) == lead_category)
            .collect::<Vec<_>>();
        same_category.sort_by(|left, right| card_cmp(*left, *right, request.trump));

        let cards = if same_category.len() < required {
            choose_short_category_fill(request.hand, &same_category, required, request.trump)
        } else {
            choose_structured_follow(&same_category, lead, request.trump)
        };
        validate_follow(request.hand, &cards, lead, request.trump)
            .map_err(ShengjiAutoPlayError::InvalidFollow)
    }
}
