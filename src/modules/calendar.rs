use crate::config::get_ini_value;
use chrono::{DateTime, Datelike, Local, NaiveDateTime, Utc};
use ureq;
use url;
fn get_appointments() -> Vec<Appointment> {
    let mut appointments: Vec<Appointment> = vec![];
    let agent = ureq::Agent::new();
    let url =
        url::Url::parse(&get_ini_value("calendar", "url").expect("could not find url in calendar"))
            .unwrap();

    let username =
        get_ini_value("calendar", "username").expect("could not find username in calendar");
    let password =
        get_ini_value("calendar", "password").expect("could not find config in calendar");
    let calendars = minicaldav::get_calendars(agent.clone(), &username, &password, &url).unwrap();
    for calendar in calendars {
        let (events, _error) =
            minicaldav::get_events(agent.clone(), &username, &password, &calendar).unwrap();
        for event in events {
            let mut summary = "".to_string();
            let mut date: DateTime<Utc> = DateTime::default();
            let mut repeat = "".to_string();
            let mut set_summary = false;
            let mut set_date = false;

            for prop in event.properties() {
                if prop.0 == "SUMMARY" {
                    set_summary = true;
                    summary = prop.1.to_string();
                }
                if prop.0 == "DTSTART" {
                    date = parse_timestamp(prop.1);
                    set_date = true;
                }
                if prop.0 == "RRULE" {
                    repeat = prop.1.to_string();
                }
            }
            if set_summary && set_date {
                if repeat.is_empty() {
                    appointments.push(Appointment {
                        calendar: calendar.name().to_string(),
                        date,
                        summary: summary.clone(),
                        repeat_rule: None,
                    });
                } else {
                    appointments.push(Appointment {
                        calendar: calendar.name().to_string(),
                        date,
                        summary: summary.clone(),
                        repeat_rule: parse_recurring_event(&repeat),
                    });
                }
            }
        }
    }

    appointments
}
#[allow(dead_code)]
pub fn get_all_appointments() -> Vec<Appointment> {
    get_appointments()
}
pub fn get_all_appointments_on_date(date: DateTime<Utc>) -> Vec<Appointment> {
    let appointments = get_appointments();
    get_appointments_on_date(appointments, date)
}

fn get_appointments_on_date(
    appointments: Vec<Appointment>,
    date: DateTime<Utc>,
) -> Vec<Appointment> {
    let mut day_appointments = vec![];
    for appointment in appointments {
        if is_appointment_on_date(&appointment, date) {
            
            day_appointments.push(appointment)
        }
    }

    day_appointments
}

fn parse_timestamp(timestamp: &str) -> DateTime<Utc> {
    let datetime = match timestamp.len() {
        14 => DateTime::<Utc>::from_utc(
            NaiveDateTime::parse_from_str(timestamp, "%Y%m%dT%H%M%S").unwrap(),
            Utc,
        ),
        8 => DateTime::<Utc>::from_utc(
            NaiveDateTime::parse_from_str(
                format!("{} 00:00:00", timestamp).as_str(),
                "%Y%m%d %H:%M:%S",
            )
            .unwrap(),
            Utc,
        ),
        15 => DateTime::<Utc>::from_utc(
            NaiveDateTime::parse_from_str(timestamp, "%Y%m%dT%H%M%S").unwrap(),
            Utc,
        ),
        16 => DateTime::<Utc>::from_utc(
            NaiveDateTime::parse_from_str(&timestamp[..timestamp.len() - 1], "%Y%m%dT%H%M%S")
                .unwrap(),
            Utc,
        ),

        _ => panic!("Invalid timestamp format"),
    };

    datetime
}

fn parse_recurrence_rule(repeat_rule_string: String) -> RecurringEvent {
    log::info!("{}", repeat_rule_string);
    let mut event = RecurringEvent {
        frequency: Frequency::Yearly,
        until: None,
        by_day: None,
        by_month_day: None,
        count: None,
    };

    for line in repeat_rule_string.lines() {
        let parts: Vec<&str> = line.trim().splitn(2, '=').collect();

        if parts.len() != 2 {
            continue; // Skip lines that don't match the expected format
        }

        let key = parts[0].trim();
        let mut value = parts[1].trim();
        let mut freq = "";
        let mut until = "";
        //cut value by ; if it exists
        if value.contains(";") {
            freq = value.split(";").next().unwrap();
        } else {
            freq = value;
        }
        if value.contains("UNTIL") {
            until = value.split("UNTIL").next().unwrap();
        } else {
            until = value;
        }
        // log::info!("{}", until);
        match key {
            "FREQ" => {
                event.frequency = match freq {
                    "YEARLY" => Frequency::Yearly,
                    "MONTHLY" => Frequency::Monthly,
                    "WEEKLY" => Frequency::Weekly,
                    "DAILY" => Frequency::Daily,
                    _ => {
                        log::info!(
                            "Invalid value, keep the default frequency: {:?}",
                            event.frequency
                        );
                        event.frequency
                    } // Invalid value, keep the default frequency
                };
            }
            "UNTIL" => {
                log::info!("UNTIL: {}", value);
                event.until = Some(parse_timestamp(value).to_string());
            }
            "BYDAY" => {
                event.by_day = Some(value.to_string());
            }
            "BYMONTHDAY" => {
                if let Ok(day) = value.parse() {
                    event.by_month_day = Some(day);
                }
            }
            "COUNT" => {
                if let Ok(count) = value.parse() {
                    event.count = Some(count);
                }
            }
            _ => {} // Ignore unrecognized keys
        }
    }
    event
}

