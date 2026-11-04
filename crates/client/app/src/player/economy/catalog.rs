use self::ShopRarity::{Common, Epic, Fine};
use leocard_game_common::PlayerPersona;
use leocard_protocol::{AvatarFrameId, GameKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum PropId {
    ObservationLens,
    ShengjiCardCounter,
    ShengjiMissingSuitCard,
    QiGui523CardCounter,
    UnoJumpInDevice,
    MahjongFanCalculator,
    DoudizhuCardCounter,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ShopRarity {
    Common,
    Fine,
    Epic,
}

pub struct ShopProp {
    pub id: PropId,
    pub game: GameKind,
    pub name: &'static str,
    pub description: &'static str,
    pub rarity: ShopRarity,
    pub price: u32,
    pub duration_seconds: u64,
}

#[derive(Clone, Copy)]
pub struct ShopAvatarFrame {
    pub id: AvatarFrameId,
    pub name: &'static str,
    pub asset: &'static str,
    pub rarity: ShopRarity,
    pub price: u32,
}

impl ShopAvatarFrame {
    pub const ALL: [Self; 12] = [
        Self::new(AvatarFrameId::Default, "胡桃木", "walnut", Common, 0),
        Self::new(AvatarFrameId::QqAdmin, "大官", "qq-admin", Epic, 10_000),
        Self::new(AvatarFrameId::IronBasin, "铁盆", "iron-basin", Fine, 2_000),
        Self::new(AvatarFrameId::Orange, "橘子", "orange", Fine, 2_000),
        Self::new(AvatarFrameId::Horse, "马儿", "horse", Fine, 2_000),
        Self::new(AvatarFrameId::ChinaRose, "月季", "china-rose", Common, 700),
        Self::new(AvatarFrameId::Cattail, "香蒲", "pvz-cattail", Fine, 2_000),
        Self::new(AvatarFrameId::Study, "知识", "study", Common, 700),
        Self::new(AvatarFrameId::Create, "机械动力", "create", Fine, 2_000),
        Self::new(AvatarFrameId::GregTech, "格雷员工", "gregtech", Fine, 2_000),
        Self::new(AvatarFrameId::Host, "东道主", "host", Fine, 2_000),
        Self::new(
            AvatarFrameId::WhaleGirl,
            "哦鲸鲸",
            "whale-girl",
            Epic,
            10_000,
        ),
    ];

    const fn new(
        id: AvatarFrameId,
        name: &'static str,
        asset: &'static str,
        rarity: ShopRarity,
        price: u32,
    ) -> Self {
        Self {
            id,
            name,
            asset,
            rarity,
            price,
        }
    }

    pub fn definition(id: AvatarFrameId) -> Self {
        Self::ALL
            .into_iter()
            .find(|frame| frame.id == id)
            .expect("every avatar frame has a shop listing")
    }

    pub(super) fn exclusive_for_name(name: &str) -> Option<AvatarFrameId> {
        match PlayerPersona::from(name) {
            PlayerPersona::Yang120 => Some(AvatarFrameId::Orange),
            PlayerPersona::DreamerLeo => Some(AvatarFrameId::Cattail),
            PlayerPersona::Wpy03 => Some(AvatarFrameId::IronBasin),
            PlayerPersona::SnowyWind => Some(AvatarFrameId::Horse),
            PlayerPersona::Cnofne => Some(AvatarFrameId::Host),
            PlayerPersona::Regular => None,
        }
    }
}

impl PropId {
    pub const ALL: [Self; 7] = [
        Self::ObservationLens,
        Self::ShengjiCardCounter,
        Self::ShengjiMissingSuitCard,
        Self::QiGui523CardCounter,
        Self::UnoJumpInDevice,
        Self::MahjongFanCalculator,
        Self::DoudizhuCardCounter,
    ];

    pub const fn definition(self) -> ShopProp {
        match self {
            Self::DoudizhuCardCounter => ShopProp {
                id: self,
                game: GameKind::Doudizhu,
                name: "记牌器",
                description: "适用于三人和四人斗地主。记录除自己手牌和公开出牌之外的剩余牌。",
                rarity: Common,
                price: 150,
                duration_seconds: 3600,
            },
            Self::MahjongFanCalculator => ShopProp {
                id: self,
                game: GameKind::Mahjong,
                name: "算番卡",
                description: "在听牌提示中显示当前牌型自摸与和牌的番数。\n不足八番也会显示，花牌不计入八番起和门槛。",
                rarity: Common,
                price: 60,
                duration_seconds: 3600,
            },
            Self::UnoJumpInDevice => ShopProp {
                id: self,
                game: GameKind::Uno,
                name: "抢出器",
                description: "开启后，在符合抢出规则时自动出牌。\n抢出结果受网络延迟影响，有效期内可随时开关。",
                rarity: Fine,
                price: 300,
                duration_seconds: 3600,
            },
            Self::QiGui523CardCounter => ShopProp {
                id: self,
                game: GameKind::QiGui523,
                name: "记牌器",
                description: "按黑、红、梅、方记录除自己手牌和已出牌外的剩余牌，标亮分牌数量。",
                rarity: Common,
                price: 120,
                duration_seconds: 3600,
            },
            Self::ObservationLens => ShopProp {
                id: self,
                game: GameKind::Texas,
                name: "观局镜",
                description: "弃牌后，查看仍在对局的玩家胜率。\n仅在弃牌所在下注轮结束后可用，不显示其他玩家的底牌。",
                rarity: Common,
                price: 40,
                duration_seconds: 3600,
            },
            Self::ShengjiMissingSuitCard => ShopProp {
                id: self,
                game: GameKind::Shengji,
                name: "缺门卡",
                description: "根据公开跟牌，标记其他玩家已经缺少的副牌花色或主牌。",
                rarity: Common,
                price: 60,
                duration_seconds: 3600,
            },
            Self::ShengjiCardCounter => ShopProp {
                id: self,
                game: GameKind::Shengji,
                name: "记牌器",
                description: "记录除自己手牌、已出牌和自己最终埋底以外的剩余牌。",
                rarity: Common,
                price: 150,
                duration_seconds: 3600,
            },
        }
    }
}
