use std::{
    fs::File,
    io::{self, prelude::*, BufReader},
    path::Path,
};

use byteorder::{BigEndian, ReadBytesExt};

fn main() -> Result<(), io::Error> {
    println!("Hello, world!");

    let mut lc3 = Lc3::default();
    lc3.load_image_file("rogue.obj")?;

    let mut running = true;
    while running {
        let instruction = lc3.memory[lc3.registers.pc as usize];
        dbg!(instruction >> 12);
        let opcode = Opcode::from(instruction >> 12);
        println!("{:?}", opcode);

        match opcode {
            Opcode::Br => {}
            Opcode::Add => {}
            Opcode::Ld => {}
            Opcode::St => {}
            Opcode::Jsr => {}
            Opcode::And => {}
            Opcode::Ldr => {}
            Opcode::Str => {}
            Opcode::Rti => {}
            Opcode::Not => {}
            Opcode::Ldi => {}
            Opcode::Sti => {}
            Opcode::Jmp => {}
            Opcode::Res => {}
            Opcode::Lea => {}
            Opcode::Trap => {
                running = false;
            }
        }

        lc3.registers.pc += 1;
    }

    Ok(())
}

/// 0x3000 is the default program counter starting position
const PC_START: u16 = 0x3000;

#[derive(Debug)]
struct Lc3 {
    memory: [u16; 65_536],
    registers: Register,
}

impl Lc3 {
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
        let mut origin = reader.read_u16::<BigEndian>()? as usize;

        while let Ok(p) = reader.read_u16::<BigEndian>() {
            self.memory[origin] = p;
            origin += 1;
        }

        Ok(())
    }
}

impl Default for Lc3 {
    fn default() -> Self {
        Self {
            memory: [0; 65_536],
            registers: Register::default(),
        }
    }
}

/// 8 general purpose registers (R0-R7)
/// 1 program counter (PC) register
/// 1 condition flags (COND) register
#[derive(Debug)]
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
            cond: ConditionalFlag::Pos,
        }
    }
}

#[derive(Debug)]
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
