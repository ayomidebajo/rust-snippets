mod bar;
pub use self::bar::Bar;

pub fn do_foo() {
	println!("Hi from foo!");
	// println!("I'm calling bar::hello() for you: {:?}", Bar::hello());
}