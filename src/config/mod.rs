extern crate ini;
use ini::Ini;

pub fn get_ini_value(section_name: &str, property_name: &str) -> Option<String> {
    let conf = Ini::load_from_file("config/config.ini").ok()?;
    let section = conf.section(Some(section_name))?;
    let value = section.get(property_name)?.to_owned();
    Some(value)
}
