use std::thread;

use rust_bert::pipelines::{
    ner::{Entity, NERModel},
    token_classification::TokenClassificationConfig,
};
use serde::Deserialize;
pub async fn recognize(input: String) -> Option<Vec<Entity>> {
    let thread = thread::spawn(move || {
        let ner_model = NERModel::new(TokenClassificationConfig::default()).unwrap();

        let sentences = [input];

        
        ner_model.predict(&sentences)
    });

    let res = thread.join();
    match res {
        Ok(o) => {
            o.first().map(|res| res.to_vec())
        }
        Err(_e) => {
            None
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Classification {
    pub entity_group: String,
    pub score: f32,
    pub word: String,
    pub start: usize,
    pub end: usize,
}
