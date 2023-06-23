use std::io::Error;

use crate::config::get_ini_value;
use chrono::{DateTime, Datelike, Local, NaiveDateTime, Utc};
use ureq;
use url;
fn get_appointments() -> Result<Vec<Appointment>, Error> {
    let mut appointments: Vec<Appointment> = vec![];
    let agent = ureq::Agent::new();
    let url =
        url::Url::parse(&get_ini_value("calendar", "url").expect("could not find url in calendar"))
            .unwrap();

    let username =
        get_ini_value("calendar", "username").expect("could not find username in calendar");
    let password =
        get_ini_value("calendar", "password").expect("could not find config in calendar");
    let calendars_res = minicaldav::get_calendars(agent.clone(), &username, &password, &url);
    match calendars_res {
        Ok(calendars) => {
            for calendar in calendars {
                let event_res =
                    minicaldav::get_events(agent.clone(), &username, &password, &calendar);
                match event_res {
                    Ok(events) => {
                        let mut summary = "".to_string();
                        let mut date: DateTime<Utc> = DateTime::default();
                        let mut repeat = "".to_string();
                        let mut set_summary = false;
                        let mut set_date = false;
                        for event in events.0 {
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

                    Err(_) => todo!(),
                }
            }
        }
        Err(_) => todo!(),
    }

    Ok(appointments)
}
#[allow(dead_code)]
pub fn get_all_appointments() -> Result<Vec<Appointment>, Error> {
    get_appointments()
}
pub fn get_all_appointments_on_date(date: DateTime<Utc>) -> Result<Vec<Appointment>, Error> {
    let appointments_res = get_appointments();
    match appointments_res {
        Ok(appointments) => Ok(get_appointments_on_date(appointments, date)),
        Err(_) => todo!(),
    }
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
            return appointment.date.month() == date.month()
                && appointment.date.day() == date.day();
        }
        Frequency::Monthly => {
            // Check if the appointment occurs on the specific day of the month
            return appointment.date.day() == date.day();
        }
        Frequency::Weekly => {
            // Check if the appointment occurs on the specific day of the week
            return appointment.date.weekday() == date.weekday();
        }
        Frequency::Daily => {
            // The appointment occurs every day, so it will always match
            return true;
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
    let re = Regex::new(
        r"(?x)
        FREQ=(?P<frequency>[A-Z]+);
        (WKST=(?P<wkst>[A-Z]+);)?
        (UNTIL=(?P<until>\d{8}T\d{6}Z);)?
        (BYDAY=(?P<by_day>[A-Z]+);)?
        (BYMONTHDAY=(?P<by_month_day>\d+);)?
    ",
    )
    .unwrap();

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
        let by_month_day = captures
            .name("by_month_day")
            .map(|m| m.as_str().parse::<u32>().ok())
            .flatten();

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

    let appointments_res = get_all_appointments_on_date(Utc::now());
    match appointments_res {
        Ok(appointments) => {
            let mut appointment_text = "".to_string();
            for appointment in appointments {
                appointment_text = format!(
                    "{} {} at {} \n",
                    appointment_text.clone(),
                    appointment.summary,
                    convert_24_to_12_hour(&appointment.date.format("%H:%M").to_string())
                )
                .to_string();
            }
            if appointment_text.is_empty() {
                appointment_text = "No appointments today".to_string();
            }

            query = format!(
            "{} \n {} can use the info provided in the || \n current time: {} \n appointments date {} \n user appointments today: \n {} \n ",
            query,get_ini_value("chat_ai", "character").unwrap(),convert_24_to_12_hour(&date.format("%H:%M").to_string()), Utc::now().format("%Y-%m-%d"),appointment_text);
        }
        Err(_) => {}
    }

    query
}
fn convert_24_to_12_hour(time_str: &str) -> String {
    let parts: Vec<&str> = time_str.split(':').collect();
    let hour: i32 = parts[0].parse().unwrap();
    let minute: i32 = parts[1].parse().unwrap();

    let am_pm = if hour < 12 { "AM" } else { "PM" };
    let hour_12 = if hour % 12 == 0 { 12 } else { hour % 12 };
    format!("{:02}:{:02} {}", hour_12, minute, am_pm)
}
