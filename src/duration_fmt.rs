use num_integer::Integer;

pub fn shorthand_displacement(seconds: u64) -> String {
    let (days, seconds) = seconds.div_rem(&86400);
    let (hours, seconds) = seconds.div_rem(&3600);
    let (minutes, seconds) = seconds.div_rem(&60);

    let time_repr = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

    format!("{} days, {}", days, time_repr)
}

pub fn longhand_displacement(seconds: u64) -> String {
    let (days, seconds) = seconds.div_rem(&86400);
    let (hours, seconds) = seconds.div_rem(&3600);
    let (minutes, seconds) = seconds.div_rem(&60);

    let mut sections = vec![];

    for (var, name) in [days, hours, minutes, seconds]
        .iter()
        .zip(["days", "hours", "minutes", "seconds"].iter())
    {
        if *var > 0 {
            sections.push(format!("{} {}", var, name));
        }
    }

    sections.join(", ")
}
