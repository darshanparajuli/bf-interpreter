use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct BfInterpreter {
    pc: usize,
    data_ptr: usize,
    program: Box<[Token]>,
    cells: Vec<u8>,
    matching_parens: HashMap<usize, usize>,
}

impl BfInterpreter {
    pub(crate) fn new(program: &[u8]) -> Result<Self, String> {
        let program = Self::parse_program(program);
        let matching_parens = Self::find_matching_parens(&program)?;
        Ok(Self {
            pc: 0,
            data_ptr: 0,
            program,
            cells: vec![0u8; 30_000],
            matching_parens,
        })
    }

    fn parse_program(program: &[u8]) -> Box<[Token]> {
        use Token::*;
        program
            .iter()
            .flat_map(|b| {
                match b {
                    b'>' => Some(IncDataPtr),
                    b'<' => Some(DecDataPtr),
                    b'+' => Some(IncByte),
                    b'-' => Some(DecByte),
                    b'.' => Some(WriteByte),
                    b',' => Some(ReadByte),
                    b'[' => Some(BeginLoop),
                    b']' => Some(EndLoop),
                    _ => {
                        // Ignore all other bytes.
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn find_matching_parens(program: &[Token]) -> Result<HashMap<usize, usize>, String> {
        let mut map = HashMap::new();
        let mut stack = vec![];

        for (i, b) in program.iter().copied().enumerate() {
            if b == Token::BeginLoop {
                stack.push((i, b));
            } else if b == Token::EndLoop {
                let (matching_index, _) = stack.pop().ok_or_else(|| "Missing '['".to_owned())?;
                map.insert(i, matching_index);
                map.insert(matching_index, i);
            }
        }

        if !stack.is_empty() {
            return Err("Missing ']'".to_owned());
        }

        Ok(map)
    }

    pub(crate) fn step(&mut self) -> Result<Ret, String> {
        if self.pc >= self.program.len() {
            return Ok(Ret::Finished);
        }

        let p = self.program[self.pc];

        use Token::*;
        match p {
            IncDataPtr => {
                // Increment the data pointer by one (to point to the next cell to the right).
                if self.data_ptr == self.cells.len() - 1 {
                    return Err("Memory overflow".to_owned());
                }
                self.data_ptr += 1;
                self.pc += 1;
            }
            DecDataPtr => {
                // Decrement the data pointer by one (to point to the next cell to the left).
                if self.data_ptr == 0 {
                    return Err("Memory underflow".to_owned());
                }

                self.data_ptr -= 1;
                self.pc += 1;
            }
            IncByte => {
                // Increment the byte at the data pointer by one.
                self.cells[self.data_ptr] = self.cells[self.data_ptr].wrapping_add(1);
                self.pc += 1;
            }
            DecByte => {
                // Decrement the byte at the data pointer by one.
                self.cells[self.data_ptr] = self.cells[self.data_ptr].wrapping_sub(1);
                self.pc += 1;
            }

            WriteByte => {
                // Output the byte at the data pointer.
                self.pc += 1;
                return Ok(Ret::Output(self.cells[self.data_ptr]));
            }
            ReadByte => {
                // Accept one byte of input, storing its value in the byte at the data pointer.
                self.pc += 1;
                return Ok(Ret::Input);
            }
            BeginLoop => {
                // If the byte at the data pointer is zero, then instead of moving
                // the instruction pointer forward to the next command, jump it
                // forward to the command after the matching ] command.
                if self.cells[self.data_ptr] == 0 {
                    self.pc = self.matching_parens[&self.pc] + 1;
                } else {
                    self.pc += 1;
                }
            }

            EndLoop => {
                // If the byte at the data pointer is nonzero, then instead of moving
                // the instruction pointer forward to the next command, jump it
                // back to the command after the matching [ command.
                if self.cells[self.data_ptr] != 0 {
                    self.pc = self.matching_parens[&self.pc] + 1;
                } else {
                    self.pc += 1;
                }
            }
        }

        Ok(Ret::Continue)
    }

    pub(crate) fn set_input(&mut self, input: u8) {
        self.cells[self.data_ptr] = input;
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Ret {
    Input,
    Output(u8),
    Continue,
    Finished,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Token {
    IncDataPtr,
    DecDataPtr,
    IncByte,
    DecByte,
    WriteByte,
    ReadByte,
    BeginLoop,
    EndLoop,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn hello_world() {
        let program = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let mut bf = BfInterpreter::new(program.as_bytes()).unwrap();

        let mut result = vec![];
        loop {
            let ret = bf.step().unwrap();
            match ret {
                Ret::Finished => break,
                Ret::Output(o) => {
                    result.push(o);
                }
                _ => {}
            }
        }
        assert_eq!(result, b"Hello World!\n")
    }

    #[test]
    fn memory_overflow() {
        let program = ">".repeat(30_001);
        let mut bf = BfInterpreter::new(program.as_bytes()).unwrap();
        loop {
            match bf.step() {
                Err(e) => {
                    assert_eq!(e, "Memory overflow");
                    break;
                }
                _ => {}
            }
        }
    }

    #[test]
    fn memory_underflow() {
        let program = "<";
        let mut bf = BfInterpreter::new(program.as_bytes()).unwrap();
        match bf.step() {
            Err(e) => {
                assert_eq!(e, "Memory underflow");
            }
            _ => {}
        }
    }

    #[test]
    fn handle_missing_brackets_error() {
        let cases = vec!["[", "[][][", "[[[[]]]"];
        for c in cases {
            let bf = BfInterpreter::new(c.as_bytes());
            assert!(bf.is_err());
            assert_eq!(bf.unwrap_err(), "Missing ']'");
        }

        let cases = vec!["]", "[][][]]", "[[[[]]]]]"];
        for c in cases {
            let bf = BfInterpreter::new(c.as_bytes());
            assert!(bf.is_err());
            assert_eq!(bf.unwrap_err(), "Missing '['");
        }
    }
}
