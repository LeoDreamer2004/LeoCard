use leocard_protocol::GameKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ItemId {
    ObservationLens,
    ShengjiCardCounter,
    ShengjiMissingSuitCard,
    QiGui523CardCounter,
}

pub struct ShopItem {
    pub id: ItemId,
    pub game: GameKind,
    pub name: &'static str,
    pub description: &'static str,
    pub price: u32,
    pub duration_seconds: u64,
}

impl ItemId {
    pub const ALL: [Self; 4] = [
        Self::ObservationLens,
        Self::ShengjiCardCounter,
        Self::ShengjiMissingSuitCard,
        Self::QiGui523CardCounter,
    ];

    pub const fn definition(self) -> ShopItem {
        match self {
            Self::QiGui523CardCounter => ShopItem {
                id: self,
                game: GameKind::QiGui523,
                name: "记牌器",
                description: "按黑、红、梅、方记录除自己手牌和已出牌外的剩余牌，标亮分牌数量。",
                price: 40,
                duration_seconds: 3600,
            },
            Self::ObservationLens => ShopItem {
                id: self,
                game: GameKind::TexasHoldem,
                name: "观局镜",
                description: "弃牌后，查看仍在对局的玩家胜率。\n仅在弃牌所在下注轮结束后可用，不显示其他玩家的底牌。",
                price: 20,
                duration_seconds: 3600,
            },
            Self::ShengjiMissingSuitCard => ShopItem {
                id: self,
                game: GameKind::Shengji,
                name: "缺门卡",
                description: "根据公开跟牌，标记其他玩家已经缺少的副牌花色或主牌。",
                price: 30,
                duration_seconds: 3600,
            },
            Self::ShengjiCardCounter => ShopItem {
                id: self,
                game: GameKind::Shengji,
                name: "记牌器",
                description: "记录除自己手牌、已出牌和自己最终埋底以外的剩余牌。",
                price: 60,
                duration_seconds: 3600,
            },
        }
    }
}
