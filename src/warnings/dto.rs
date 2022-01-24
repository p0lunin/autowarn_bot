use serde::{Deserialize, Serialize};
use teloxide::types::ChatPermissions;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UserWarning {
    pub user_id: i64,
    pub info: WarningInfo,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WarningInfo {
    pub trigger: String,
    pub points: u64,
    pub group: WarningGroup,
    pub on_warn: OnWarnAction,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum OnWarnAction {
    DeleteMessage,
    Nothing,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WarningGroup {
    pub name: String,
    pub max_points: u64,
    pub punishment: Punishment,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Punishment {
    pub time: PunishmentTime,
    pub kind: PunishmentKind,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum PunishmentTime {
    // Seconds.
    Time(u64),
    Forever,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum PunishmentKind {
    Ban,
    Mute,
    Restrict(ChatPermissions),
}
