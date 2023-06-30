use std::io::Write;
use std::{fs::File, io::Read};

use crate::ai::chat::History;
use crate::config::get_ini_value;
pub fn write_history_to_file(history: &History) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let json_data = serde_json::to_string(&history)?;
    let mut file = File::create(format!(
        "history_{}.json",
        get_ini_value("chat_ai", "character").unwrap()
    ))?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}
pub fn read_json_from_file() -> Option<History> {
    let file = File::open(format!(
        "history_{}.json",
        get_ini_value("chat_ai", "character").unwrap()
    ));
    if let Ok(mut f) = file {
        let mut json_data = String::new();
        let res = f.read_to_string(&mut json_data);
        match res {
            Ok(_) => log::info!("Read from file"),
            Err(_) => log::error!("Error reading from file"),
        };
        let deserialized_data = serde_json::from_str(&json_data);
        if let Ok(d) = deserialized_data {
            Some(d)
        } else {
            Some(History {
                internal: vec![],
                visible: vec![],
            })
        }
    } else {
        Some(History {
            internal: vec![],
            visible: vec![],
        })
    }
}
