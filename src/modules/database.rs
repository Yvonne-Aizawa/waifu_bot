use chrono::{DateTime, Utc};
use reqwest::header;
use rust_bert::{
    pipelines::sentence_embeddings::{SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType},
    RustBertError,
};
use serde::{Deserialize, Serialize};

use std::thread;
pub async fn send_string_to_server(
    string: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let res = vectorize(string.to_string()).await.unwrap();
    
    send(string.to_string(), res).await
}
pub async fn get_simmilar(
    string: String,
) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
    let res = vectorize(string.to_string()).await.unwrap();
    
    get(res).await
}
pub async fn vectorize(input: String) -> Result<Vec<f32>, RustBertError> {
    let thread = thread::spawn(move || {
        let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
            .create_model()?;

        let sentences = [input];

        
        model.encode(&sentences)
    });

    let res = thread.join().unwrap();
    match res {
        Ok(res) => Ok(res[0].clone()),
        Err(e) => Err(e),
    }
}
async fn send(
    id: String,
    vector: Vec<f32>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let inserta = Insert {
        id,
        vector,
        metadata: MetaData {
            date: DateTime::<Utc>::from(std::time::SystemTime::now()),
        },
    };

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client
        .post("http://localhost:3002/collections/test/insert")
        .headers(headers)
        .body(inserta.to_string())
        .send()
        .await;
    Ok(response.unwrap().text().await.unwrap())
}
#[derive(Serialize, Deserialize)]

struct vecResponse {
    score: f32,
    embedding: vecEmbedding,
}
#[derive(Serialize, Deserialize)]

struct vecEmbedding {
    id: String,
    vector: Vec<f32>,
}
async fn get(query: Vec<f32>) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let inserta = getSim { query };

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client
        .post("http://localhost:3002/collections/test")
        .headers(headers)
        .body(inserta.to_string())
        .send()
        .await;
    let api_response: Result<Vec<Response>, serde_json::Error> =
        serde_json::from_str(&response?.text().await?);
    match api_response {
        Ok(res) => {
            let first = res.first();
            match first {
                None => Err("No results".into()),
                Some(first) => {
                    Ok(Response {
                        score: first.score,
                        embedding: first.embedding.clone(),
                    })
                }
            }
        }
        Err(e) => Err(e.into()),
    }
}

#[derive(Serialize, Deserialize)]
pub struct Insert {
    id: String,
    vector: Vec<f32>,
    metadata: MetaData,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct MetaData {
    pub date: DateTime<Utc>,
}
#[derive(Serialize, Deserialize)]

pub struct getSim {
    query: Vec<f32>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Response {
    pub score: f64,
    pub embedding: Embedding,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Embedding {
    pub id: String,
    pub vector: Vec<f64>,
    pub metadata: MetaData,
}

impl ToString for getSim {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl ToString for Insert {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
