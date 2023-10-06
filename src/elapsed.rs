use std::time::Instant;

pub fn get_elapsed_time(time: Instant) -> String {
    let elapsed = time.elapsed().as_millis();
    let formatted_elapsed = match elapsed {
        0..=999 => format!("{}ms", elapsed),
        1000..=59999 => format!("{}s", elapsed / 1000),
        60000..=3599999 => format!("{}m {}s", elapsed / 60000, (elapsed % 60000) / 1000),
        _ => format!("{}h {}m", elapsed / 3600000, (elapsed % 3600000) / 60000)
    };

    return formatted_elapsed;
}