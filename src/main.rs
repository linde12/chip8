use std::collections::HashMap;

#[derive(Debug)]
enum Operand {
    Addr(u16),
    Byte(u8),
    Register(Register), // TODO: Register enum
}

#[derive(Debug)]
pub enum Register {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    Va,
    Vb,
    Vc,
    Vd,
    Ve,
    Vf,

    I,

    Dt,
    St,

    Pc,
    Sp,
}

#[derive(Debug)]
enum Op {
    CLS,         // Clear
    RET,         // Return
    JP(Operand), // Jump
    CALL(Operand),
    SE(Operand, Operand),  // Skip next if eq
    SNE(Operand, Operand), // Skip next if not eq
    LD(Operand, Operand),

    ADD(Operand, Operand),
    OR(Operand, Operand),
    AND(Operand, Operand),
    XOR(Operand, Operand),
    SUB(Operand, Operand),
    SHR(Operand, Operand),
    SUBN(Operand, Operand),
    SHL(Operand, Operand),
    RND(Operand, Operand),

    DRW(Operand, Operand, Operand),
    SKP(Operand),
    SKNP(Operand),
}

struct Mmu {
    mem: Vec<u8>,
}

impl Mmu {
    fn new() -> Mmu {
        Mmu { mem: Vec::new() }
    }

    fn read_byte(&self, index: usize) -> Result<u8, ()> {
        if index > self.mem.len() {
            Err(())
        } else {
            Ok(self.mem[index])
        }
    }

    fn read_word(&self, index: usize) -> Result<u16, ()> {
        if index + 1 > self.mem.len() {
            Err(())
        } else {
            let word: u16 = ((self.mem[index] as u16) << 8) + self.mem[index + 1] as u16;
            Ok(word)
        }
    }

    fn load_rom(&mut self, rom: Vec<u8>) -> Result<(), ()> {
        if rom.len() > 4096 {
            return Err(());
        }
        self.mem = rom;
        Ok(())
    }
}

struct Cpu {
    mmu: Mmu,
    reg8: HashMap<u8, u8>,
    pc: usize,
}

impl Cpu {
    fn new(mmu: Mmu) -> Cpu {
        Cpu {
            mmu: mmu,
            reg8: HashMap::new(),
            pc: 0,
        }
    }

    fn reg_from_nibble(&self, op: u8) -> Option<Register> {
        match op {
            0 => Some(Register::V0),
            1 => Some(Register::V1),
            2 => Some(Register::V2),
            3 => Some(Register::V3),
            4 => Some(Register::V4),
            5 => Some(Register::V5),
            6 => Some(Register::V6),
            7 => Some(Register::V7),
            8 => Some(Register::V8),
            9 => Some(Register::V9),
            _ => None,
        }
    }

    fn read_instruction(&mut self) -> Result<Op, ()> {
        let op = self.mmu.read_word(self.pc)?;
        self.pc += 2;

        let high_nib: u8 = (op >> 12) as u8;

        match op {
            op if high_nib == 0x1 => Ok(Op::JP(Operand::Addr(0xFFFF))),
            op if high_nib == 0x2 => Ok(Op::CALL(Operand::Addr(0xFFFF))),
            op if high_nib == 0x3 => {
                if let Some(reg) = self.reg_from_nibble((op >> 8) as u8) {
                    return Ok(Op::SE(Operand::Register(reg), Operand::Byte(0xFFFF)));
                }
                Err(())
            }
            op if high_nib == 0x4 => Ok(Op::SNE(
                Operand::Register(Register::V0),
                Operand::Byte(0xFFFF),
            )),
            op if high_nib == 0x5 => Ok(Op::SE(
                Operand::Register(Register::V0),
                Operand::Register(Register::V1),
            )),
            op if high_nib == 0x6 => Ok(Op::LD(
                Operand::Register(Register::V0),
                Operand::Register(Register::V1),
            )),
            op if high_nib == 0x7 => Ok(Op::ADD(
                Operand::Register(Register::V0),
                Operand::Byte(0xFFFF),
            )),

            op if op & 0xF00F == 0x8001 => Ok(Op::OR(
                Operand::Register(Register::V0),
                Operand::Register(Register::V1),
            )),
            op if op & 0xF00F == 0x8002 => Ok(Op::AND(
                Operand::Register(Register::V0),
                Operand::Register(Register::V1),
            )),
            op if op & 0xF00F == 0x8003 => Ok(Op::XOR(
                Operand::Register(Register::V0),
                Operand::Register(Register::V1),
            )),
            op if op & 0xF00F == 0x8004 => Ok(Op::ADD(
                Operand::Register(Register::V0),
                Operand::Register(Register::V1),
            )),
            op if op & 0xF00F == 0x8005 => Ok(Op::SUB(
                Operand::Register(Register::V0),
                Operand::Register(Register::V1),
            )),
            op if op & 0xF00F == 0x8006 => Ok(Op::SHR(
                Operand::Register(Register::V0),
                Operand::Register(Register::V1),
            )),
            op if op & 0xF00F == 0x8007 => Ok(Op::SUBN(
                Operand::Register(Register::V0),
                Operand::Register(Register::V1),
            )),
            op if op & 0xF00F == 0x800E => Ok(Op::SHL(
                Operand::Register(Register::V0),
                Operand::Register(Register::V1),
            )),
            op if high_nib == 0x9 => Ok(Op::SNE(
                Operand::Register(Register::V0),
                Operand::Register(Register::V1),
            )),
            op if high_nib == 0xA => Ok(Op::LD(
                Operand::Register(Register::I),
                Operand::Addr(0xFFFF),
            )),
            op if high_nib == 0xB => Ok(Op::JP(Operand::Addr(0xFFFA))),
            _ => Err(()),
        }
    }
}

fn main() {
    let rom = vec![0x10, 0x00, 0x20, 0x10];

    let mut mmu = Mmu::new();
    mmu.load_rom(rom).expect("failed to load rom");

    let mut cpu = Cpu::new(mmu);

    loop {
        match cpu.read_instruction() {
            Ok(op) => println!("{:?}", op),
            Err(_) => break, // TODO: inform user
        }
    }

    println!("Program exited.")
}
