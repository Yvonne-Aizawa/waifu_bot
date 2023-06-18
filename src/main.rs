mod config;
mod ai;
mod history;
mod message_parsers;
mod modules;
use teloxide::{prelude::*, types::InputFile};
use crate::{
    config::get_ini_value,
    ai::chat::History, history::file::write_history_to_file,
    message_parsers::is_question_about_appointment,
};
#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting waifu bot...");

    let bot = Bot::new(get_ini_value("telegram", "token").unwrap());
    //wait for messages
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        let opt_history = history::file::read_json_from_file();
        let mut history = History {
            internal: vec![],
            visible: vec![],
        };
        match opt_history {
            Some(h) => {
                history = h;
            }
            None => {
                log::error!("No history found");
            }
        }
        let user = msg.from().unwrap().username.as_ref().unwrap();
        let chat_id = msg.chat.id;
        let message_text = msg.text().unwrap();
        // only send when user is the same as in the config
        if user == &get_ini_value("telegram", "user").unwrap() {
            // test if user asked for pictures
            if message_parsers::user_asked_for_pictures(message_text) {
                bot.send_message(chat_id, "Generating picture...").await?;
                // generate a picture
                // ask ai for a promt.

                let mut msg = format!("{}|Describe it in very high detail so the user can see it, then send it to the user|", message_text);
                log::trace!("{}", msg);

                let response = ai::chat::play_promt(msg.to_string(), history).await;
                match response {
                    Ok(res) => {
                        history = res.results[0].history.clone();
                        write_history_to_file(&history);
                        if message_parsers::has_multiple_self_references(&history.last().unwrap()) {
                         msg = format!("{} {} ",msg, &get_ini_value("sd_ai", "lora").unwrap());
                        }

                        let img_res = ai::image::generate_image(format!("{msg}")).await;
                        match img_res {
                            Ok(_) => {
                                log::info!("photo generated");
                                //send picture
                                let input_file = InputFile::file("output_image.png");
                                let res = bot.send_photo(chat_id, input_file).await;
                                match res {
                                    Ok(_) => {
                                        log::info!("image sent");
                                    }
                                    Err(e) => {
                                        log::error!("{:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                //notify user of error
                                let res = bot.send_message(chat_id, "could not send image").await;
                                log::error!("{:?}", e);
                                match res {
                                    Ok(_) => {
                                        log::info!("user notified of error");
                                    }
                                    Err(e) => {
                                        log::error!("{:?}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("{:?}", e);
                        let res = bot
                            .send_message(chat_id, "could not contact ai server for image promt")
                            .await;
                        match res {
                            Ok(_) => {
                                log::info!("user notified of error");
                            }
                            Err(e) => {
                                log::error!("{:?}", e);
                            }
                        }
                    }
                }
            } else {
                let mut message = message_text.to_owned();
                // no image was requested
                // TODO implement history
                // TODO implement calendar
                if is_question_about_appointment(&message_text) {
                    log::info!("asked for appointments");
                    message = modules::calendar::parse_query(message.to_string()).to_string();
                    log::debug!("appointments parsed {}", message);
                }

                log::info!("message: {}", message);
                let response = ai::chat::play_promt(message.to_string(), history).await;
                //send response
                match response {
                    Ok(response) => match response.results[0].history.clone().last() {
                        Some(last_message) => {
                            write_history_to_file(&response.results[0].history.clone());

                            let res = bot.send_message(chat_id, last_message).await;
                            match res {
                                Ok(_) => {
                                    log::info!("message sent");
                                }
                                Err(e) => {
                                    log::error!("{:?}", e);
                                }
                            }
                        }
                        None => {
                            log::error!("some kind of error occured")
                        }
                    },
                    Err(e) => {
                        // TODO notify user of error
                        log::error!("{:?}", e);
                    }
                }
            }
        }
        Ok(())
    })
    .await;
}
