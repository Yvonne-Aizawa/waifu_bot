use base64::decode;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::Write;

use crate::config;
use config::get_ini_value;
// !TODO should return result of the generation
pub async fn generate_image(prompt: String) -> Result<(), Error> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let override_settings = HashMap::new();
    // override_settings.insert("key1".to_string(), "value1".to_string());

    let alwayson_scripts = HashMap::new();
    // alwayson_scripts.insert("script1".to_string(), "value1".to_string());
    let image_request = UserContext {
        enable_hr: false,
        denoising_strength: 0,
        firstphase_height: 0,
        firstphase_width: 0,
        hr_scale: 2,
        hr_upscaler: "".to_string(),
        hr_second_pass_steps: 0,
        prompt,
        styles: vec![],
        seed: -1,
        subseed: -1,
        hr_resize_x: 0,
        hr_resize_y: 0,
        hr_sampler_name: "".to_string(),
        hr_prompt: "".to_string(),
        hr_negative_prompt: "".to_string(),
        subseed_strength: 0,
        seed_resize_from_h: -1,
        seed_resize_from_w: -1,
        sampler_name: "".to_string(),
        batch_size: 1,
        n_iter: 1,
        steps: 50,
        cfg_scale: 7,
        width: 512,
        height: 512,
        restore_faces: false,
        tiling: false,
        do_not_save_samples: false,
        do_not_save_grid: false,
        negative_prompt: get_ini_value("sd_ai", "negative_promt").unwrap(),
        eta: 0,
        s_min_uncond: 0,
        s_churn: 0,
        s_tmax: 0,
        s_tmin: 0,
        s_noise: 1,
        override_settings,
        override_settings_restore_afterwards: true,
        script_args: vec![],
        sampler_index: "Euler".to_string(),
        script_name: "".to_string(),
        send_images: true,
        save_images: false,
        alwayson_scripts,
    };
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client
        .post(format!(
            "{}/sdapi/v1/txt2img",
            get_ini_value("sd_ai", "url").expect("No stable diffusion url defined")
        ))
        .headers(headers)
        .body(serde_json::to_string(&image_request).unwrap())
        .send()
        .await;

    if let Ok(x) = response {
        let response_string = x.text().await;
        let res_image = serde_json::from_str(&response_string.unwrap());
        match res_image {
            Ok(_) => {
                let image: GeneratedImage = res_image.unwrap();
                let result = save_without_splitting_image(image.images[0].to_string());
                return result;
            }
            Err(e) => {
                log::error!("Error{}", e);
                return Err(Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "Could not generate image",
                ));
            }
        }
    }
    Ok(())

    // return Ok(image)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeneratedImage {
    images: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct UserContext {
    enable_hr: bool,
    denoising_strength: u32,
    firstphase_width: u32,
    firstphase_height: u32,
    hr_scale: u32,
    hr_upscaler: String,
    hr_second_pass_steps: u32,
    hr_resize_x: u32,
    hr_resize_y: u32,
    hr_sampler_name: String,
    hr_prompt: String,
    hr_negative_prompt: String,
    prompt: String,
    styles: Vec<String>,
    seed: i32,
    subseed: i32,
    subseed_strength: u32,
    seed_resize_from_h: i32,
    seed_resize_from_w: i32,
    sampler_name: String,
    batch_size: u32,
    n_iter: u32,
    steps: u32,
    cfg_scale: u32,
    width: u32,
    height: u32,
    restore_faces: bool,
    tiling: bool,
    do_not_save_samples: bool,
    do_not_save_grid: bool,
    negative_prompt: String,
    eta: u32,
    s_min_uncond: u32,
    s_churn: u32,
    s_tmax: u32,
    s_tmin: u32,
    s_noise: u32,
    override_settings: std::collections::HashMap<String, String>,
    override_settings_restore_afterwards: bool,
    script_args: Vec<String>,
    sampler_index: String,
    script_name: String,
    send_images: bool,
    save_images: bool,
    alwayson_scripts: std::collections::HashMap<String, String>,
}

fn save_without_splitting_image(image_base64: String) -> std::io::Result<()> {
    let image_data = decode(image_base64);

    let file_path = "./out/output_image.png";
    let file = File::create(file_path);
    if let Ok(mut f) = file {
        f.write_all(&image_data.unwrap())?;
        log::debug!("Image saved to {}", file_path);
        return Ok(());
    }
    Err(file.unwrap_err())
}
