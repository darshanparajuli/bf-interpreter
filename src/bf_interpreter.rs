use std::collections::HashMap;

pub(crate) struct BfInterpreter {
    pc: usize,
    data_ptr: usize,
    program: Box<[Token]>,
    cells: Vec<u8>,
    matching_parens: HashMap<usize, usize>,
}

impl BfInterpreter {
    pub(crate) fn new(program: &[u8]) -> Self {
        let program = Self::parse_program(program);
        let matching_parens = Self::find_matching_parens(&program);
        Self {
            pc: 0,
            data_ptr: 0,
            program,
            cells: vec![0u8; 30_000],
            matching_parens,
        }
    }

    fn parse_program(program: &[u8]) -> Box<[Token]> {
        use Token::*;
        program
            .iter()
            .map(|b| {
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
            .filter(|e| e.is_some())
            .map(|e| e.unwrap())
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn find_matching_parens(program: &[Token]) -> HashMap<usize, usize> {
        let mut map = HashMap::new();
        let mut stack = vec![];

        for (i, b) in program.iter().copied().enumerate() {
            if b == Token::BeginLoop {
                stack.push((i, b));
            } else if b == Token::EndLoop {
                let (matching_index, _) = stack.pop().unwrap();
                map.insert(i, matching_index);
                map.insert(matching_index, i);
            }
        }

        return map;
    }

    pub(crate) fn step(&mut self) -> Ret {
        if self.pc >= self.program.len() {
            return Ret::Finished;
        }

        let p = self.program[self.pc];

        use Token::*;
        match p {
            IncDataPtr => {
                // Increment the data pointer by one (to point to the next cell to the right).
                self.data_ptr += 1;
                self.pc += 1;
            }
            DecDataPtr => {
                // Decrement the data pointer by one (to point to the next cell to the left).
                assert!(self.data_ptr > 0, "Memory underflow!");
                if self.data_ptr == 0 {
                    panic!("Memory underflow!");
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
                return Ret::Output(self.cells[self.data_ptr]);
            }
            ReadByte => {
                // Accept one byte of input, storing its value in the byte at the data pointer.
                self.pc += 1;
                return Ret::Input;
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

        Ret::Continue
    }

    pub(crate) fn set_input(&mut self, input: u8) {
        self.cells[self.data_ptr] = input;
    }
}

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
