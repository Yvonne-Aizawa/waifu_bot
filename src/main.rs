mod ai;
mod config;
mod history;
mod message_parsers;
mod modules;
use oobabooga_rs::{History, Mode};
use tokio::fs;

use crate::{
    config::get_ini_value,
    history::file::write_history_to_file,
    message_parsers::{
        is_question_about_appointment, is_question_about_pokemon, is_question_about_weather,
    },
    modules::{
        audio::{extract_audio_from_file, generate_voice},
        pokeapi::PokemonEx, database::{send_string_to_server, get_simmilar},
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
                            if text.starts_with("/"){
                                if text == "/reset"{
                                let his_res = write_history_to_file(&History {
                                    internal: vec![],
                                    visible: vec![],
                                });
                                match his_res{
                                    Ok(_) =>{
                                        bot.send_message(chat_id, "History has been reset.").await;

                                    }
                                    Err(e) => {
                                        log::error!("{}", e);
                                    }
                                }
                            }
                            else if text == "/undo"{
                                let history = history::file::read_json_from_file();
                                match history{
                                    Some(mut h) => {
                                        write_history_to_file(&h.undo());
                                        bot.send_message(chat_id, format!("undo Sucessful. \n last message: {}", h.last().unwrap_or_default())).await;

                                    }
                                    None => {
                                    }
                                }
                            }
                            else if text == "/sticker"{
                                let sticker = InputFile::file("/home/yvonne/Documents/GitHub/teloxide/stickers/Embarrasment.png");
                                bot.send_sticker(chat_id, sticker).await;
                            }
                            }else{
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
    history: History,
) -> Result<(), Box<dyn std::error::Error>> {
    //create ai client and config
    let mut ai_config = oobabooga_rs::Config::default();
    ai_config.url = get_ini_value("chat_ai", "url").unwrap();
    let ai_client = oobabooga_rs::Client::new(ai_config);
    let mut chat_config = oobabooga_rs::ChatRequest::default();
    chat_config.mode = Mode::Chat;
    chat_config.character = get_ini_value("chat_ai", "character").unwrap();
    chat_config.your_name = get_ini_value("chat_ai", "your_name").unwrap();

    chat_config.history = history.clone();
    chat_config.regenerate = false;
    chat_config._continue = true;
    chat_config.stop_at_newline = false;
    chat_config.chat_prompt_size = 2048;
    chat_config.chat_generation_attempts = 1;
    chat_config.chat_instruct_command = "Continue the chat dialogue below. Write a single reply for the character \"Assistant\"\n\n".to_string();
    chat_config.max_new_tokens = 250;
    chat_config.do_sample = true;
    chat_config.temprature = 0.7;
    chat_config.top_p = 0.1;
    chat_config.typical_p = 1.0;
    chat_config.epsilon_cutoff = 0.0;
    chat_config.eta_cutoff = 0.0;
    chat_config.tfs = 0;
    chat_config.top_a = 0;
    chat_config.repetition_penalty = 1.18;
    chat_config.top_k = 40;
    chat_config.min_length = 0;
    chat_config.no_repeat_ngram_size = 0;
    chat_config.num_beams = 1;
    chat_config.penalty_alpha = 0.0;
    chat_config.length_penalty = 1.0;
    chat_config.early_stopping = false;
    chat_config.mirostat_mode = 0;
    chat_config.mirostat_mode_tau = 5;
    chat_config.mirostat_mode_eta = 0.1;
    chat_config.seed = -1;
    chat_config.add_bos_token = true;
    chat_config.truncation_length = 2048;
    chat_config.ban_eos_token = false;
    chat_config.skip_special_tokens = true;
    chat_config.stopping_strings = vec![];

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


        chat_config.user_input = msg.clone();
        let response = ai_client.get_chat(chat_config).await;
        match response {
            Ok(res) => {
                log::info!("ai replied");
                match write_history_to_file(&res) {
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
        //find simmilar
        // let sim_res = get_simmilar(message.clone()).await;
        // match sim_res{
        //     Ok(res) => {
        //         log::info!("message: {} simmilar found {:?} score: {} ",message_text, res.embedding.id, res.score);
        //         bot.send_message(chat_id, format!("message: {} simmilar found {:?} score: {}, metadata {} ",message_text, res.embedding.id, res.score, res.embedding.metadata.date)).await;
        //     }
            
        //     Err(e) => {
        //         log::error!("{:?}", e);
        //     }
        // }
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
        if is_question_about_weather(&message_text) {
            log::info!("asked for weather {}", message_text);
            let mut config = huggingface_inference_rs::Config::default();
            config.key = get_ini_value("huggingface", "token").unwrap();
            let client = huggingface_inference_rs::Client::new(config);
            let res = client.get_classifications(message_text.to_owned()).await;
            match &res {
                Ok(res) => {
                    let mut first_loc: Vec<&str> = Vec::new();

                    // TODO implement weather module
                    // if res contains a LOC entity_group
                    log::info!("res: {:?}", res);
                    for entity in res {
                        if entity.entity_group == "LOC" {
                            first_loc.push(entity.word.as_ref());
                        }
                    }
                    if first_loc.len() == 0 {
                        for entity in res {
                            if entity.entity_group == "ORG" {
                                first_loc.push(entity.word.as_ref());
                            }
                        }
                    }
                    // if there is a first location
                    log::info!("first location: {:?}", first_loc);
                    if first_loc.len() > 0 {
                        match modules::weather::get_weather(first_loc[0].to_string()).await {
                            None => {
                                log::error!("could not get weather");
                            }
                            Some(w) => {
                                message = format!("{} | this is the weather information you can relay it to the user |  {}", message, w);
                            }
                        }
                        log::info!("weather: {:?}", message);
                    }
                }

                Err(e) => {
                    log::error!("error: {}", e)
                }
            }
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
        // let out = send_string_to_server(message.clone()).await;


        log::info!("message: {}", message);
        chat_config.user_input = message;

        let response = ai_client.get_chat(chat_config).await;
        log::info!("response: {:?}", response);

        //send response
        match response {
            Ok(response) => match response.clone().last() {
                Some(last_message) => {
                    match write_history_to_file(&response.clone()) {
                        Ok(_) => {
                            log::info!("history written to file")
                        }
                        Err(e) => {
                            log::error!("error writing history to file{:?}", e)
                        }
                    }
                    // let out = send_string_to_server(last_message.clone()).await;
                    // log::info!("{:?}", out);
                    let res = bot.send_message(chat_id, last_message.to_owned()).await;
                    let mut hg_config = huggingface_inference_rs::Config::default();
                    hg_config.key = get_ini_value("huggingface", "token").unwrap();
                    let hg_client = huggingface_inference_rs::Client::new(hg_config);
                    //if mood is enabled
                    if get_ini_value("huggingface", "mood").unwrap() == "true" {
                        let mood = hg_client.get_emotions(last_message.to_owned()).await;
                        match mood {
                            Ok(mood) => {
                                let highest_scoring_mood = mood
                                    .iter()
                                    .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
                                match highest_scoring_mood {
                                    Some(mood) => {
                                        log::info!("mood: {:?}", mood);
                                        bot.send_sticker(
                                            chat_id,
                                            InputFile::file(format!(
                                                "./stickers/{:?}.png",
                                                mood.label
                                            )),
                                        )
                                        .await;
                                    }
                                    None => log::error!("could not get mood"),
                                }
                            }
                            Err(e) => {}
                        }
                    }

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

                return Err(e);
            }
        }
    }
    Ok(())
}
