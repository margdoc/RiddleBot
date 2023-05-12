use lazy_static::lazy_static;
use regex::Regex;
use teloxide::{requests::Requester, types::ChatId, Bot};

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;
pub(crate) type HandlerResult<R = ()> = Result<R, Error>;

lazy_static! {
    static ref RE: Regex = Regex::new(r"([\[_\*\[\]\(\)~`>#\+-=\|{}\.!])").unwrap();
}

pub(crate) fn escape_chars<T>(text: T) -> String
where
    T: Into<String>,
{
    text.into() // TODO fix to enable markdown
                // RE.replace_all(&*text.into(), r"\\$1").to_string()
                // .replace("_", "\\_")
                // .replace("*", "\\*")
                // .replace("[", "\\[")
                // .replace("]", "\\]")
                // .replace("(", "\\(")
                // .replace(")", "\\)")
                // .replace("~", "\\~")
                // .replace("`", "\\`")
                // .replace(">", "\\>")
                // .replace("#", "\\#")
                // .replace("+", "\\+")
                // .replace("-", "\\-")
                // .replace("=", "\\=")
                // .replace("|", "\\|")
                // .replace("{", "\\{")
                // .replace("}", "\\}")
                // .replace(".", "\\.")
                // .replace("!", "\\!")
}

pub(crate) async fn send_message<T>(bot: &Bot, chat_id: ChatId, message: T) -> HandlerResult
where
    T: Into<String>,
{
    bot.send_message(chat_id, escape_chars(message))
        // .parse_mode(ParseMode::MarkdownV2)
        .await?;
    Ok(())
}
