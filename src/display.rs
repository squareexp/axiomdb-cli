use colored::Colorize;
use comfy_table::{presets::UTF8_BORDERS_ONLY, Attribute, Cell, Color, ContentArrangement, Table};

// Orange palette — consistent with art.rs
fn o(s: &str) -> colored::ColoredString   { s.truecolor(255, 140, 0) }
fn dim(s: &str) -> colored::ColoredString { s.truecolor(100, 100, 100) }

pub fn ok(msg: &str) {
    println!("  {}  {}", "✔".truecolor(255, 140, 0).bold(), msg.white());
}

pub fn err(msg: &str) {
    eprintln!("  {}  {}", "✖".truecolor(220, 60, 60).bold(), msg.white());
}

pub fn info(msg: &str) {
    println!("  {}  {}", "◈".truecolor(255, 140, 0), msg.truecolor(200, 200, 200));
}

#[allow(dead_code)]
pub fn warn(msg: &str) {
    println!("  {}  {}", "⚠".truecolor(255, 200, 50).bold(), msg.truecolor(220, 220, 180));
}

pub fn header(title: &str) {
    println!();
    println!("  {}  {}", o("◆").bold(), title.truecolor(230, 230, 230).bold());
    println!("  {}", "─".repeat((title.len() + 5).min(60)).truecolor(80, 50, 0));
}

/// Aligned key → value pairs, keys in orange-dim, values in white
pub fn kv(pairs: &[(&str, String)]) {
    if pairs.is_empty() { return; }
    let max = pairs.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, val) in pairs {
        let padded = format!("{:<width$}", key, width = max + 2);
        println!("  {}  {}", dim(&padded), val.truecolor(220, 220, 220));
    }
}

/// Styled table — orange header cells
pub fn table(headers: &[&str], rows: Vec<Vec<String>>) {
    if rows.is_empty() {
        println!("  {}", dim("(none)"));
        return;
    }
    let mut t = Table::new();
    t.load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(
            headers.iter()
                .map(|h| {
                    Cell::new(h)
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Rgb { r: 255, g: 140, b: 0 })
                })
                .collect::<Vec<_>>(),
        );
    for row in rows {
        t.add_row(row);
    }
    println!("{t}");
}

/// Colour-code a status string (orange for positive states)
pub fn status_color(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "active" | "succeeded" | "live"  => s.truecolor(255, 140, 0).bold().to_string(),
        "pending" | "running" | "idle"   => s.truecolor(200, 200, 50).to_string(),
        "failed"  | "deleted"            => s.truecolor(220, 60, 60).to_string(),
        _                                => dim(s).to_string(),
    }
}

/// Format optional — em-dash if None/empty
pub fn opt(v: Option<&str>) -> String {
    match v {
        Some(s) if !s.is_empty() => s.truecolor(220, 220, 220).to_string(),
        _ => dim("—").to_string(),
    }
}
