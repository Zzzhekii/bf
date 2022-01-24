use std::env;
use std::fs;
use colored::Colorize;

// Usage
fn print_usage() {
    println!("Usage: Last argument must be the path to the source file.")
}

// Format error message (with pointer to error in source)
fn format_error_source(source: &str, error_pos: usize) -> String{
    format!("{}'{}'{}", &source[..error_pos], &source.chars().nth(error_pos).unwrap().to_string().red(), &source[error_pos+1..])
}

fn main() {
    // Accept arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        print_usage();
        return;
    }

    // Read source file
    let source = fs::read_to_string(&args[args.len()-1])
        .expect(format!("Couldn't read provided \"{}\" source code file.", &args[args.len()-1]).as_str());
    
    // Run provided bf code
    let res = bf::run(&source);

    // Check for error codes
    if let Err(e) = res
    {
        match e {
            bf::Error::UnclosedLB(v) => println!("Error: Unclosed left bracket\n{}", format_error_source(&source, v)),
            bf::Error::UnclosedRB(v) => println!("Error: Unclosed right bracket\n{}", format_error_source(&source, v)),
            bf::Error::NegDP => println!("Attempted to set negative data pointer."),
        }
    }
}

mod bf {
    use std::collections::HashMap;
    use std::time::Instant;
    use getch::Getch;
    use std::io::Write;
    use std::io::stdout;

    // Error codes
    pub enum Error {
        UnclosedLB(usize),   // Unclosed left bracket with usize pointing to char in source
        UnclosedRB(usize),   // Unclosed right bracket with usize pointing to char in source
        NegDP,               // Attempted to set negative data pointer
    }

    // Run from source
    pub fn run(source: &str) -> Result<(), Error>{
        // Start message
        print!("Loading bf code...\n");

        // Parde the program and print time
        let now = Instant::now();
        let parsed = parse(source)?;
        print!("Program has been loaded. [{}s]\n", now.elapsed().as_millis() as f32 / 1000f32);

        // Execute program and print time
        let now = Instant::now();
        runtime(&parsed)?;
        print!("Program has been executed successfuly. [{}s]\n", now.elapsed().as_millis() as f32 / 1000f32);

        Ok(())
    }

    // Token types (?)
    #[derive(PartialEq)]
    enum Token {
        DPoint(i32), // Move data poiner
        Cell(i16),   // Add value to a cell at DP
        Output,      // output byte at DP
        Input,       // input byte at DP
        LB(usize),   // left bracket
        RB(usize),   // right bracket
    }

    // Run parsed program
    fn runtime(tokens: &Vec<Token>) -> Result<(), Error> {
        // Cells hash and data pointer
        let mut cells: HashMap<usize, u8> = HashMap::new();
        let mut dp: usize = 0;

        // Execution cycle
        let mut i: usize = 0;
        while i < tokens.len() {
            match &tokens[i] {
                // Check if dp isn't going to be negative, else return error
                Token::DPoint(v) => if dp as i32 + v >= 0 { dp = (dp as i32 + v) as usize } else { return Err(Error::NegDP) },

                // Change cell value
                Token::Cell(v) => {
                    let value: i16 = get_cell(&cells, dp) as i16 + v;
                    set_cell(&mut cells, dp, value)
                },

                // Output cell's byte
                Token::Output => {
                    print!("{}", get_cell(&cells, dp) as char);
                    // Flush!
                    stdout().flush().expect("Unable to flush terminal");
                },

                // Input byte to a cell using getch
                Token::Input => {
                    let getch = Getch::new();
                    let byte = getch.getch().expect("Unable to get input from terminal.");
                    set_cell(&mut cells, dp, byte as i16);
                },

                // Process brackets
                Token::LB(v) => {
                    if get_cell(&cells, dp) == 0 {
                        i = *v
                    }
                },
                Token::RB(v) => {
                    if get_cell(&cells, dp) != 0 {
                        i = *v
                    }
                },
            }

            // Increment instruction pointer
            i += 1;
        }

        print!("\n");

        // Set cell's value
        fn set_cell(cells: &mut HashMap<usize, u8>, key: usize,  value: i16) {
            // If cell already contains smth, then remove it; insert new value
            if cells.contains_key(&key) {
                cells.remove(&key);
                cells.insert(key, value as u8);
            } else {
                cells.insert(key, value as u8);
            }
        }

        // Get cell's value
        fn get_cell(cells: &HashMap<usize, u8>, key: usize) -> u8 {
            // If cell doesn't exist, retirn 0
            if cells.contains_key(&key) {
                cells[&key]
            } else {
                0
            }
        }

        Ok(())
    }

    // Parse source code
    fn parse(source: &str) -> Result<Vec<Token>, Error> {
        // Tokens list
        let mut tokens: Vec<Token> = Vec::new();

        // Saves position of to-be-parsed left brackets in parsed code; second argument is the position in source
        let mut left_brackets: Vec<(usize, usize)> = Vec::new();

        // Iterate over source
        let mut i: usize = 0;
        while i < source.len() {
            match source.bytes().nth(i).unwrap() {
                // If < or >
                b'>' | b'<' => {
                    let mut value: i32 = 0;
                    // Count all < and > in a row (and add or substract from 'value')
                    while i < source.len() {
                        if source.bytes().nth(i).unwrap() == b'>' {
                            value += 1
                        } else if source.bytes().nth(i).unwrap() == b'<' {
                            value -= 1
                        } else {
                            break
                        }
                        // Increment iterator
                        i += 1
                    }
                    // Decrement iterator, since we'll increment it again in the end.
                    i -= 1;
                    // Push the value
                    tokens.push(Token::DPoint(value))
                },
                // If + or -
                b'+' | b'-' => {
                    let mut value: i16 = 0;
                    // Count all + and - in a row (and add or substract from 'value')
                    while i < source.len() {
                        if source.bytes().nth(i).unwrap() == b'+' {
                            value += 1
                        } else if source.bytes().nth(i).unwrap() == b'-' {
                            value -= 1
                        } else {
                            break
                        }
                        // Increment iterator
                        i += 1
                    }
                    // Decrement iterator, since we'll increment it again in the end.
                    i -= 1;
                    // Push the value
                    tokens.push(Token::Cell(value))
                },
                // Output && input tokens
                b'.' => tokens.push(Token::Output),
                b',' => tokens.push(Token::Input),
                // Brackets
                b'[' => {
                    // Push empty token
                    tokens.push(Token::LB(0));
                    // Push current position (in tokens AND in source)
                    left_brackets.push((tokens.len() - 1, i));
                },
                b']' => {
                    // Get matching left bracket
                    let lb = match left_brackets.pop() {
                        Some(s) => s,
                        // Or else return an error
                        None => return Err(Error::UnclosedRB(i)),
                    };
                    // Push token pointing to it's mathcing left bracket
                    tokens.push(Token::RB(lb.0));
                    // Replace matching left bracket token
                    tokens[lb.0] = Token::LB(tokens.len() - 1);
                },
                _ => (),
            }
            // Increment iterator
            i += 1;
        }

        // Return unclosed left bracket error
        if left_brackets.len() > 0 
            { return Err(Error::UnclosedLB(left_brackets[0].1)) }

        Ok(tokens)
    }
}