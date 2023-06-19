use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};

use rustemon::model::pokemon::Pokemon;

pub async fn get_pokemon(name: &str) -> Option<Pokemon> {
    let rustemon_client = rustemon::client::RustemonClient::default();
    let pokemon = rustemon::pokemon::pokemon::get_by_name(name, &rustemon_client).await;
    match pokemon {
        Ok(p) => Some(p),
        Err(_) => None,
    }
}
pub trait PokemonEx {
    fn to_ai_string(&self) -> String;
}
impl PokemonEx for Pokemon {
    fn to_ai_string(&self) -> String {
    let rustemon_client = rustemon::client::RustemonClient::default();

        let types: Vec<String> = self
            .types
            .iter()
            .map(|t| t.type_.clone())
            .into_iter()
            .map(|t| t.name)
            .collect();
        let abilities: Vec<String> = self
            .abilities
            .iter()
            .map(|t| t.ability.clone())
            .into_iter()
            .map(|t| t.name)
            .collect();
        // let evolutions: Vec<String> = rustemon::evolution::evolution_chain::get_by_name(&self.name, &rustemon_client).await;
        let string = format!(
            "info about {}\ntypes {}\nabilities {}\nweight {}",
            self.name.clone(),
            types.join(" "),
            abilities.join(" "),
            self.weight
        );

        string
    }
}
use regex::Regex;

pub fn find_pokemon(sentence: &str) -> Option<String> {
    log::info!("{}", sentence);
    let file_path = "config/pokemon";
    match read_file_into_vec(file_path) {
        Ok(pokemon_list) => {
            for pokemon in pokemon_list {
                let re = Regex::new(&format!(r"\b{}\b", pokemon)).unwrap();
                if re.is_match(&sentence.to_lowercase()) {
                    log::info!("match: {}", pokemon);
                    return Some(pokemon);
                }
            }
            None
        }
        Err(e) => {
            log::error!("{}", e);
            None
        },
    }
}
fn read_file_into_vec(file_path: &str) -> io::Result<Vec<String>> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?.to_lowercase());
    }
    Ok(lines)
}
