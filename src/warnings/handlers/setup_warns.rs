use crate::{
    warnings::{
        commands::SetupWarnsCommands,
        dto::{OnWarnAction, WarningGroup, WarningInfo},
        handlers::WarnsStorage,
    },
    HandlerOut, TBot, WarnsRepository,
};
use serde::{Deserialize, Serialize};
use teloxide::{
    macros::DialogueState,
    prelude2::*,
    types::{InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup},
};

pub async fn handle_setup_warns_commands(
    bot: TBot,
    mes: Message,
    dialogue: Dialogue,
    cmd: SetupWarnsCommands,
) -> HandlerOut {
    match cmd {
        SetupWarnsCommands::NewWarn { chat_id } => {
            match dialogue.current_state().await? {
                Some(_) => {
                    bot.send_message(mes.chat.id, "You already setup new warn type.").await?;
                    return Ok(());
                }
                None => {}
            };
            dialogue.next(SetupWarnState::WaitForWarnGroup(chat_id)).await?;
            bot.send_message(
                mes.chat.id,
                "Good. Send me the name of the warn group the warn must relate to.",
            )
            .await?;
        }
        SetupWarnsCommands::Cancel => {
            dialogue.exit().await?;
            bot.send_message(mes.chat.id, "Cancelled.").await?;
        }
    }

    Ok(())
}

#[derive(DialogueState, Clone, Serialize, Deserialize)]
#[handler_out(anyhow::Result<()>)]
pub enum SetupWarnState {
    #[handler(wait_for_warn_group_handler)]
    WaitForWarnGroup(i64),

    #[handler(wait_for_points_handler)]
    WaitForPoints(WaitForPointsState),

    #[handler(wait_for_trigger_handler)]
    WaitForTrigger(WaitForTriggerState),

    #[handler(wait_for_on_warn_handler)]
    WaitForOnWarn(WaitForOnWarnState),
}

impl Default for SetupWarnState {
    fn default() -> Self {
        Self::WaitForWarnGroup(0)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WaitForPointsState {
    chat_id: i64,
    group: WarningGroup,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WaitForTriggerState {
    chat_id: i64,
    group: WarningGroup,
    max_points: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WaitForOnWarnState {
    chat_id: i64,
    group: WarningGroup,
    max_points: u64,
    trigger: String,
}

type Dialogue = teloxide::dispatching2::dialogue::Dialogue<SetupWarnState, WarnsStorage>;

async fn wait_for_warn_group_handler(
    bot: TBot,
    mes: Message,
    dialogue: Dialogue,
    repo: WarnsRepository,
    chat_id: i64,
) -> HandlerOut {
    let text = match mes.text() {
        Some(text) => text,
        None => return Ok(()),
    };
    let group = repo.get_warn_group(text).await?;
    let group = match group {
        Some(g) => g,
        None => {
            bot.send_message(
                mes.chat.id,
                "There are no such warn group. Send me the name of the warn group the warn must \
                 relate to.",
            )
            .await?;
            return Ok(());
        }
    };
    dialogue.next(SetupWarnState::WaitForPoints(WaitForPointsState { chat_id, group })).await?;
    bot.send_message(
        mes.chat.id,
        "Good. Now send me amount of the points the user will receive by this warn.",
    )
    .await?;

    Ok(())
}

async fn wait_for_points_handler(
    bot: TBot,
    mes: Message,
    dialogue: Dialogue,
    state: WaitForPointsState,
) -> HandlerOut {
    let text = match mes.text() {
        Some(text) => text,
        None => return Ok(()),
    };
    let max_points = match text.parse::<u64>() {
        Ok(p) => p,
        Err(_) => {
            bot.send_message(
                mes.chat.id,
                "Send me the maximum points the user receive before they gets punished.",
            )
            .await?;
            return Ok(());
        }
    };
    dialogue
        .next(SetupWarnState::WaitForTrigger(WaitForTriggerState {
            chat_id: state.chat_id,
            group: state.group,
            max_points,
        }))
        .await?;

    bot.send_message(
        mes.chat.id,
        "Good. Now send me a text trigger for the warn. It then will be used by `/warn <trigger>` \
         format.",
    )
    .await?;

    Ok(())
}

async fn wait_for_trigger_handler(
    bot: TBot,
    mes: Message,
    dialogue: Dialogue,
    repo: WarnsRepository,
    state: WaitForTriggerState,
) -> HandlerOut {
    let trigger = match mes.text() {
        Some(t) => t.to_string(),
        None => return Ok(()),
    };
    match repo.find_warn_by_trigger(&trigger).await? {
        Some(_) => {
            bot.send_message(mes.chat.id, "Warn with such trigger already exists.").await?;
            return Ok(());
        }
        _ => {}
    };

    dialogue
        .next(SetupWarnState::WaitForOnWarn(WaitForOnWarnState {
            chat_id: state.chat_id,
            group: state.group,
            max_points: state.max_points,
            trigger,
        }))
        .await?;
    let kb = {
        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::new(
                "Delete message",
                InlineKeyboardButtonKind::CallbackData("delete".into()),
            )],
            [InlineKeyboardButton::new(
                "Do nothing",
                InlineKeyboardButtonKind::CallbackData("nothing".into()),
            )],
        ])
    };
    bot.send_message(
        mes.chat.id,
        "Good. Do you want to delete the message you reply to when warning?",
    )
    .reply_markup(kb)
    .await?;

    Ok(())
}

async fn wait_for_on_warn_handler(bot: TBot, mes: Message) -> HandlerOut {
    bot.send_message(mes.chat.id, "Please, use one of the buttons above.").await?;
    Ok(())
}

pub async fn wait_for_on_warn_callback_query_handler(
    bot: TBot,
    q: CallbackQuery,
    d: Dialogue,
    repo: WarnsRepository,
) -> HandlerOut {
    let state = match d.current_state().await? {
        Some(SetupWarnState::WaitForOnWarn(x)) => x,
        _ => return Ok(()),
    };
    let on_warn = match q.data.as_ref().map(|x| x.as_str()) {
        Some("delete") => OnWarnAction::DeleteMessage,
        Some("nothing") => OnWarnAction::Nothing,
        Some(other) => {
            log::warn!("Unexpected data: {}", other);
            return Ok(());
        }
        None => {
            // I really do not know when `q.data` can be `None`.
            log::warn!("Data is none: {:?}", &q);
            return Ok(());
        }
    };
    let text =
        format!("You have added new warn type. To use it use /warn {} command", &state.trigger);
    d.exit().await?;
    repo.insert_warn_type(WarningInfo {
        trigger: state.trigger,
        points: state.max_points,
        group: state.group,
        on_warn,
    })
    .await?;

    bot.answer_callback_query(q.id).await?;
    if let Some(mes) = &q.message {
        bot.edit_message_text(mes.chat.id, mes.id, "Selected.").await?;
        bot.send_message(mes.chat.id, text).await?;
    }

    Ok(())
}
