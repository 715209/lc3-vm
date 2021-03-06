use std::{
    fs::File,
    io::{self, prelude::*, BufReader},
    path::Path,
};

/// 0x3000 is the default program counter starting position
pub const PC_START: u16 = 0x3000;

#[derive(Clone)]
pub struct Lc3State {
    memory: [u16; 65_536],
    registers: Register,
}

#[derive(Debug)]
pub struct Lc3 {
    memory: [u16; 65_536],
    registers: Register,
    running: bool,
}

impl Lc3 {
    pub fn into_state(self) -> Lc3State {
        Lc3State {
            memory: self.memory,
            registers: self.registers,
        }
    }

    pub fn from_state(state: Lc3State) -> Self {
        Self {
            running: false,
            memory: state.memory,
            registers: state.registers,
        }
    }

    pub fn load_image_file<S>(&mut self, path: S) -> Result<(), io::Error>
    where
        S: AsRef<Path>,
    {
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);

        self.insert_file_into_memory(buf_reader)
    }

    /// Read file into memory
    pub fn insert_file_into_memory<R>(&mut self, mut reader: R) -> io::Result<()>
    where
        R: Read,
    {
        let mut origin = Self::read_u16_be(&mut reader)? as usize;

        while let Ok(p) = Self::read_u16_be(&mut reader) {
            self.memory[origin] = p;
            origin += 1;
        }

        Ok(())
    }

    pub fn read_u16_be<R>(mut reader: R) -> io::Result<u16>
    where
        R: Read,
    {
        let mut buf = [0; 2];
        reader.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Extend to 16 bits
    #[inline]
    fn sign_extend(mut x: u16, bit_count: u16) -> u16 {
        if Self::is_negative(x, bit_count) {
            x |= u16::MAX << bit_count;
        }

        x
    }

    /// Check if negative
    /// read bit at position (bit_count - 1) since we begin at 0
    #[inline]
    fn is_negative(x: u16, bit_count: u16) -> bool {
        ((x >> (bit_count - 1)) & 1) == 1
    }

    pub fn read_mem(&mut self, address: u16) -> u16 {
        if address == MemoryMappedRegisters::Kbsr as u16 {
            let character = Self::read_char();

            if character != 0 {
                self.memory[MemoryMappedRegisters::Kbsr as usize] = 1 << 15;
                self.memory[MemoryMappedRegisters::Kbdr as usize] = character;
            } else {
                self.memory[MemoryMappedRegisters::Kbsr as usize] = 0;
            }
        }

        self.memory[address as usize]
    }

    /// read a single ASCII char
    pub fn read_char() -> u16 {
        let mut buffer = [0; 1];
        std::io::stdin().read_exact(&mut buffer).unwrap();

        buffer[0] as u16
    }

    pub fn run(&mut self) {
        while self.running {
            let instruction = self.memory[self.registers.pc as usize];
            let opcode = Opcode::from(instruction >> 12);
            self.registers.increment_pc();

            //println!("Opcode: {:?}, Instruction: {:b}", opcode, instruction);

            match opcode {
                Opcode::Add => self.add(instruction),
                Opcode::And => self.and(instruction),
                Opcode::Br => self.br(instruction),
                Opcode::Jmp => self.jmp(instruction),
                Opcode::Jsr => self.jsr(instruction),
                Opcode::Ld => self.ld(instruction),
                Opcode::Ldi => self.ldi(instruction),
                Opcode::Ldr => self.ldr(instruction),
                Opcode::Lea => self.lea(instruction),
                Opcode::Not => self.not(instruction),
                Opcode::Res => unimplemented!(),
                Opcode::Rti => unimplemented!(),
                Opcode::St => self.st(instruction),
                Opcode::Sti => self.sti(instruction),
                Opcode::Str => self.op_str(instruction),
                Opcode::Trap => self.trap(instruction),
            }
        }
    }
}

impl Lc3 {
    pub fn add(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let sr1 = self.registers.get_register((instruction >> 6) & 0x7);
        let is_imm_flag = (instruction >> 5) & 0x1 == 1;

        if is_imm_flag {
            let imm5 = Self::sign_extend(instruction & 0x1F, 5);
            self.registers
                .set_register(dr, u16::wrapping_add(sr1, imm5));
        } else {
            let sr2 = self.registers.get_register(instruction & 0x7);
            self.registers.set_register(dr, u16::wrapping_add(sr1, sr2));
        }

        self.registers.set_cc_flag(dr);
    }

    pub fn and(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let sr1 = self.registers.get_register((instruction >> 6) & 0x7);
        let is_imm_flag = (instruction >> 5) & 0x1 == 1;

        if is_imm_flag {
            let imm5 = Self::sign_extend(instruction & 0x1F, 5);
            self.registers.set_register(dr, sr1 & imm5);
        } else {
            let sr2 = self.registers.get_register(instruction & 0x7);
            self.registers.set_register(dr, sr1 & sr2);
        }

        self.registers.set_cc_flag(dr);
    }

