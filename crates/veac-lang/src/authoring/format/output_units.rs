pub(super) fn bitrate(value: u64) -> String {
    scaled(value, "bps", "kbps", "mbps")
}

pub(super) fn buffer_size(value: u64) -> String {
    scaled(value, "bit", "kbit", "mbit")
}

pub(super) fn sample_rate(value: u32) -> String {
    if value % 1_000 == 0 {
        format!("{}khz", value / 1_000)
    } else {
        format!("{value}hz")
    }
}

pub(super) fn channel_layout(channels: u8) -> &'static str {
    match channels {
        1 => "mono",
        2 => "stereo",
        3 => "discrete-3",
        4 => "discrete-4",
        5 => "discrete-5",
        6 => "surround-5-1",
        7 => "discrete-7",
        8 => "surround-7-1",
        _ => "invalid",
    }
}

fn scaled(value: u64, base: &str, kilo: &str, mega: &str) -> String {
    if value % 1_000_000 == 0 {
        format!("{}{mega}", value / 1_000_000)
    } else if value % 1_000 == 0 {
        format!("{}{kilo}", value / 1_000)
    } else {
        format!("{value}{base}")
    }
}
