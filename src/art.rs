#![allow(dead_code)]
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// ── Orange palette ──────────────────────────────────────────────────────────
// Primary:  255, 140,   0
// Bright:   255, 180,  50
// Dim:      180,  90,   0

pub fn orange(s: &str) -> colored::ColoredString {
    s.truecolor(255, 140, 0)
}

pub fn orange_bright(s: &str) -> colored::ColoredString {
    s.truecolor(255, 195, 60)
}

pub fn orange_dim(s: &str) -> colored::ColoredString {
    s.truecolor(180, 90, 0)
}

// ── Logo ────────────────────────────────────────────────────────────────────
//  The PulsarDB pulsar-dot logo, orange gradient top→bottom

pub fn logo_lines() -> Vec<String> {
    // Each tuple: (line text, r, g, b)
    let rows: &[(&str, u8, u8, u8)] = &[
        ("        ....        ", 255, 195,  60),
        ("      ........      ", 255, 175,  40),
        ("     ..........     ", 255, 160,  20),
        ("    ............    ", 255, 140,   0),
        ("   ..............   ", 245, 130,   0),
        ("    ............    ", 235, 115,   0),
        ("     ..........     ", 220, 100,   0),
        ("      ........      ", 200,  85,   0),
        ("        ....        ", 180,  70,   0),
    ];
    rows.iter()
        .map(|(line, r, g, b)| {
            line.chars()
                .map(|c| {
                    if c == '.' {
                        format!("{}", "●".truecolor(*r, *g, *b))
                    } else {
                        " ".to_string()
                    }
                })
                .collect()
        })
        .collect()
}

// ── Banner (logo + wordmark) ────────────────────────────────────────────────

pub fn print_banner() {
    println!();
    let logo = logo_lines();

    // Pair each logo row with a wordmark line
    let wordmark: &[&str] = &[
        "",
        &format!("  {}", orange_bright("P U L S A R  D B").bold()),
        &format!("  {}", "Database control plane".truecolor(150, 150, 150)),
        "",
        &format!("  {} {}", orange("▸").bold(), "Multi-branch Postgres"),
        &format!("  {} {}", orange("▸").bold(), "Prisma-ready connections"),
        &format!("  {} {}", orange("▸").bold(), "Real-time monitoring"),
        "",
        &format!("  {}", "v0.1.0".truecolor(100, 100, 100)),
    ];

    for (i, logo_line) in logo.iter().enumerate() {
        let word = wordmark.get(i).copied().unwrap_or("");
        println!("  {}    {}", logo_line, word);
    }
    println!();
    println!(
        "  {}",
        "─".repeat(50).truecolor(80, 80, 80)
    );
    println!();
}

// ── Welcome / onboarding ─────────────────────────────────────────────────────

pub fn print_welcome() {
    print_banner();
    println!(
        "  {} {}",
        orange("→").bold(),
        "Run the following to get started:".white()
    );
    println!();
    println!(
        "    {}",
        "pulsardb login".truecolor(255, 140, 0).bold()
    );
    println!();
}

// ── Spinner (orange, braille animation) ─────────────────────────────────────

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            // Braille frames — form a smooth rotating circle
            .tick_strings(&[
                "\x1b[38;2;255;140;0m⠋\x1b[0m",
                "\x1b[38;2;255;150;10m⠙\x1b[0m",
                "\x1b[38;2;255;160;20m⠹\x1b[0m",
                "\x1b[38;2;255;170;30m⠸\x1b[0m",
                "\x1b[38;2;255;180;40m⠼\x1b[0m",
                "\x1b[38;2;255;170;30m⠴\x1b[0m",
                "\x1b[38;2;255;160;20m⠦\x1b[0m",
                "\x1b[38;2;255;150;10m⠧\x1b[0m",
                "\x1b[38;2;255;140;0m⠇\x1b[0m",
                "\x1b[38;2;220;110;0m⠏\x1b[0m",
            ])
            .template("{spinner} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.truecolor(200, 200, 200).to_string());
    pb.enable_steady_tick(Duration::from_millis(90));
    pb
}

// ── Pulse loader (used for longer ops) ──────────────────────────────────────

pub fn pulse_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&[
                "\x1b[38;2;255;140;0m◉\x1b[0m\x1b[38;2;100;100;100m○○○\x1b[0m",
                "\x1b[38;2;100;100;100m○\x1b[0m\x1b[38;2;255;140;0m◉\x1b[0m\x1b[38;2;100;100;100m○○\x1b[0m",
                "\x1b[38;2;100;100;100m○○\x1b[0m\x1b[38;2;255;140;0m◉\x1b[0m\x1b[38;2;100;100;100m○\x1b[0m",
                "\x1b[38;2;100;100;100m○○○\x1b[0m\x1b[38;2;255;140;0m◉\x1b[0m",
                "\x1b[38;2;100;100;100m○○\x1b[0m\x1b[38;2;255;140;0m◉\x1b[0m\x1b[38;2;100;100;100m○\x1b[0m",
                "\x1b[38;2;100;100;100m○\x1b[0m\x1b[38;2;255;140;0m◉\x1b[0m\x1b[38;2;100;100;100m○○\x1b[0m",
            ])
            .template("{spinner}  {msg}")
            .unwrap(),
    );
    pb.set_message(msg.truecolor(200, 200, 200).to_string());
    pb.enable_steady_tick(Duration::from_millis(150));
    pb
}

// ── Step printer ─────────────────────────────────────────────────────────────

pub fn step(n: usize, total: usize, msg: &str) {
    let badge = format!("[{n}/{total}]");
    println!(
        "  {}  {}",
        badge.truecolor(255, 140, 0).bold(),
        msg.white()
    );
}

pub fn step_ok(msg: &str) {
    println!(
        "  {}  {}",
        "✔".truecolor(255, 140, 0).bold(),
        msg.white()
    );
}

pub fn step_err(msg: &str) {
    println!(
        "  {}  {}",
        "✖".truecolor(220, 60, 60).bold(),
        msg.white()
    );
}

// ── Section divider ──────────────────────────────────────────────────────────

pub fn divider() {
    println!(
        "  {}",
        "─".repeat(50).truecolor(60, 60, 60)
    );
}

pub fn section(title: &str) {
    println!();
    println!(
        "  {} {}",
        orange("◆").bold(),
        title.truecolor(230, 230, 230).bold()
    );
    println!(
        "  {}",
        "─".repeat(title.len() + 4).truecolor(80, 50, 0)
    );
}
