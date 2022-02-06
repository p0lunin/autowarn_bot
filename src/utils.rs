use crate::TBot;
use teloxide::prelude2::*;

pub fn filter_chat_owner() -> crate::Handler {
    dptree::filter_async(|bot: TBot, mes: Message| async move {
        let user = match mes.from() {
            Some(u) => u,
            None => return false,
        };
        let chat_member = match bot.get_chat_member(mes.chat.id, user.id).await {
            Ok(resp) => resp,
            _ => return false,
        };
        chat_member.kind.is_owner()
    })
}
