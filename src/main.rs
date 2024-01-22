mod bf_interpreter;

use bf_interpreter::{BfInterpreter, Ret};
use std::{
    io::{Read, Write},
    process::ExitCode,
};

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        eprintln!("ERROR: invalid args");
        return ExitCode::FAILURE;
    }

    let content = std::fs::read_to_string(args.first().unwrap()).unwrap();
    let mut interpreter = BfInterpreter::new(content.as_bytes());

    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    loop {
        let ret = interpreter.step();
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

    return ExitCode::SUCCESS;
}
