use std::env;
use std::fs;

fn print_usage() {
    println!("Usage: Last argument must be the path to the source file.")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        print_usage();
        return;
    }

    let source = fs::read_to_string(&args[args.len()-1])
        .expect(format!("Couldn't read provided \"{}\" source code file.", &args[args.len()-1]).as_str());
    
    // Run provided bf code
    bf::run(&source);
}

mod bf {
    use std::collections::HashMap;
    use std::time::Instant;

    pub fn run(source: &str) {
        print!("Loading bf code...\n");

        let now = Instant::now();
        let parsed = parse(source);
        print!("Program has been loaded. [{}s]\n", now.elapsed().as_millis() as f32 / 1000f32);

        let now = Instant::now();
        runtime(&parsed);
        print!("Program has been executed successfuly. [{}s]\n", now.elapsed().as_millis() as f32 / 1000f32);
    }

    fn runtime(tokens: &Vec<Token>) {
        let mut cells: HashMap<usize, u8> = HashMap::new();
        let mut dp: usize = 0;

        let mut i: usize = 0;
        while i < tokens.len() {
            match &tokens[i] {
                Token::DPoint(v) => if dp as i32 + v >= 0 { dp = (dp as i32 + v) as usize } else { panic!("Attempted to set negative data pointer.") }, 
                Token::Cell(v) => {
                    let value: i16 = get_cell(&cells, dp) as i16 + v;
                    set_cell(&mut cells, dp, value)
                },
                Token::Output => print!("{}", get_cell(&cells, dp) as char),
                Token::Input => (), //set_cell(&mut cells, dp, std::io::stdin().bytes()[0]),
                Token::LB => {
                    if get_cell(&cells, dp) == 0 {
                        let mut nesting: i32 = 1;
                        i += 1;
                        while nesting != 0 && i < tokens.len() {
                            if tokens[i] == Token::LB { nesting += 1 }
                            if tokens[i] == Token::RB { nesting -= 1 }
                            i += 1
                        }
                        i -= 1;
                    }
                },
                Token::RB => {
                    if get_cell(&cells, dp) != 0 {
                        let mut nesting: i32 = -1;
                        i -= 1;
                        while nesting != 0 && i < tokens.len() {
                            if tokens[i] == Token::LB { nesting += 1 }
                            if tokens[i] == Token::RB { nesting -= 1 }
                            i -= 1
                        }
                    }
                },
            }

            i += 1;
        }

        print!("\n");

        fn set_cell(cells: &mut HashMap<usize, u8>, key: usize,  value: i16) {
            if cells.contains_key(&key) {
                cells.remove(&key);
                cells.insert(key, value as u8);
            } else {
                cells.insert(key, value as u8);
            }
        }

        fn get_cell(cells: &HashMap<usize, u8>, key: usize) -> u8 {
            if cells.contains_key(&key) {
                cells[&key]
            } else {
                0
            }
        }
    }

    #[derive(PartialEq, Debug)]
    enum Token {
        DPoint(i32), // Move data poiner
        Cell(i16),  // Add value to a cell at DP
        Output,     // output byte at DP
        Input,      // input byte at DP
        LB,         // left bracket
        RB,         // right bracket
    }

    fn parse(source: &str) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();

        let mut i: usize = 0;
        while i < source.len() {
            match source.bytes().nth(i).unwrap() {
                b'>' | b'<' => {
                    let mut value: i32 = 0;
                    while i < source.len() {
                        if source.bytes().nth(i).unwrap() == b'>' {
                            value += 1
                        } else if source.bytes().nth(i).unwrap() == b'<' {
                            value -= 1
                        } else {
                            break
                        }
                        i += 1
                    }
                    i -= 1;
                    tokens.push(Token::DPoint(value))
                },
                b'+' | b'-' => {
                    let mut value: i16 = 0;
                    while i < source.len() {
                        if source.bytes().nth(i).unwrap() == b'+' {
                            value += 1
                        } else if source.bytes().nth(i).unwrap() == b'-' {
                            value -= 1
                        } else {
                            break
                        }
                        i += 1
                    }
                    i -= 1;
                    tokens.push(Token::Cell(value))
                },
                b'.' => tokens.push(Token::Output),
                b',' => tokens.push(Token::Input),
                b'[' => tokens.push(Token::LB),
                b']' => tokens.push(Token::RB),
                _ => (),
            }
            i += 1;
        }
        tokens
    }
}