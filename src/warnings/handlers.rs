use crate::{
    utils,
    warnings::{
        commands::WarnsCommand,
        dto::{OnWarnAction, Punishment, PunishmentKind, PunishmentTime, UserWarning, WarningInfo},
        repository::WarnsRepository,
    },
    HandlerOut, TBot, BOT_NAME,
};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use teloxide::{
    prelude2::*,
    types::{ChatPermissions, User},
};

pub fn setup_warnings_handler() -> crate::Handler {
    dptree::entry().branch(
        utils::filter_chat_owner()
            .add_command::<WarnsCommand>(BOT_NAME.into())
            .endpoint(handle_warns_commands),
    )
}

async fn handle_warns_commands(
    bot: TBot,
    mes: Message,
    cmd: WarnsCommand,
    repo: WarnsRepository,
) -> HandlerOut {
    match cmd {
        WarnsCommand::Warn { trigger } => {
            let reply_to_message = match mes.reply_to_message() {
                Some(mes) => mes.clone(),
                None => {
                    bot.send_message(mes.chat.id, "Reply to a user message to warn.").await?;
                    return Ok(());
                }
            };
            let reply_to = match reply_to_message.from() {
                Some(user) => user.clone(),
                None => {
                    bot.send_message(mes.chat.id, "Reply to a user message to warn.").await?;
                    return Ok(());
                }
            };
            let warn_info = repo.find_warn_by_trigger(trigger.as_str()).await?;
            match warn_info {
                Some(warn) => {
                    warn_user(bot.clone(), mes, repo, reply_to, &warn).await?;
                    match warn.on_warn {
                        OnWarnAction::DeleteMessage => {
                            bot.delete_message(reply_to_message.chat.id, reply_to_message.id)
                                .await?;
                        }
                        OnWarnAction::Nothing => {}
                    }
                }
                None => {
                    bot.send_message(mes.chat.id, "There are no such warning type.").await?;
                }
            }
        }
    }

    Ok(())
}

async fn warn_user(
    bot: TBot,
    mes: Message,
    repo: WarnsRepository,
    user: User,
    warn: &WarningInfo,
) -> HandlerOut {
    let points = repo.get_user_warn_points(user.id, &warn.group).await?;
    let new_points = points + warn.points;
    if new_points >= warn.group.max_points {
        punish_user(bot.clone(), &mes, user.id, &warn.group.punishment).await?;
        repo.remove_actual_warns(user.id, &warn.group).await?;
        bot.send_message(mes.chat.id, message_user_punished(&user, &warn.group.punishment)).await?;
    } else {
        let text = format!(
            "{} has been warned! {}/{} points.",
            user.full_name(),
            new_points,
            warn.group.max_points
        );
        repo.insert_warn(UserWarning { user_id: user.id, info: warn.clone() }).await?;
        bot.send_message(mes.chat.id, text).await?;
    }

    Ok(())
}

async fn punish_user(
    bot: TBot,
    mes: &Message,
    user_id: i64,
    punishment: &Punishment,
) -> HandlerOut {
    let mes_time =
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(mes.date as i64, 0), Utc);
    let punishment_time = match punishment.time {
        PunishmentTime::Time(d) => Duration::seconds(d as i64),
        // If we restrict user until `now` time, telegram will restrict user forever.
        PunishmentTime::Forever => Duration::seconds(0),
    };
    let until_time = mes_time + punishment_time;

    match punishment.kind {
        PunishmentKind::Ban => {
            bot.ban_chat_member(mes.chat.id, user_id)
                .until_date(until_time.timestamp() as u64)
                .await?;
        }
        PunishmentKind::Mute => {
            bot.restrict_chat_member(mes.chat.id, user_id, ChatPermissions::default())
                .until_date(until_time)
                .await?;
        }
        PunishmentKind::Restrict(perms) => {
            bot.restrict_chat_member(mes.chat.id, user_id, perms).until_date(until_time).await?;
        }
    }

    Ok(())
}

fn message_user_punished(user: &User, punishment: &Punishment) -> String {
    let time = match punishment.time {
        PunishmentTime::Time(s) => format!("for a {} seconds.", s),
        PunishmentTime::Forever => format!("forever!"),
    };
    match punishment.kind {
        PunishmentKind::Mute => {
            format!("User {} has been muted {}", user.full_name(), time)
        }
        PunishmentKind::Ban => {
            format!("User {} has been baned {}", user.full_name(), time)
        }
        PunishmentKind::Restrict(_) => {
            format!("User {} has been restricted {}", user.full_name(), time)
        }
    }
}
