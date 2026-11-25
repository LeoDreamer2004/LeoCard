use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DevicePreference {
    #[default]
    Auto,
    Cpu,
    Gpu,
}

impl DevicePreference {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Auto => "自动",
            Self::Cpu => "CPU",
            Self::Gpu => "GPU",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Auto => Self::Cpu,
            Self::Cpu => Self::Gpu,
            Self::Gpu => Self::Auto,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ModelId {
    Douzero,
    Kdens3,
    GuandanActor,
}

impl ModelId {
    pub const ALL: [Self; 3] = [Self::Douzero, Self::Kdens3, Self::GuandanActor];
}
