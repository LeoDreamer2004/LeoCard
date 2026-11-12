use super::catalog::{MAJOR_FAN_ART, MajorFanArtSpec};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use leocard_mahjong::{MahjongFan, MahjongWinRecord};

pub(super) struct MajorFanArt {
    pub spec: &'static MajorFanArtSpec,
    pub background: Handle<Image>,
    pub title: Handle<Image>,
}

#[derive(Resource, Default)]
pub(crate) struct MahjongMajorFanAssets {
    active: Vec<MajorFanArt>,
}

impl MahjongMajorFanAssets {
    pub(crate) fn prepare(
        &mut self,
        server: &AssetServer,
        fans: impl IntoIterator<Item = MahjongFan>,
    ) {
        let fans = fans.into_iter().collect::<Vec<_>>();
        self.active.retain(|art| fans.contains(&art.spec.fan));
        for fan in fans {
            if self.active.iter().any(|art| art.spec.fan == fan) {
                continue;
            }
            let spec = MAJOR_FAN_ART
                .iter()
                .find(|spec| spec.fan == fan)
                .expect("every fan worth at least 48 points has cinematic artwork");
            let load = |name| {
                server
                    .load_builder()
                    .with_settings(|settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::linear();
                    })
                    .load(format!("ui/mahjong/major-fans/{}/{name}", spec.slug))
            };
            self.active.push(MajorFanArt {
                spec,
                background: load("background.webp"),
                title: load("title.png"),
            });
        }
    }

    pub(super) fn get(&self, fan: MahjongFan) -> &MajorFanArt {
        self.active
            .iter()
            .find(|art| art.spec.fan == fan)
            .expect("active winner artwork is prepared before composing the table")
    }
}

pub(crate) fn mahjong_major_fan(winner: &MahjongWinRecord) -> Option<MahjongFan> {
    winner
        .score
        .fans
        .iter()
        .map(|value| value.fan)
        .filter(|fan| fan.points() >= 48)
        .max_by_key(|fan| fan.points())
}
