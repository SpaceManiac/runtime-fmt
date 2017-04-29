#[macro_use] extern crate runtime_fmt;

fn main() {
	let format_string = "Hello, {}!";
	rt_println!(format_string, "world").unwrap();
	rt_println!("bogus value {}").unwrap_err();
	rt_println!("bogus}{string").unwrap_err();
}
