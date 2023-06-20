use reqwest::multipart;
use tokio::{fs::File, io::AsyncReadExt};

use std::path::PathBuf;

use rust_ai::azure::{ssml::Speak, Locale, Speech, VoiceName, SSML};

pub async fn extract_audio_from_file() -> Result<String, ()> {
    let client = reqwest::Client::new();

    // Read the file contents into a Vec<u8>
    let file = File::open("output_audio.ogg").await;
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
                Err(e) => {
                    log::error!("{:?}", e);
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

pub async fn generate_voice(string: String) -> Result<(), ()> {
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
                    log::error!("{:?}", e);
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
