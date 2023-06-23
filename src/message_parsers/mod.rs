use regex::Regex;

#[allow(dead_code)]
pub fn is_question_about_time(question: &str) -> bool {
    let lower_question = question.to_lowercase();
    lower_question.contains("time")
        || lower_question.contains("hour")
        || lower_question.contains("minute")
        || lower_question.contains("clock")
        || lower_question.contains("date")
}
pub fn is_question_about_appointment(question: &str) -> bool {
    let lower_question = question.to_lowercase();
    lower_question.contains("appointment")
        || lower_question.contains("calendar")
        || lower_question.contains("schedule")
}
pub fn user_asked_for_pictures(question: &str) -> bool {
    let lower_question = question.to_lowercase();
    lower_question.contains("picture")
        || lower_question.contains("photo")
        || lower_question.contains("image")
        || lower_question.contains("see you")
        || lower_question.contains("show")
}
pub fn is_question_about_pokemon(question: &str) -> bool {
    let lower_question = question.to_lowercase();
    lower_question.contains("pokemon")
        || lower_question.contains("pokedex")
        || lower_question.contains("dex")
        || lower_question.contains("poke")
        || lower_question.contains("pok")
        || lower_question.contains("type")
}
pub fn is_question_about_weather(question: &str) -> bool {
    let lower_question = question.to_lowercase();
    lower_question.contains("temperature")
        || lower_question.contains("weather")
        || lower_question.contains("forecast")
}

pub fn has_multiple_self_references(text: &str) -> bool {
    let regex = Regex::new(r"\b(i|me|my|mine|me)\b").unwrap();
    let mut count = 0;

    for word in text.split_whitespace() {
        if regex.is_match(word) {
            count += 1;
            if count >= 2 {
                return true;
            }
        }
    }

    false
}
