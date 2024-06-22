use teloxide::{
    payloads::{AnswerCallbackQuerySetters, EditMessageTextSetters, SendMessageSetters},
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode},
    Bot,
};

use crate::{supervisor::{SupervisorService, Process}, utils::markdown};

pub struct BotHandler {
    supervisor_service: SupervisorService,
}

impl BotHandler {
    pub fn new() -> Self {
        BotHandler {
            supervisor_service: SupervisorService::new(),
        }
    }

    async fn get_supervisor_process_list(&self) -> Vec<Process> {
        let supervisor_service = self.supervisor_service.clone();
        tokio::task::spawn_blocking(move || supervisor_service.process_list())
            .await
            .unwrap()
    }

    fn create_supervisor_keyboard(&self, process_list: &Vec<Process>) -> InlineKeyboardMarkup {
        let mut keyboard = vec![vec![InlineKeyboardButton::callback(
            "Supervisors 👇".to_owned(),
            "-".to_owned(),
        )]];

        let supervisor_services_button = process_list
            .chunks(2)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|program| {
                        let state_emoji = if program.state == "RUNNING" || program.state == "RESTARTING" {
                            "✅"
                        } else {
                            "❌"
                        };

                        InlineKeyboardButton::callback(
                            format!("{} {}", &program.name, state_emoji),
                            format!("supervisor_{}", &program.name),
                        )
                    })
                    .collect()
            })
            .collect::<Vec<Vec<InlineKeyboardButton>>>();

        keyboard.extend(supervisor_services_button);

        keyboard.extend(vec![
            vec![
            InlineKeyboardButton::callback("Start all programs ✅", "start_supervisors"),
            InlineKeyboardButton::callback("Stop all programs ❌", "stop_supervisors"),
        ],
            vec![
            InlineKeyboardButton::callback("Reload supervisor 🔄", "reload_supervisors"),
        ]
        ]);

        InlineKeyboardMarkup::new(keyboard)
    }

    fn format_supervisor_status(&self, process_list: &Vec<Process>) -> String {
        let supervisor_programs = process_list
            .iter()
            .map(|program| {
                let state_emoji = if program.state == "RUNNING" || program.state == "RESTARTING" {
                    "✅"
                } else {
                    "❌"
                };

                format!(
                    "*name*: {}\n*status*: *{}* {}",
                    markdown::replace_specail_chars(&program.name),
                    &program.state,
                    state_emoji
                )
            })
            .collect::<Vec<String>>()
            .join(&markdown::replace_specail_chars(&"\n---------------------------------\n".to_string()));

        format!(
            "You can see a summary of the supervisor's status:\n\n\n{}\n\n\\.",
            &supervisor_programs
        )
    }

    async fn update_supervisor_message(&self, bot: &Bot, msg: &Message, text: String, keyboard: InlineKeyboardMarkup) -> Result<(), teloxide::RequestError> {
        bot.edit_message_text(msg.chat.id, msg.id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .reply_markup(keyboard)
            .await?;
        Ok(())
    }

    pub async fn start_message_handler(&self, bot: &Bot, msg: &Message, is_back: bool) -> Result<(), teloxide::RequestError> {
        let process_list = &self.get_supervisor_process_list().await;
        let text = self.format_supervisor_status(&process_list);
        let keyboard = self.create_supervisor_keyboard(&process_list);

        if is_back {
            self.update_supervisor_message(bot, msg, text, keyboard).await?;
        } else {
            bot.send_message(msg.chat.id, text)
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(keyboard)
                .await?;
        }
        Ok(())
    }

    async fn handle_supervisor_action(&self, bot: &Bot, msg: &Message, query: &CallbackQuery, supervisor_name: &str, action: &str) -> Result<(), teloxide::RequestError> {
        
        let supervisor_name_ref = supervisor_name.to_string();
        let action_ref = action.to_string();
        let supervisor_service = self.supervisor_service.clone();
        let action_result = tokio::task::spawn_blocking(move || {
            // let action = action_ref.clone().as_str();
            match action_ref.clone().as_str() {
                "start" => supervisor_service.start_process(supervisor_name_ref.clone()),
                "stop" => supervisor_service.stop_process(supervisor_name_ref.clone()),
                _ => false,
            }
        })
        .await
        .unwrap();

        if !action_result && action != "manage" {
            bot.answer_callback_query(&query.id)
                .text(format!("Error in {} {} supervisor.", action, supervisor_name))
                .show_alert(true)
                .await?;
            return Ok(());
        }

        let process_list = &self.get_supervisor_process_list().await;
        let supervisor_process = process_list.iter().find(|program| program.name == supervisor_name);

        if let Some(program) = supervisor_process {
            let state_emoji = if program.state == "RUNNING" || program.state == "RESTARTING" {
                "✅"
            } else {
                "❌"
            };

            let keyboard = vec![
                vec![
                    InlineKeyboardButton::callback("Start".to_owned(), format!("supervisor_{}_start", &program.name)),
                    InlineKeyboardButton::callback("Stop".to_owned(), format!("supervisor_{}_stop", &program.name)),
                ],
                vec![InlineKeyboardButton::callback("Back 🔙".to_owned(), "back_to_home")],
            ];

            let text = format!(
                "*name*: {}\n*status*: *{}* {}\nuptime: {}\n\n\\.",
                markdown::replace_specail_chars(&program.name),
                &program.state,
                state_emoji,
                markdown::replace_specail_chars(&program.uptime)
            );

            self.update_supervisor_message(bot, msg, text, InlineKeyboardMarkup::new(keyboard)).await?;

            if action != "manage" {
                bot.answer_callback_query(&query.id)
                    .text(format!("Supervisor {} {}ed successfully ✅.", supervisor_name, action))
                    .show_alert(false)
                    .await?;
            }
        } else {
            bot.answer_callback_query(&query.id)
                .text(format!("The supervisor {} not found.", supervisor_name))
                .show_alert(true)
                .await?;
        }

        Ok(())
    }

    pub async fn supervisor_manager_handler(&self, bot: &Bot, msg: &Message, query: &CallbackQuery, supervisor_name: &str) -> Result<(), teloxide::RequestError> {
        self.handle_supervisor_action(bot, msg, query, supervisor_name, "manage").await
    }

    pub async fn supervisor_start_handler(&self, bot: &Bot, msg: &Message, query: &CallbackQuery, supervisor_name: &str) -> Result<(), teloxide::RequestError> {
        self.handle_supervisor_action(bot, msg, query, supervisor_name, "start").await
    }

    pub async fn supervisor_stop_handler(&self, bot: &Bot, msg: &Message, query: &CallbackQuery, supervisor_name: &str) -> Result<(), teloxide::RequestError> {
        self.handle_supervisor_action(bot, msg, query, supervisor_name, "stop").await
    }

    pub async fn supervisor_stop_all_handler(&self, bot: &Bot, msg: &Message, query: &CallbackQuery) -> Result<(), teloxide::RequestError> {
        let supervisor_service = self.supervisor_service.clone();
        let stop_all_status = tokio::task::spawn_blocking(move || supervisor_service.stop_all_process())
            .await
            .unwrap();

        if !stop_all_status {
            bot.answer_callback_query(&query.id)
                .text("Error in stop all supervisor programs.")
                .show_alert(true)
                .await?;
        } else {
            bot.answer_callback_query(&query.id)
                .text("All supervisor programs stopped successfully ✅.")
                .show_alert(true)
                .await?;

            let process_list = &self.get_supervisor_process_list().await;
            let text = self.format_supervisor_status(&process_list);
            let keyboard = self.create_supervisor_keyboard(&process_list);

            self.update_supervisor_message(bot, msg, text, keyboard).await?;
        }

        Ok(())
    }

    pub async fn supervisor_start_all_handler(&self, bot: &Bot, msg: &Message, query: &CallbackQuery) -> Result<(), teloxide::RequestError> {
        let supervisor_service = self.supervisor_service.clone();
        let start_all_status = tokio::task::spawn_blocking(move || supervisor_service.start_all_process())
            .await
            .unwrap();

        if !start_all_status {
            bot.answer_callback_query(&query.id)
                .text("Error in stop all supervisor programs.")
                .show_alert(true)
                .await?;
        } else {
            bot.answer_callback_query(&query.id)
                .text("All supervisor programs stopped successfully ✅.")
                .show_alert(true)
                .await?;

            let process_list = &self.get_supervisor_process_list().await;
            let text = self.format_supervisor_status(&process_list);
            let keyboard = self.create_supervisor_keyboard(&process_list);

            self.update_supervisor_message(bot, msg, text, keyboard).await?;
        }

        Ok(())
    }

    pub async fn supervisor_reload_handler(&self, bot: &Bot, msg: &Message, query: &CallbackQuery) -> Result<(), teloxide::RequestError> {
        let supervisor_service = self.supervisor_service.clone();
        let reload_status = tokio::task::spawn_blocking(move || supervisor_service.reload_supervisor())
            .await
            .unwrap();

        if !reload_status {
            bot.answer_callback_query(&query.id)
                .text("Error in reload supervisor programs.")
                .show_alert(true)
                .await?;
        } else {
            bot.answer_callback_query(&query.id)
                .text("Supervisor reloaded successfully ✅.")
                .show_alert(true)
                .await?;

            let process_list = &self.get_supervisor_process_list().await;
            let text = self.format_supervisor_status(&process_list);
            let keyboard = self.create_supervisor_keyboard(&process_list);

            self.update_supervisor_message(bot, msg, text, keyboard).await?;
        }

        Ok(())
    }
}