    pub fn br(&mut self, instruction: u16) {
        let nzp = (instruction >> 9) & 0x7;
        let cond = self.registers.cond as u16;
        let pc_offset = Self::sign_extend(instruction & 0x1FF, 9);

        if nzp & cond != 0 {
            self.registers.pc = u16::wrapping_add(self.registers.pc, pc_offset);
        }
    }

    /// JMP also contains RET
    pub fn jmp(&mut self, instruction: u16) {
        let base_r = self.registers.get_register((instruction >> 6) & 0x7);
        self.registers.pc = base_r;
    }

    /// JSR also contains JSRR
    pub fn jsr(&mut self, instruction: u16) {
        let flag = (instruction >> 11) & 0x1;
        self.registers.set_register(7, self.registers.pc);

        if flag == 0 {
            let base_r = self.registers.get_register((instruction >> 6) & 0x7);
            self.registers.pc = base_r;
        } else {
            let pc_offset = Self::sign_extend(instruction & 0x7FF, 11);
            self.registers.pc = u16::wrapping_add(self.registers.pc, pc_offset);
        }
    }

    pub fn ld(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let pc_offset = Self::sign_extend(instruction & 0x1FF, 9);
        let to_be_loaded = self.read_mem(u16::wrapping_add(self.registers.pc, pc_offset));

        self.registers.set_register(dr, to_be_loaded);
        self.registers.set_cc_flag(dr);
    }

    pub fn ldi(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let pc_offset = Self::sign_extend(instruction & 0x1FF, 9);

        let address = self.read_mem(u16::wrapping_add(self.registers.pc, pc_offset));
        let to_be_loaded = self.read_mem(address);

        self.registers.set_register(dr, to_be_loaded);
        self.registers.set_cc_flag(dr);
    }

    pub fn ldr(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let offset = Self::sign_extend(instruction & 0x3F, 6);
        let base_r = self.registers.get_register((instruction >> 6) & 0x7);

        let to_be_loaded = self.read_mem(u16::wrapping_add(base_r, offset));

        self.registers.set_register(dr, to_be_loaded);
        self.registers.set_cc_flag(dr);
    }

    pub fn lea(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let pc_offset = Self::sign_extend(instruction & 0x1FF, 9);

        self.registers
            .set_register(dr, u16::wrapping_add(self.registers.pc, pc_offset));
        self.registers.set_cc_flag(dr);
    }

    pub fn not(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let sr = (instruction >> 6) & 0x7;

        self.registers
            .set_register(dr, !self.registers.get_register(sr));
        self.registers.set_cc_flag(dr);
    }

    pub fn st(&mut self, instruction: u16) {
        let sr = self.registers.get_register((instruction >> 9) & 0x7);
        let pc_offset = Self::sign_extend(instruction & 0x1FF, 9);

        self.memory[(u16::wrapping_add(self.registers.pc, pc_offset)) as usize] = sr;
    }

    pub fn sti(&mut self, instruction: u16) {
        let sr = self.registers.get_register((instruction >> 9) & 0x7);
        let pc_offset = Self::sign_extend(instruction & 0x1FF, 9);

        let mem_loc = self.read_mem(u16::wrapping_add(self.registers.pc, pc_offset));
        self.memory[mem_loc as usize] = sr;
    }

    pub fn op_str(&mut self, instruction: u16) {
        let sr = self.registers.get_register((instruction >> 9) & 0x7);
        let base_r = self.registers.get_register((instruction >> 6) & 0x7);
        let offset = Self::sign_extend(instruction & 0x3F, 6);

        let loc = u16::wrapping_add(base_r, offset);
        self.memory[loc as usize] = sr;
    }

    pub fn trap(&mut self, instruction: u16) {
        self.registers.set_register(7, self.registers.pc);

        let trap = Trap::from(instruction & 0xFF);
        // println!("TRAP: {:?}", trap);

        match trap {
            Trap::Getc => {
                self.registers.set_register(0, Self::read_char());
            }
            Trap::Out => {
                let loc = self.registers.get_register(0);
                print!("{}", (loc as u8) as char);
                let _ = std::io::stdout().flush();
            }
            Trap::Puts => {
                let mut loc = self.registers.get_register(0) as usize;

                while self.memory[loc] != 0x0000 {
                    let character = (self.memory[loc] as u8) as char;
                    print!("{}", character);

                    loc += 1;
                }

                let _ = std::io::stdout().flush();
            }
            Trap::In => {
                print!("Enter a character: ");
                let ascii_char = Self::read_char();
                println!("{}", ascii_char);
                self.registers.set_register(0, ascii_char);
            }
            Trap::Putsp => {
                let mut loc = self.registers.get_register(0) as usize;

                while self.memory[loc] != 0x0000 {
                    let char1 = ((self.memory[loc] & 0xFF) as u8) as char;
                    print!("{}", char1);

                    let char2 = ((self.memory[loc] >> 8) as u8) as char;
                    print!("{}", char2);

                    loc += 1;
                }

                let _ = std::io::stdout().flush();
            }
            Trap::Halt => {
                // println!("HALT");
                self.running = false;
            }
        }
    }
}

