use rand::Rng;
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};

use crate::config::get_ini_value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatResult {
    pub history: History,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub results: Vec<ChatResult>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatRequest {
    user_input: String,
    history: History,
    mode: String,
    character: String,
    instruction_template: String,
    your_name: String,
    regenerate: bool,
    _continue: bool,
    stop_at_newline: bool,
    chat_prompt_size: u32,
    chat_generation_attempts: u32,
    max_new_tokens: u32,
    do_sample: bool,
    temperature: f64,
    top_p: f64,
    typical_p: u32,
    epsilon_cutoff: u32,
    eta_cutoff: u32,
    tfs: u32,
    top_a: u32,
    repetition_penalty: f64,
    top_k: u32,
    min_length: u32,
    no_repeat_ngram_size: u32,
    num_beams: u32,
    penalty_alpha: u32,
    length_penalty: u32,
    early_stopping: bool,
    mirostat_mode: u32,
    mirostat_tau: u32,
    mirostat_eta: f64,
    seed: i32,
    add_bos_token: bool,
    truncation_length: u32,
    ban_eos_token: bool,
    skip_special_tokens: bool,
    stopping_strings: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct History {
    pub internal: Vec<Vec<String>>,
    pub visible: Vec<Vec<String>>,
}
// undo's last response of the ai
impl History {
    pub fn undo(&mut self) -> History {
        self.internal.pop();
        self.visible.pop();
        self.clone()
    }
    pub fn last(self) -> Option<String> {
        let mut history = self;
        let last = history.internal.pop();
        match last {
            Some(last) => {
                return Some(last[1].clone());
            }
            None => None,
        }
    }
}

pub async fn play_promt(prompt: String, history: History) -> Result<ApiResponse, ApiError> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let seed = rand::thread_rng().gen_range(0..100000);
    let chat_request = ChatRequest {
        user_input: prompt,
        history,
        mode: "chat".to_string(),
        character: get_ini_value("chat_ai", "character").expect("No character specified"),
        instruction_template: "Vicuna-v1.1".to_string(),
        your_name: get_ini_value("chat_ai", "your_name").expect("No username specified"),
        regenerate: false,
        _continue: false,
        stop_at_newline: false,
        chat_prompt_size: 250,
        chat_generation_attempts: 1,
        max_new_tokens: 250,
        do_sample: true,
        temperature: 1.0,
        top_p: 0.5,
        typical_p: 1,
        epsilon_cutoff: 0,
        eta_cutoff: 0,
        tfs: 1,
        top_a: 0,
        repetition_penalty: 1.18,
        top_k: 40,
        min_length: 0,
        no_repeat_ngram_size: 0,
        num_beams: 4,
        penalty_alpha: 0,
        length_penalty: 1,
        early_stopping: false,
        mirostat_mode: 0,
        mirostat_tau: 5,
        mirostat_eta: 0.1,
        seed,
        add_bos_token: true,
        truncation_length: 2048,
        ban_eos_token: false,
        skip_special_tokens: true,
        stopping_strings: vec![],
    };
    if let Some(url) = get_ini_value("chat_ai", "url") {
        let response = client
            .post(format!("{}api/v1/chat", url))
            .headers(headers)
            .body(chat_request.to_string())
            .send()
            .await;
        if let Ok(x) = response {
            let status = &x.status();
            let text = x.text();

            // Deserialize the JSON string into the ApiResponse struct
            let api_response: Result<ApiResponse, serde_json::Error> =
                serde_json::from_str(text.await.as_ref().ok().unwrap());
            match api_response {
                Ok(x) => {
                    return Ok(x);
                }
                Err(_) => {
                    if status == &StatusCode::UNAUTHORIZED {
                        return Err(ApiError::SeverStarting);
                    } else if status == &StatusCode::NOT_FOUND {
                        return Err(ApiError::ServerNotUp);
                    }
                    if status == &StatusCode::OK {
                        // this means most likely that the iamge could not be generated
                        return Err(ApiError::ImageServerNotUp);
                    }
                    return Err(ApiError::Unknown);
                }
            }
        } else {
            return Err(ApiError::Unknown);
        }
    }
    Err(ApiError::Unknown) //get the status
}

#[derive(Debug)]
pub enum ApiError {
    ServerNotUp,
    SeverStarting,
    ImageServerNotUp,
    Unknown,
}
impl ToString for ChatRequest {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
