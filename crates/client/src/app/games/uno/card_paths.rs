//! UNO 各扩展牌面的素材路径。

use leocard_uno::{UnoCard, UnoColor, UnoFace};

pub(crate) fn uno_card_asset_path(card: UnoCard) -> String {
    match card.color() {
        None => match card.face() {
            UnoFace::Wild => "cards/uno/wild.png".to_owned(),
            UnoFace::DarkWild => "cards/uno-extension/uno-flip/dark/wild.png".to_owned(),
            UnoFace::WildDrawTwo => {
                "cards/uno-extension/uno-flip/light/wild_draw_two.png".to_owned()
            }
            UnoFace::WildDrawColor => {
                "cards/uno-extension/uno-flip/dark/wild_draw_color.png".to_owned()
            }
            UnoFace::WildDrawFour => "cards/uno/wild_draw_four.png".to_owned(),
            UnoFace::WildForceTrade => {
                "cards/uno-extension/swap-pack/wild_force_trade.png".to_owned()
            }
            UnoFace::WildPassHands => {
                "cards/uno-extension/swap-pack/wild_pass_hands.png".to_owned()
            }
            UnoFace::WildPowerReverse => {
                "cards/uno-extension/reverse-pack/wild_power_reverse.png".to_owned()
            }
            UnoFace::WildNoU => "cards/uno-extension/reverse-pack/wild_no_u.png".to_owned(),
            UnoFace::WildStackThree => {
                "cards/uno-extension/stack-pack/wild_stack_three.png".to_owned()
            }
            UnoFace::WildStackNumber => {
                "cards/uno-extension/stack-pack/wild_stack_number.png".to_owned()
            }
            UnoFace::WildReverseDrawFour => {
                "cards/uno-extension/no-mercy/wild_reverse_draw_four.png".to_owned()
            }
            UnoFace::WildDrawSix => "cards/uno-extension/no-mercy/wild_draw_six.png".to_owned(),
            UnoFace::WildDrawTen => "cards/uno-extension/no-mercy/wild_draw_ten.png".to_owned(),
            UnoFace::WildColorRoulette => {
                "cards/uno-extension/no-mercy/wild_color_roulette.png".to_owned()
            }
            _ => unreachable!("无颜色的 UNO 牌必须是万能牌"),
        },
        Some(color) => {
            let color = match color {
                UnoColor::Red => "red",
                UnoColor::Yellow => "yellow",
                UnoColor::Green => "green",
                UnoColor::Blue => "blue",
                UnoColor::Pink => "pink",
                UnoColor::Teal => "teal",
                UnoColor::Orange => "orange",
                UnoColor::Purple => "purple",
            };
            let face = match card.face() {
                UnoFace::Number(value) => value.to_string(),
                UnoFace::DrawOne => "draw_one".to_owned(),
                UnoFace::DrawTwo => "draw_two".to_owned(),
                UnoFace::DrawFive => "draw_five".to_owned(),
                UnoFace::Reverse => "reverse".to_owned(),
                UnoFace::Skip => "skip".to_owned(),
                UnoFace::Flip => "flip".to_owned(),
                UnoFace::DrawFour => {
                    return format!("cards/uno-extension/no-mercy/{color}_draw_four.png");
                }
                UnoFace::SkipEveryone => {
                    if matches!(
                        card.color(),
                        Some(UnoColor::Pink | UnoColor::Teal | UnoColor::Orange | UnoColor::Purple)
                    ) {
                        "skip_everyone".to_owned()
                    } else {
                        return format!("cards/uno-extension/no-mercy/{color}_skip_everyone.png");
                    }
                }
                UnoFace::DiscardAll => {
                    return format!("cards/uno-extension/no-mercy/{color}_discard_all.png");
                }
                UnoFace::SwapOne => {
                    return format!("cards/uno-extension/swap-pack/{color}_swap_one.png");
                }
                UnoFace::RefreshHand => {
                    return format!("cards/uno-extension/swap-pack/{color}_refresh_hand.png");
                }
                UnoFace::ReverseDrawTwo => {
                    return format!(
                        "cards/uno-extension/reverse-pack/{color}_reverse_draw_two.png"
                    );
                }
                UnoFace::ReverseSkip => {
                    return format!("cards/uno-extension/reverse-pack/{color}_reverse_skip.png");
                }
                UnoFace::StackOne => {
                    return format!("cards/uno-extension/stack-pack/{color}_stack_one.png");
                }
                UnoFace::StackTwo => {
                    return format!("cards/uno-extension/stack-pack/{color}_stack_two.png");
                }
                UnoFace::Wild
                | UnoFace::DarkWild
                | UnoFace::WildDrawTwo
                | UnoFace::WildDrawFour
                | UnoFace::WildDrawColor
                | UnoFace::WildForceTrade
                | UnoFace::WildPassHands
                | UnoFace::WildPowerReverse
                | UnoFace::WildNoU
                | UnoFace::WildStackThree
                | UnoFace::WildStackNumber
                | UnoFace::WildReverseDrawFour
                | UnoFace::WildDrawSix
                | UnoFace::WildDrawTen
                | UnoFace::WildColorRoulette => unreachable!("万能牌没有颜色"),
            };
            if color == "pink" || color == "teal" || color == "orange" || color == "purple" {
                format!("cards/uno-extension/uno-flip/dark/{color}_{face}.png")
            } else if matches!(card.face(), UnoFace::DrawOne | UnoFace::Flip) {
                format!("cards/uno-extension/uno-flip/light/{color}_{face}.png")
            } else {
                format!("cards/uno/{color}_{face}.png")
            }
        }
    }
}
