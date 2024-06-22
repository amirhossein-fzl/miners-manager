use regex::Regex;
use std::env;
use std::error::Error;
use teloxide::prelude::*;
use tokio::sync::mpsc;

use crate::bot_handler::BotHandler;

pub struct TelegramBotService {
    bot: Bot,
    handler: BotHandler,
}

impl TelegramBotService {
    pub fn new() -> Self {
        TelegramBotService {
            bot: Bot::new(env::var("BOT_TOKEN").unwrap()),
            handler: BotHandler::new(),
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(text) = msg.text() {
            if text == "/start" && msg.chat.id.0.to_string() == env::var("ADMIN_ID").unwrap() {
                let _ = &self
                    .handler
                    .start_message_handler(&self.bot, &msg, false)
                    .await;
            }
        }
        Ok(())
    }

    async fn handle_callback_query(
        &self,
        q: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(data) = &q.data {
            if let Some(message) = &q.message {
                if message.chat.id.0.to_string() == env::var("ADMIN_ID").unwrap() {
                    if let Some(captures) = Regex::new(r"^supervisor_(.*)_start")
                        .unwrap()
                        .captures(&data)
                    {
                        let _ = &self
                            .handler
                            .supervisor_start_handler(
                                &self.bot,
                                &message,
                                &q,
                                captures.get(1).unwrap().as_str(),
                            )
                            .await;
                    } else if let Some(captures) = Regex::new(r"^supervisor_(.*)_stop")
                        .unwrap()
                        .captures(&data)
                    {
                        let _ = &self
                            .handler
                            .supervisor_stop_handler(
                                &self.bot,
                                &message,
                                &q,
                                captures.get(1).unwrap().as_str(),
                            )
                            .await;
                    } else if let Some(captures) =
                        Regex::new(r"^supervisor_(.*)").unwrap().captures(&data)
                    {
                        let _ = &self
                            .handler
                            .supervisor_manager_handler(
                                &self.bot,
                                &message,
                                &q,
                                captures.get(1).unwrap().as_str(),
                            )
                            .await;
                    } else if &data.as_str() == &"start_supervisors" {
                        let _ = &self
                            .handler
                            .supervisor_start_all_handler(&self.bot, &message, &q)
                            .await;
                    } else if &data.as_str() == &"stop_supervisors" {
                        let _ = &self
                            .handler
                            .supervisor_stop_all_handler(&self.bot, &message, &q)
                            .await;
                    } else if &data.as_str() == &"reload_supervisors" {
                        let _ = &self
                            .handler
                            .supervisor_reload_handler(&self.bot, &message, &q)
                            .await;
                    } else if &data.as_str() == &"back_to_home" {
                        let _ = &self
                            .handler
                            .start_message_handler(&self.bot, &message, true)
                            .await;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn initialize(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (tx, mut rx) = mpsc::channel(100);

        let bot_clone = self.bot.clone();
        let tx_clone = tx.clone();

        let message_handler = move |msg: Message| {
            let tx = tx_clone.clone();
            async move {
                tx.send(("message", serde_json::to_value(msg).unwrap()))
                    .await
                    .unwrap();
                respond(())
            }
        };

        let callback_handler = move |q: CallbackQuery| {
            let tx = tx.clone();
            async move {
                tx.send(("callback", serde_json::to_value(q).unwrap()))
                    .await
                    .unwrap();
                respond(())
            }
        };

        let handler = dptree::entry()
            .branch(Update::filter_message().endpoint(message_handler))
            .branch(Update::filter_callback_query().endpoint(callback_handler));

        tokio::spawn(async move {
            Dispatcher::builder(bot_clone, handler)
                .enable_ctrlc_handler()
                .build()
                .dispatch()
                .await;
        });

        while let Some((update_type, update_data)) = rx.recv().await {
            match update_type {
                "message" => {
                    let msg: Message = serde_json::from_value(update_data).unwrap();
                    self.handle_message(msg).await?;
                }
                "callback" => {
                    let q: CallbackQuery = serde_json::from_value(update_data).unwrap();
                    self.handle_callback_query(q).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
