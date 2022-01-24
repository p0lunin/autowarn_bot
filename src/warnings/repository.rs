use crate::warnings::dto::{
    OnWarnAction, Punishment, PunishmentKind, PunishmentTime, UserWarning, WarningGroup,
    WarningInfo,
};
use mongodb::{
    bson::{doc, Document},
    options::UpdateOptions,
    Collection, Database,
};
use teloxide::prelude::StreamExt;

#[derive(Debug, Clone)]
pub struct WarnsRepository {
    old_warns: Collection<UserWarning>,
    actual_warns: Collection<UserWarning>,
    warning_types: Collection<WarningInfo>,
    warning_groups: Collection<WarningGroup>,
}

impl WarnsRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            old_warns: db.collection("old_warns"),
            actual_warns: db.collection("actual_warns"),
            warning_types: db.collection("warning_types"),
            warning_groups: db.collection("warning_groups"),
        }
    }

    pub async fn insert_default_values(&self) -> Result<(), mongodb::error::Error> {
        let mut options = UpdateOptions::default();
        options.upsert = Some(true);

        let carizm = doc! {
            "name": "царизм",
            "max_points": 100,
            "punishment": {
                "time": mongodb::bson::to_bson(&PunishmentTime::Forever).unwrap(),
                "kind": mongodb::bson::to_bson(&PunishmentKind::Mute).unwrap(),
            },
        };

        self.warning_groups
            .update_one(doc! { "name": "царизм" }, doc! { "$set": carizm.clone() }, options.clone())
            .await?;

        self.warning_types
            .update_one(
                doc! { "trigger": "макака" },
                doc! {
                    "$set": {
                        "trigger": "макака",
                        "points": 30,
                        "group": carizm.clone(),
                        "on_warn": mongodb::bson::to_bson(&OnWarnAction::DeleteMessage).unwrap(),
                    }
                },
                options.clone(),
            )
            .await?;

        Ok(())
    }

    pub async fn insert_warn(&self, warn: UserWarning) -> Result<(), mongodb::error::Error> {
        self.actual_warns.insert_one(warn, None).await.map(|_| ())
    }

    pub async fn find_warn_by_trigger(
        &self,
        trigger: &str,
    ) -> Result<Option<WarningInfo>, mongodb::error::Error> {
        let warn = self.warning_types.find_one(doc! { "trigger" : trigger }, None).await?;
        let warn = warn.and_then(|w| if w.trigger == trigger { Some(w) } else { None });
        Ok(warn)
    }

    pub async fn get_user_warn_points(
        &self,
        user_id: i64,
        group: &WarningGroup,
    ) -> Result<u64, mongodb::error::Error> {
        let sum = self
            .get_actual_warns_by_group(user_id, &group.name)
            .await?
            .into_iter()
            .map(|x| x.info.points)
            .sum::<u64>();
        Ok(sum)
    }

    pub async fn remove_actual_warns(
        &self,
        user_id: i64,
        group: &WarningGroup,
    ) -> Result<(), mongodb::error::Error> {
        // TODO: transactions.
        let warns = self.get_actual_warns_by_group(user_id, &group.name).await?;
        self.actual_warns
            .delete_many(doc! { "info.group.name": &group.name, "user_id": user_id }, None)
            .await?;
        for warn in warns {
            self.old_warns.insert_one(warn, None).await?;
        }
        Ok(())
    }

    pub async fn get_actual_warns(
        &self,
        user_id: i64,
    ) -> Result<Vec<UserWarning>, mongodb::error::Error> {
        self.actual_warns
            .find(doc! { "user_id": user_id }, None)
            .await?
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_actual_warns_by_group(
        &self,
        user_id: i64,
        group_name: &str,
    ) -> Result<Vec<UserWarning>, mongodb::error::Error> {
        self.actual_warns
            .find(doc! { "info.group.name": group_name, "user_id": user_id }, None)
            .await?
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
    }
}
