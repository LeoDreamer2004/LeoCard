use super::{McGuessAttribute, McGuessMatchQuality};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// The direction in which the guess must change to reach the answer.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum McGuessValueDirection {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessAttributeFeedback {
    pub quality: McGuessMatchQuality,
    pub direction: Option<McGuessValueDirection>,
}

impl McGuessAttribute {
    pub fn feedback(&self, answer: &Self) -> McGuessAttributeFeedback {
        let quality = self.compare(answer);
        let direction = (quality == McGuessMatchQuality::Wrong)
            .then(|| self.ordered_comparison(answer))
            .flatten()
            .and_then(|order| match order {
                Ordering::Less => Some(McGuessValueDirection::Up),
                Ordering::Greater => Some(McGuessValueDirection::Down),
                Ordering::Equal => None,
            });
        McGuessAttributeFeedback { quality, direction }
    }

    fn ordered_comparison(&self, answer: &Self) -> Option<Ordering> {
        if self.label != answer.label || self.values.len() != 1 || answer.values.len() != 1 {
            return None;
        }
        let ours = &self.values[0];
        let theirs = &answer.values[0];
        match self.label.as_str() {
            "硬度" | "爆炸抗性" | "堆叠数量" | "最大生命" | "高度" | "最高等级" | "温度"
            | "降水" => {
                let number = |text: &str| {
                    let value = text.parse::<f64>().ok()?;
                    // Negative hardness is the unbreakable sentinel, not a softer block.
                    Some(if self.label == "硬度" && value < 0.0 {
                        f64::INFINITY
                    } else {
                        value
                    })
                };
                number(ours)?.partial_cmp(&number(theirs)?)
            }
            "首次加入版本" => Some(Version::parse(ours)?.cmp(&Version::parse(theirs)?)),
            "稀有度" => {
                let levels = ["普通", "罕见", "稀有", "史诗"];
                let rank = |text: &str| levels.iter().position(|level| *level == text);
                Some(rank(ours)?.cmp(&rank(theirs)?))
            }
            _ => None,
        }
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum Version {
    PreClassic,
    Release(Vec<u32>),
}

impl Version {
    fn parse(text: &str) -> Option<Self> {
        if text == "pre-classic" {
            return Some(Self::PreClassic);
        }
        let mut parts = text
            .split('.')
            .map(str::parse)
            .collect::<Result<Vec<u32>, _>>()
            .ok()?;
        while parts.last() == Some(&0) {
            parts.pop();
        }
        Some(Self::Release(parts))
    }
}
