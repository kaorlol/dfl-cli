use std::time::Instant;

const TIMES: [u128; 3] = [1000, 60000, 3600000];

pub fn get_elapsed_time(time: Instant) -> String {
    let elapsed = time.elapsed().as_millis();
    let formatted_elapsed = match elapsed {
        0..=999 => format!("{}ms", elapsed),
        1000..=59999 => format!("{}s", elapsed / TIMES[0]),
        60000..=3599999 => format!("{}m {}s", elapsed / TIMES[1], (elapsed % TIMES[1]) / TIMES[0]),
        _ => format!("{}h {}m", elapsed / TIMES[2], (elapsed % TIMES[2]) / TIMES[1])
    };

    return formatted_elapsed;
}