use owo_colors::OwoColorize;
use std::{thread::sleep, time::Duration};

const TYPING_SPEED_MS: u64 = 50;
const PAUSE_MS: u64 = 800;

const TITLE_1: &str = "Make a Living";
const TITLE_2: &str = "on Code";
const TITLE_3: &str = "You Love";

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn type_text(text: &str, speed: u64) {
    for c in text.chars() {
        print!("{c}");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        sleep(Duration::from_millis(speed));
    }
}

fn print_centered(text: &str) {
    // Assuming 80 chars terminal width
    let padding = (80 - text.len()) / 2;
    println!("{:>width$}{}", "", text, width = padding);
}

pub fn show_welcome_animation() {
    clear_screen();

    // Type out the main title with color gradient
    print!("\n\n");
    print_centered("");
    type_text(&TITLE_1.bright_blue().to_string(), TYPING_SPEED_MS);
    sleep(Duration::from_millis(PAUSE_MS));

    print!(" ");
    type_text(&TITLE_2.bright_cyan().to_string(), TYPING_SPEED_MS);
    sleep(Duration::from_millis(PAUSE_MS));

    print!(" ");
    type_text(&TITLE_3.bright_white().to_string(), TYPING_SPEED_MS);
    println!("\n");
    sleep(Duration::from_millis(PAUSE_MS));

    // Show command example
    type_text(
        &"$ bounty solve facebook/react#42"
            .bright_green()
            .to_string(),
        TYPING_SPEED_MS,
    );
    println!();
    sleep(Duration::from_millis(PAUSE_MS));

    // Show success message
    print_centered("");
    type_text(
        &"✨ Branch created • PR opened • Ready to earn"
            .bright_cyan()
            .to_string(),
        TYPING_SPEED_MS + 10,
    );
    println!("\n");
    sleep(Duration::from_millis(PAUSE_MS * 2));

    clear_screen();
}