impl Default for Lc3 {
    fn default() -> Self {
        Self {
            memory: [0; 65_536],
            registers: Register::default(),
            running: true,
        }
    }
}

/// 8 general purpose registers (R0-R7)
/// 1 program counter (PC) register
/// 1 condition flags (COND) register
#[derive(Debug, Clone)]
struct Register {
    pub r0: u16,
    pub r1: u16,
    pub r2: u16,
    pub r3: u16,
    pub r4: u16,
    pub r5: u16,
    pub r6: u16,
    pub r7: u16,
    pub pc: u16,
    pub cond: ConditionalFlag,
}

impl Register {
    pub fn set_register(&mut self, register: u16, value: u16) {
        match register {
            0 => self.r0 = value,
            1 => self.r1 = value,
            2 => self.r2 = value,
            3 => self.r3 = value,
            4 => self.r4 = value,
            5 => self.r5 = value,
            6 => self.r6 = value,
            7 => self.r7 = value,
            _ => unreachable!(),
        }
    }

    pub fn get_register(&self, register: u16) -> u16 {
        match register {
            0 => self.r0,
            1 => self.r1,
            2 => self.r2,
            3 => self.r3,
            4 => self.r4,
            5 => self.r5,
            6 => self.r6,
            7 => self.r7,
            _ => unreachable!(),
        }
    }

    pub fn set_cc_flag(&mut self, r: u16) {
        let r = self.get_register(r);

        if r == 0 {
            return self.cond = ConditionalFlag::Zro;
        }

        if Lc3::is_negative(r, 16) {
            return self.cond = ConditionalFlag::Neg;
        }

        self.cond = ConditionalFlag::Pos
    }

    #[inline]
    pub fn increment_pc(&mut self) {
        self.pc += 1;
    }
}

impl Default for Register {
    fn default() -> Self {
        Self {
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            pc: PC_START,
            cond: ConditionalFlag::Zro,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ConditionalFlag {
    /// P
    Pos = 1 << 0,
    /// Z
    Zro = 1 << 1,
    /// N
    Neg = 1 << 2,
}

#[derive(Debug)]
enum Opcode {
    /// branch
    Br = 0,
    /// add
    Add,
    /// load
    Ld,
    /// store
    St,
    /// jump register
    Jsr,
    /// bitwise and
    And,
    /// load register
    Ldr,
    /// store register
    Str,
    /// unused
    Rti,
    /// bitwise not
    Not,
    /// load indirect
    Ldi,
    /// store indirect
    Sti,
    /// jump
    Jmp,
    /// reserved (unused)
    Res,
    /// load effective address
    Lea,
    /// execute trap
    Trap,
}

impl From<u16> for Opcode {
    fn from(opcode: u16) -> Self {
        match opcode {
            0 => Opcode::Br,
            1 => Opcode::Add,
            2 => Opcode::Ld,
            3 => Opcode::St,
            4 => Opcode::Jsr,
            5 => Opcode::And,
            6 => Opcode::Ldr,
            7 => Opcode::Str,
            8 => Opcode::Rti,
            9 => Opcode::Not,
            10 => Opcode::Ldi,
            11 => Opcode::Sti,
            12 => Opcode::Jmp,
            13 => Opcode::Res,
            14 => Opcode::Lea,
            15 => Opcode::Trap,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
enum Trap {
    /// get character from keyboard, not echoed onto the terminal
    Getc = 0x20,
    /// output a character
    Out = 0x21,
    /// output a word string
    Puts = 0x22,
    /// get character from keyboard, echoed onto the terminal
    In = 0x23,
    /// output a byte string
    Putsp = 0x24,
    /// halt the program
    Halt = 0x25,
}

impl From<u16> for Trap {
    fn from(trap: u16) -> Self {
        match trap {
            0x20 => Trap::Getc,
            0x21 => Trap::Out,
            0x22 => Trap::Puts,
            0x23 => Trap::In,
            0x24 => Trap::Putsp,
            0x25 => Trap::Halt,
            _ => unreachable!(),
        }
    }
}

enum MemoryMappedRegisters {
    /// Keyboard status
    Kbsr = 0xFE00,
    /// Keyboard data
    Kbdr = 0xFE02,
}
