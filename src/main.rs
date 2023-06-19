mod ai;
mod config;
mod history;
mod message_parsers;
mod modules;
use std::{path::PathBuf, process::exit};

use rust_ai::azure::{ssml::Speak, Locale, Speech, VoiceName, SSML};

use reqwest::multipart;
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
};

use crate::{
    ai::chat::History,
    config::get_ini_value,
    history::file::write_history_to_file,
    message_parsers::{is_question_about_appointment, is_question_about_pokemon},
    modules::pokeapi::PokemonEx,
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

        let message_text = msg.text();
        if user == &get_ini_value("telegram", "user").unwrap() {
            match message_text {
                Some(text) => {
                    ai_reply(chat_id, &bot, text, history).await;
                }

                None => match msg.kind {
                    teloxide::types::MessageKind::Common(msg_common) => match msg_common.media_kind
                    {
                        Audio(audio) => {
                            log::info!("audio received {:?}", audio.audio.file);
                        }
                        Voice(voice) => {
                            // log::info!("voice received {:?}", voice.voice.file.id);;
                            let res = bot.get_file(voice.voice.file.id).await;
                            match res {
                                Ok(file) => {
                                    let mut dst = fs::File::create("output_audio.ogg").await?;
                                    let res = bot.download_file(&file.path, &mut dst).await;
                                    match res {
                                        Ok(()) => {
                                            log::info!("audio downloaded");
                                            let res =
                                                extract_audio_from_file("output_audio.ogg").await;
                                            match res {
                                                Ok(o) => {
                                                    bot.send_message(
                                                        chat_id,
                                                        format!("heard: {}", &o),
                                                    )
                                                    .await;
                                                    let res = ai_reply(
                                                        chat_id,
                                                        &bot,
                                                        &o,
                                                        history.to_owned(),
                                                    )
                                                    .await;
                                                }
                                                Err(e) => {
                                                    log::error!("Error extracting audio {:?}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {}
                                    }
                                }
                                Err(e) => {}
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
async fn extract_audio_from_file(file_path: &str) -> Result<String, ()> {
    let client = reqwest::Client::new();

    // Read the file contents into a Vec<u8>
    let mut file = File::open("output_audio.ogg").await;
    match file {
        Ok(mut f) => {
            let mut file_contents = Vec::new();
            f.read_to_end(&mut file_contents).await.unwrap();

            // Create a multipart form
            let audio_file = multipart::Part::bytes(file_contents)
                .mime_str("video/ogg")
                .unwrap()
                .file_name("output_audio.ogg");
            let form = multipart::Form::new().part("audio_file", audio_file);

            // Send the request
            let response = client
                .post(
                    "http://localhost:9000/asr?task=transcribe&language=en&encode=true&output=txt",
                )
                .header("accept", "application/json")
                .multipart(form)
                .send()
                .await;
            match response {
                Ok(res) => {
                    // Print the response status and body
                    println!("Status: {}", res.status());
                    let body = res.text().await.unwrap();
                    println!("Body: {}", body);

                    Ok(body)
                }
                Err(e) => Err(()),
            }
        }
        Err(e) => Err(()),
    }
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
                write_history_to_file(&history);
                if message_parsers::has_multiple_self_references(&history.last().unwrap()) {
                    msg = format!("{} {} ", msg, &get_ini_value("sd_ai", "lora").unwrap());
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
                    write_history_to_file(&response.results[0].history.clone());

                    let res = bot.send_message(chat_id, last_message.to_owned()).await;

                    match res {
                        Ok(_) => {
                            // lets check if tts is enabled
                            if get_ini_value("tts", "enabled").unwrap() == "true" {
                                log::info!("message sent");
                                let res = generate_voice(last_message.to_owned()).await;
                                match res {
                                    Ok(_) => {
                                        let input_file = InputFile::file("output.mp3");
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
            }
        }
    }
    Ok(())
}
async fn generate_voice(string: String) -> Result<(), ()> {
    let ssml =
        SSML::from(Speak::voice_content(VoiceName::en_US_JennyNeural, &string).lang(Locale::en_US));

    log::debug!("{}", ssml.to_string());

    let result = Speech::from(ssml).tts().await;
    match result {
        Ok(result) => {
            log::debug!("{:?}", result);
            let res = std::fs::write(PathBuf::from(r"./output.mp3"), result);
            match res {
                Ok(_) => return Ok(()),
                Err(e) => {
                    return Err(());
                }
            }
        }
        Err(e) => {
            log::error!("{:?}", e);
            return Err(());
        }
    }
}
