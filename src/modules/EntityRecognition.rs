use std::{thread};

use rust_bert::{
    pipelines::{ner::{NERModel, Entity}, token_classification::TokenClassificationConfig}
};
use serde::Deserialize;
pub async fn recognize(input: String) -> Option<Vec<Entity>> {
    let thread = thread::spawn(move || {
        let ner_model = NERModel::new(TokenClassificationConfig::default()).unwrap();

        let sentences = [input];

        let output = ner_model.predict(&sentences);
        output
    });

    let res = thread.join();
    match res {
        Ok(o) => {
            match o.first() {
                Some(res) => return Some(res.to_vec()),
                None => return None,
            };
        }
        Err(e) =>{
            return None;
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