//! Tabular data display example.
//!
//! Accepts one command-line argument which is used as the format string for
//! printing a small set of tabular rows. If no argument is provided, a default
//! is used. Demonstrates formatting according to user input.

#[macro_use]
extern crate runtime_fmt;

fn main() {
    let format_spec = match std::env::args().nth(1) {
        Some(arg) => arg,
        None => "| {id:<5} | {name:<20} | {city:<20} |".into(),
    };

    if let Err(e) = rt_println!(format_spec, id="ID", name="NAME", city="CITY") {
        println!("error in header: {}", e);
        if let runtime_fmt::Error::BadSyntax(_) = e { return }
    }
    for row in rows() {
        if let Err(e) = rt_println!(format_spec, id=row.id, name=row.name, city=row.city) {
            println!("error: {}", e);
            return;
        }
    }
}

struct Row {
    id: u64,
    name: &'static str,
    city: &'static str,
}

fn rows() -> Vec<Row> {
    vec![
        Row { id: 1, name: "Bort", city: "Neotokyo" },
        Row { id: 2, name: "Xyzzyx", city: "Twisty Passageville" },
        Row { id: 3, name: "Yoshikage Kira", city: "Morioh" },
        Row { id: 4, name: "M.", city: "Vaporwave" },
    ]
}

