# Roff-bot
This is a bot for a [rust-offtopic chat](https://t.me/rust_offtopic) in the Telegram.

## Bot usage
This section describes features available in the bot.

### Structural warnings
Structural warnings is a flexible system of warnings allow you to describe conditions upon which user must be warned and punishments.

Warnings are divided into a groups, these groups do not intersect each other. You can set up any group any time.

Under a group there are score point panel. When score point panel is full, punishment follows.

TODO: add time limit for warnings.
TODO: progressive time for recidivists.

#### Punishments
There are few types of punishments:
1. Ban. User was removed from the chat for a time or forever.
2. Mute. User cannot send messages for a time or forever.
3. Restrict. User cannot send specific kinds of messages for a time or forever.

#### Set up warning group
TODO
