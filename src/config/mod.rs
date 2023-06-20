extern crate ini;
use ini::Ini;

pub fn get_ini_value(section_name: &str, property_name: &str) -> Option<String> {
    log::trace!("getting config");
    let conf = Ini::load_from_file("./config/config.ini");
    match conf {
        Ok(conf) => {
            log::trace!("conf: {:?}", conf);
            let section = conf.section(Some(section_name))?;
            log::trace!("section: {:?}", section);
            let value = section.get(property_name)?.to_owned();
            log::trace!("value: {:?}", value);
            Some(value)
        }
        Err(e) => {
            log::error!("error: {:?}", e);
            None
        }
    }
}
