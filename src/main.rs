mod bf_interpreter;

use bf_interpreter::{BfInterpreter, Ret};
use std::{
    io::{BufRead, Read, Write},
    process::ExitCode,
};

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        eprintln!("ERROR: invalid args");
        return ExitCode::FAILURE;
    }

    let arg = args.first().unwrap();

    if arg == "--repl" {
        repl();
    } else {
        let content = std::fs::read_to_string(arg).unwrap();
        run_interpreter(content.as_bytes());
    }

    ExitCode::SUCCESS
}

fn run_interpreter(program: &[u8]) -> Result<(), String> {
    let mut interpreter = BfInterpreter::new(program);

    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    loop {
        match interpreter.step() {
            Ok(ret) => {
                match ret {
                    Ret::Input => {
                        let mut buf = [0u8; 1];
                        match stdin.read_exact(&mut buf) {
                            Ok(_) => {
                                interpreter.set_input(buf[0]);
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                    Ret::Output(byte) => {
                        write!(stdout, "{}", unsafe {
                            std::str::from_utf8_unchecked(&[byte])
                        })
                        .unwrap();
                        stdout.flush().unwrap();
                    }
                    Ret::Continue => {
                        // Continue.
                    }
                    Ret::Finished => break,
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}

fn repl() {
    let mut buf = String::new();
    loop {
        print!("# ");
        std::io::stdout().flush().unwrap();

        buf.clear();
        let input_ret = std::io::stdin().lock().read_line(&mut buf);
        let buf = buf.trim_end();

        match input_ret {
            Ok(_) => {
                if buf == "exit" {
                    return;
                }

                match run_interpreter(buf.as_bytes()) {
                    Ok(_) => {
                        // Do nothing.
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                        std::io::stdout().flush().unwrap();
                    }
                }
            }
            Err(_) => {
                break;
            }
        }
    }
}
