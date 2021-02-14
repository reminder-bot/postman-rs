use num_integer::Integer;

pub fn fmt_displacement(format: &str, seconds: u64) -> String {
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
