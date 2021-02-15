use chrono::{NaiveDateTime, Utc};
use chrono_tz::Tz;
use num_integer::Integer;
use regex::{Captures, Regex};

lazy_static! {
    pub static ref TIMEFROM_REGEX: Regex =
        Regex::new(r#"<<timefrom:(?P<time>\d+):(?P<format>.+)?>>"#).unwrap();
    pub static ref TIMENOW_REGEX: Regex =
        Regex::new(r#"<<timenow:(?P<timezone>(?:\w|/|_)+):(?P<format>.+)?>>"#).unwrap();
}

fn fmt_displacement(format: &str, seconds: u64) -> String {
    let mut seconds = seconds;
    let mut days: u64 = 0;
    let mut hours: u64 = 0;
    let mut minutes: u64 = 0;

    for (rep, time_type, div) in [
        ("%d", &mut days, 86400),
        ("%h", &mut hours, 3600),
        ("%m", &mut minutes, 60),
    ]
    .iter_mut()
    {
        if format.contains(*rep) {
            let (divided, new_seconds) = seconds.div_rem(&div);

            **time_type = divided;
            seconds = new_seconds;
        }
    }

    format
        .replace("%s", &seconds.to_string())
        .replace("%m", &minutes.to_string())
        .replace("%h", &hours.to_string())
        .replace("%d", &days.to_string())
}

pub fn substitute(string: &str) -> String {
    let new = TIMEFROM_REGEX.replace(string, |caps: &Captures| {
        let final_time = caps.name("time").unwrap().as_str();
        let format = caps.name("format").unwrap().as_str();

        if let Ok(final_time) = final_time.parse::<i64>() {
            let dt = NaiveDateTime::from_timestamp(final_time, 0);
            let now = Utc::now().naive_utc();

            let difference = {
                if now < dt {
                    dt - Utc::now().naive_utc()
                } else {
                    Utc::now().naive_utc() - dt
                }
            };

            fmt_displacement(format, difference.num_seconds() as u64)
        } else {
            String::new()
        }
    });

    TIMENOW_REGEX
        .replace(&new, |caps: &Captures| {
            let timezone = caps.name("timezone").unwrap().as_str();

            println!("{}", timezone);

            if let Ok(tz) = timezone.parse::<Tz>() {
                let format = caps.name("format").unwrap().as_str();
                let now = Utc::now().with_timezone(&tz);

                now.format(format).to_string()
            } else {
                String::new()
            }
        })
        .to_string()
}
