use leocard_qigui523::{QiGuiSuitComparison, QiGuiTimeControl};

pub(crate) fn suit_comparison_label(comparison: QiGuiSuitComparison) -> &'static str {
    match comparison {
        QiGuiSuitComparison::HighestCard => "极大法",
        QiGuiSuitComparison::Lexicographic => "逐项法",
        QiGuiSuitComparison::SumPoints => "记点法",
    }
}

pub(crate) fn time_control_label(control: QiGuiTimeControl) -> &'static str {
    match control {
        QiGuiTimeControl::Unlimited => "不限时",
        QiGuiTimeControl::FivePlusTen => "5 + 10",
        QiGuiTimeControl::FivePlusThirty => "5 + 30",
        QiGuiTimeControl::FifteenPlusThirty => "15 + 30",
        QiGuiTimeControl::ThirtyPlusSixty => "30 + 60",
    }
}

pub(crate) fn previous_time_control(control: QiGuiTimeControl) -> Option<QiGuiTimeControl> {
    match control {
        QiGuiTimeControl::FivePlusTen => None,
        QiGuiTimeControl::FivePlusThirty => Some(QiGuiTimeControl::FivePlusTen),
        QiGuiTimeControl::FifteenPlusThirty => Some(QiGuiTimeControl::FivePlusThirty),
        QiGuiTimeControl::ThirtyPlusSixty => Some(QiGuiTimeControl::FifteenPlusThirty),
        QiGuiTimeControl::Unlimited => Some(QiGuiTimeControl::ThirtyPlusSixty),
    }
}

pub(crate) fn next_time_control(control: QiGuiTimeControl) -> Option<QiGuiTimeControl> {
    match control {
        QiGuiTimeControl::FivePlusTen => Some(QiGuiTimeControl::FivePlusThirty),
        QiGuiTimeControl::FivePlusThirty => Some(QiGuiTimeControl::FifteenPlusThirty),
        QiGuiTimeControl::FifteenPlusThirty => Some(QiGuiTimeControl::ThirtyPlusSixty),
        QiGuiTimeControl::ThirtyPlusSixty => Some(QiGuiTimeControl::Unlimited),
        QiGuiTimeControl::Unlimited => None,
    }
}

pub(crate) fn previous_suit_comparison(comparison: QiGuiSuitComparison) -> QiGuiSuitComparison {
    match comparison {
        QiGuiSuitComparison::HighestCard => QiGuiSuitComparison::SumPoints,
        QiGuiSuitComparison::Lexicographic => QiGuiSuitComparison::HighestCard,
        QiGuiSuitComparison::SumPoints => QiGuiSuitComparison::Lexicographic,
    }
}

pub(crate) fn next_suit_comparison(comparison: QiGuiSuitComparison) -> QiGuiSuitComparison {
    match comparison {
        QiGuiSuitComparison::HighestCard => QiGuiSuitComparison::Lexicographic,
        QiGuiSuitComparison::Lexicographic => QiGuiSuitComparison::SumPoints,
        QiGuiSuitComparison::SumPoints => QiGuiSuitComparison::HighestCard,
    }
}
