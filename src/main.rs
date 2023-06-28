mod ai;
mod config;
mod history;
mod message_parsers;
mod modules;

use std::process::exit;

use tokio::fs;

use crate::{
    ai::chat::History,
    config::get_ini_value,
    history::file::write_history_to_file,
    message_parsers::{
        is_question_about_appointment, is_question_about_pokemon,
        is_question_about_weather,
    },
    modules::{
        audio::{extract_audio_from_file, generate_voice},
        pokeapi::PokemonEx,
    },
};
use dotenv::dotenv;
use teloxide::{
    net::Download,
    types::{MediaKind::Audio, MediaKind::Voice},
};
use teloxide::{prelude::*, types::InputFile};
#[tokio::main]
async fn main() {
    dotenv().ok();

    std::env::set_var("RUST_LOG", get_ini_value("log", "level").unwrap());
    pretty_env_logger::init();
    log::info!("Starting waifu bot...");
    let token = get_ini_value("telegram", "token");
    match token {
        Some(t) => {
            let bot = Bot::new(t);

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

                let message_text = msg.text();
                if user == &get_ini_value("telegram", "user").unwrap() {
                    match message_text {
                        Some(text) => {
                            let res = ai_reply(chat_id, &bot, text, history).await;
                            match res {
                                Ok(_) => {
                                    //lol seems like it always returns Ok
                                    log::info!("ai has replied")
                                }
                                Err(e) => {
                                    log::error!("Error: {}", e)
                                }
                            }
                        }

                        None => match msg.kind {
                            teloxide::types::MessageKind::Common(msg_common) => match msg_common
                                .media_kind
                            {
                                Audio(audio) => {
                                    log::info!("audio received {:?}", audio.audio.file);
                                }
                                Voice(voice) => {
                                    // log::info!("voice received {:?}", voice.voice.file.id);;
                                    let res = bot.get_file(voice.voice.file.id).await;
                                    match res {
                                        Ok(file) => {
                                            let mut dst =
                                                fs::File::create("./out/output_audio.ogg").await?;
                                            let res = bot.download_file(&file.path, &mut dst).await;
                                            match res {
                                                Ok(()) => {
                                                    log::info!("audio downloaded");
                                                    let res = extract_audio_from_file().await;
                                                    match res {
                                                        Ok(o) => {
                                                            let heard_reply = bot
                                                                .send_message(
                                                                    chat_id,
                                                                    format!("heard: {}", &o),
                                                                )
                                                                .await;
                                                            match heard_reply {
                                                                Ok(o) => {
                                                                    log::trace!(
                                                                        "heard reply: {:?}",
                                                                        o
                                                                    )
                                                                }
                                                                Err(e) => {
                                                                    log::error!("error: {}", e)
                                                                }
                                                            }
                                                            let res = ai_reply(
                                                                chat_id,
                                                                &bot,
                                                                &o,
                                                                history.to_owned(),
                                                            )
                                                            .await;
                                                            match res {
                                                                Ok(_) => {
                                                                    log::info!("ai has replied")
                                                                }
                                                                Err(e) => {
                                                                    log::error!("Error: {}", e)
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            log::error!(
                                                                "Error extracting audio {:?}",
                                                                e
                                                            );
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    log::error!("Error downloading file {:?}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            log::error!("Error getting file {:?}", e);
                                        }
                                    }
                                }

                                _ => log::info!("unknown message type"),
                            },
                            _ => log::info!("unknown message type {:?}", msg),
                        },
                    }
                }
                // only send when user is the same as in the config

                Ok(())
            })
            .await;
        }
        None => {
            log::error!("No token found");
        }
    }
    //wait for messages
}

async fn ai_reply(
    chat_id: ChatId,
    bot: &Bot,
    message_text: &str,
    mut history: History,
) -> Result<(), Box<dyn std::error::Error>> {
    // test if user asked for pictures
    if message_parsers::user_asked_for_pictures(message_text)
        && get_ini_value("sd_ai", "enabled").unwrap() == "true"
    {
        bot.send_message(chat_id, "Generating picture...").await?;
        // generate a picture
        // ask ai for a promt.

        let mut msg = format!(
            "{}|Describe it in very high detail so the user can see it, then send it to the user|",
            message_text
        );
        log::trace!("{}", msg);

        let response = ai::chat::play_promt(msg.to_string(), history).await;
        match response {
            Ok(res) => {
                history = res.results[0].history.clone();
                match write_history_to_file(&history) {
                    Ok(_) => {
                        log::info!("history written to file")
                    }
                    Err(e) => {
                        log::error!("error writing history to file{:?}", e)
                    }
                }

                if message_parsers::has_multiple_self_references(&history.last().unwrap()) {
                    msg = format!("{} {} ", msg, &get_ini_value("sd_ai", "lora").unwrap());
                }

                let img_res = ai::image::generate_image(format!("{msg}")).await;
                match img_res {
                    Ok(_) => {
                        log::info!("photo generated");
                        //send picture
                        let input_file = InputFile::file("./out/output_image.png");
                        let res = bot.send_photo(chat_id, input_file).await;
                        match res {
                            Ok(_) => {
                                log::info!("image sent");
                            }
                            Err(e) => {
                                log::error!("{:?}", e);
                            }
                        };
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
        if is_question_about_appointment(&message_text)
            && get_ini_value("calendar", "enabled").unwrap() == "true"
        {
            log::info!("asked for appointments");
            message = modules::calendar::parse_query(message.to_string()).to_string();
            log::debug!("appointments parsed {}", message);
        }

        if is_question_about_pokemon(&message_text) {
            match modules::pokeapi::find_pokemon(message_text) {
                Some(pokemon) => {
                    let res = modules::pokeapi::get_pokemon(&pokemon).await;
                    match res {
                        Some(pokemon) => message = pokemon.to_ai_string(),
                        None => {
                            log::error!("could not get pokemon")
                        }
                    }
                }
                None => {}
            }
        }

        log::info!("message: {}", message);
        let response = ai::chat::play_promt(message.to_string(), history).await;
        //send response
        match response {
            Ok(response) => match response.results[0].history.clone().last() {
                Some(last_message) => {
                    match write_history_to_file(&response.results[0].history.clone()) {
                        Ok(_) => {
                            log::info!("history written to file")
                        }
                        Err(e) => {
                            log::error!("error writing history to file{:?}", e)
                        }
                    }
                    let res = bot.send_message(chat_id, last_message.to_owned()).await;

                    match res {
                        Ok(_) => {
                            // lets check if tts is enabled
                            if get_ini_value("tts", "enabled").unwrap() == "true" {
                                log::info!("message sent");
                                let res = generate_voice(last_message.to_owned()).await;
                                match res {
                                    Ok(_) => {
                                        let input_file = InputFile::file("./out/output.mp3");
                                        let res = bot.send_voice(chat_id, input_file).await;
                                        match res {
                                            Ok(_) => {}
                                            Err(e) => {
                                                log::error!("{:?}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("{:?}", e);
                                    }
                                }
                            }
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
                match e {
                    ai::chat::ApiError::ServerNotUp => {
                        bot.send_message(chat_id, "ai server not up or configured incorrectly")
                            .await?;
                    }
                    ai::chat::ApiError::SeverStarting => {
                        bot.send_message(chat_id, "ai server is starting").await?;
                    }
                    ai::chat::ApiError::Unknown => {
                        bot.send_message(chat_id, "unknown error").await?;
                    }
                }

                return Err(Box::new(e));
            }
        }
    }
    Ok(())
}
