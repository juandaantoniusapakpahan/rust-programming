fn main(){
	println!("{} days", 31);

	println!("{0}, this is {1}. {1}, this is {0}", "Alice", "Bob");

	// as can named arguments.
	println!("{subject} {verb} {object}",
		object="the lazy dog",
		subject="the quick brown fox",
		verb="jumps over");
		
	println!("Base 10: 		{}", 69420);
	println!("Base 2 (binary): 	{:b}", 69420);
	println!("Base 8 (octal):	{:o}", 69420);
	println!("Base 16 (hexadecimal):	{:x}", 69420);
	println!("Base 16 (hexadecimal):	{:X}", 69420);
	
	println!("{number:>5}", number=5);
	println!("{number:0<5}",number=4);

	// you can use named arguments in the format specifier by appending a '$'.
	println!("{number:0>width$}", number=1, width=5);

	#[allow(dead_code)] // disable 'dead_code' which warn against unused module
	struct Structure(i32);
	
	let number: f64 = 1.0;
	let width: usize = 5;
	println!("{number:>width$}");
}