fn is_appointment_on_date(appointment: &Appointment, date: DateTime<Utc>) -> bool {
    // Check if the appointment is a one-time event (no repeat rule)
    if appointment.repeat_rule.is_none() {
        return appointment.date.date_naive() == date.date_naive();
    }

    // Check if the appointment matches the specific date based on the repeat rule
    let repeat_rule = appointment.repeat_rule.as_ref().unwrap();
    if repeat_rule.until.is_some() {
        return parse_timestamp(repeat_rule.until.as_ref().unwrap()) > date;
    }

    match repeat_rule.frequency {
        Frequency::Yearly => {
            // Check if the appointment occurs on the specific day of the year
            return appointment.date.month() == date.month() && appointment.date.day() == date.day()
        }
        Frequency::Monthly => {
            // Check if the appointment occurs on the specific day of the month
            return appointment.date.day() == date.day()
        }
        Frequency::Weekly => {
            // Check if the appointment occurs on the specific day of the week
            return appointment.date.weekday() == date.weekday()
        }
        Frequency::Daily => {
            // The appointment occurs every day, so it will always match
            return true
        }
    }

}

#[derive(Debug, Clone)]
pub struct RepeatRule {
    pub frequency: Option<String>,
    pub by_day: Option<Vec<String>>,
    pub by_monthday: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Appointment {
    pub calendar: String,
    pub date: DateTime<Utc>,
    pub summary: String,
    pub repeat_rule: Option<RecurringEvent>,
}
#[derive(Debug, Clone, Copy)]
pub enum Frequency {
    Yearly,
    Monthly,
    Weekly,
    Daily,
}

#[derive(Debug, Clone)]
pub struct RecurringEvent {
    frequency: Frequency,
    until: Option<String>, // Date and time until the event will occur (optional)
    by_day: Option<String>, // Day of the week for weekly events (optional)
    by_month_day: Option<u32>, // Day of the month for yearly events (optional)
    count: Option<u32>,    // Number of occurrences (optional)
}
use regex::Regex;

fn parse_recurring_event(data: &str) -> Option<RecurringEvent> {
    let re = Regex::new(r"(?x)
        FREQ=(?P<frequency>[A-Z]+);
        (WKST=(?P<wkst>[A-Z]+);)?
        (UNTIL=(?P<until>\d{8}T\d{6}Z);)?
        (BYDAY=(?P<by_day>[A-Z]+);)?
        (BYMONTHDAY=(?P<by_month_day>\d+);)?
    ").unwrap();

    if let Some(captures) = re.captures(data) {
        let frequency = match captures.name("frequency").unwrap().as_str() {
            "YEARLY" => Frequency::Yearly,
            "MONTHLY" => Frequency::Monthly,
            "WEEKLY" => Frequency::Weekly,
            "DAILY" => Frequency::Daily,
            _ => return None,
        };

        let until = captures.name("until").map(|m| m.as_str().to_owned());
        let by_day = captures.name("by_day").map(|m| m.as_str().to_owned());
        let by_month_day = captures.name("by_month_day").map(|m| m.as_str().parse::<u32>().ok()).flatten();

        Some(RecurringEvent {
            frequency,
            until,
            by_day,
            by_month_day,
            count: None, // You can add count parsing if needed
        })
    } else {
        None
    }
}

pub fn parse_query(mut query: String) -> String {
    let date = Local::now();

    let appointments = get_all_appointments_on_date(Utc::now());
    let mut appointment_text = "".to_string();
    for appointment in appointments {
        appointment_text = format!(
            "{} {} at {} \n",
            appointment_text.clone(),
            appointment.summary,
            appointment.date.format("%H:%M")
        )
        .to_string();
    }
    if appointment_text.is_empty() {
        appointment_text = "No appointments today".to_string();
    }

    query = format!(
    "{} \n {} can use the info provided in the || \n current time: {} \n appointments date {} \n user appointments today: \n {} \n ",
    query,get_ini_value("chat_ai", "character").unwrap(),date.format("%H:%M:%S"), Utc::now().format("%Y-%m-%d"),appointment_text
);
    query
}
