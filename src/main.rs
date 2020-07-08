use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
enum ProgramCounter {
    Next,
    Jump(usize)
}

// TODO: refactor, too generic to be useful
#[derive(Debug)]
enum Operand {
    Addr(u16),
    Byte(u8),
    Register(Register), // TODO: Register enum
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Register {
    V(usize),
    I,
    Dt,
    St,
    Pc,
    Sp,
}

#[derive(Debug)]
enum Op {
    CLS,                     // Clear
    RET,                     // Return
    JP(usize),             // Jump
    JPREG(Operand, Operand), // Jump
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

    DRAW(Operand, Operand, Operand),
    SKP(Operand),
    SKNP(Operand),

    SKIPKEY(Operand),
    SKIPNOKEY(Operand),
    WAITKEY(Operand),
    SPRITECHAR(Operand),
    MOVBCD(Operand),
    READM(Operand),
    WRITEM(Operand),
}

struct Mmu {
    ram: Vec<u8>,
    vram: Vec<u8>,
}

impl Mmu {
    fn new() -> Mmu {
        Mmu { ram: vec![0; 4096], vram: vec![0; 2048] }
    }

    fn read_byte(&self, index: usize) -> Result<u8, String> {
        if index > self.ram.len() {
            Err("unable to read byte".into())
        } else {
            Ok(self.ram[index])
        }
    }

    fn read_word(&self, index: usize) -> Result<u16, String> {
        if index + 1 > self.ram.len() {
            Err("unable to read word".into())
        } else {
            let word: u16 = ((self.ram[index] as u16) << 8) + self.ram[index + 1] as u16;
            Ok(word)
        }
    }

    fn load_rom(&mut self, rom: Vec<u8>) -> Result<(), String> {
        if rom.len() > 4096 {
            return Err("rom size too large".into());
        }
        // self.mem.clone_from_slice(&rom);
        for (i, item) in rom.iter().enumerate() {
            self.ram.insert(i + 0x200, *item);
        }
        // copy(&mut self.mem[..], &mut rom);
        // self.mem = rom;
        Ok(())
    }

    fn clear_vram(&mut self) {
        self.vram.clear();
    }
}

struct Cpu {
    mmu: Mmu,
    // general purpose registers
    v: [u8; 16],

    // address store register
    i: usize,

    stack: [usize; 16],
    pc: usize,
    sp: usize,
}

impl Cpu {
    fn new(mmu: Mmu) -> Cpu {
        Cpu {
            mmu,
            v: [0u8; 16],
            i: 0,
            stack: [0usize; 16],
            pc: 0x200,
            sp: 0,
        }
    }

    fn reg_from_nibble(&self, op: u8) -> Option<Register> {
        match op {
            0..=9 => Some(Register::V(op as usize)),
            _ => None,
        }
    }

    fn read_instruction(&mut self) -> Result<Op, String> {
        let op = self.mmu.read_word(self.pc)?;
        let high_nib: u8 = (op >> 12) as u8;

        match op {
            // op if high_nib == 0 => Ok(Op::SYS),
            op if op == 0x00E0 => Ok(Op::CLS),
            op if op == 0x00EE => Ok(Op::RET),
            op if high_nib == 0x1 => Ok(Op::JP((op & 0x0FFF) as usize)),
            op if high_nib == 0x2 => Ok(Op::CALL(Operand::Addr(op & 0x0FFF))),
            op if high_nib == 0x3 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::SE(Operand::Register(reg), Operand::Byte(op as u8)))
            }
            op if high_nib == 0x4 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::SNE(Operand::Register(reg), Operand::Byte(op as u8)))
            }
            op if high_nib == 0x5 => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::SE(Operand::Register(reg_a), Operand::Register(reg_b)))
            }
            op if high_nib == 0x6 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::LD(Operand::Register(reg), Operand::Byte(op as u8)))
            }
            op if high_nib == 0x7 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::ADD(Operand::Register(reg), Operand::Byte(op as u8)))
            }

            op if op & 0xF00F == 0x8000 => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::LD(Operand::Register(reg_a), Operand::Register(reg_b)))
            }
            op if op & 0xF00F == 0x8001 => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::OR(Operand::Register(reg_a), Operand::Register(reg_b)))
            }
            op if op & 0xF00F == 0x8002 => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::AND(Operand::Register(reg_a), Operand::Register(reg_b)))
            }
            op if op & 0xF00F == 0x8003 => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::XOR(Operand::Register(reg_a), Operand::Register(reg_b)))
            }
            op if op & 0xF00F == 0x8004 => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::ADD(Operand::Register(reg_a), Operand::Register(reg_b)))
            }
            op if op & 0xF00F == 0x8005 => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::SUB(Operand::Register(reg_a), Operand::Register(reg_b)))
            }
            op if op & 0xF00F == 0x8006 => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::SHR(Operand::Register(reg_a), Operand::Register(reg_b)))
            }
            op if op & 0xF00F == 0x8007 => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::SUBN(Operand::Register(reg_a), Operand::Register(reg_b)))
            }
            op if op & 0xF00F == 0x800E => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::SHL(Operand::Register(reg_a), Operand::Register(reg_b)))
            }
            op if high_nib == 0x9 => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::SNE(Operand::Register(reg_a), Operand::Register(reg_b)))
            }
            op if high_nib == 0xA => Ok(Op::LD(
                Operand::Register(Register::I),
                Operand::Addr(op & 0x0FFF),
            )),
            op if high_nib == 0xB => Ok(Op::JPREG(
                Operand::Register(Register::V(0)),
                Operand::Addr(op & 0x0FFF),
            )),
            op if high_nib == 0xC => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::RND(
                    Operand::Register(reg),
                    Operand::Byte(op as u8 & 0x00FF),
                ))
            }
            op if high_nib == 0xD => {
                let reg_a = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                let reg_b = self.reg_from_nibble((op >> 4 & 0xF) as u8).ok_or("can't read reg from nibble")?;
                let nibble = (op >> 4 & 0xF) & 0x00FF;
                Ok(Op::DRAW(
                    Operand::Register(reg_a),
                    Operand::Register(reg_b),
                    Operand::Byte(nibble as u8),
                ))
            }
            op if op & 0xF0FF == 0xE09E => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::SKIPKEY(Operand::Register(reg)))
            }
            op if op & 0xF0FF == 0xE0A1 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::SKIPNOKEY(Operand::Register(reg)))
            }
            op if op & 0xF0FF == 0xF007 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::LD(
                    Operand::Register(reg),
                    Operand::Register(Register::Dt),
                ))
            }
            op if op & 0xF0FF == 0xF00A => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::WAITKEY(Operand::Register(reg)))
            }
            op if op & 0xF0FF == 0xF015 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::LD(
                    Operand::Register(Register::Dt),
                    Operand::Register(reg),
                ))
            }
            op if op & 0xF0FF == 0xF018 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::LD(
                    Operand::Register(Register::St),
                    Operand::Register(reg),
                ))
            }
            op if op & 0xF0FF == 0xF01E => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::ADD(
                    Operand::Register(Register::I),
                    Operand::Register(reg),
                ))
            }
            op if op & 0xF0FF == 0xF029 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::SPRITECHAR(Operand::Register(reg)))
            }
            op if op & 0xF0FF == 0xF033 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::MOVBCD(Operand::Register(reg)))
            }
            op if op & 0xF0FF == 0xF055 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::READM(Operand::Register(reg)))
            }
            op if op & 0xF0FF == 0xF065 => {
                let reg = self.reg_from_nibble((op >> 8 & 0x0F) as u8).ok_or("can't read reg from nibble")?;
                Ok(Op::WRITEM(Operand::Register(reg)))
            }
            _ => {
                println!("unknown op {}", high_nib);
                Err(format!("can't handle op {:x?}", op))
            }
        }
    }

    fn execute_instruction(&mut self, instruction: Op) {
        let pc_change = match instruction {
            Op::CLS => self.clear_vram(),
            // Op::RET => {}
            Op::JP(dst) => {
                ProgramCounter::Jump(dst as usize)
            }
            // Op::JPREG(_, _) => {}
            // Op::CALL(_) => {}
            // Op::SE(_, _) => {}
            // Op::SNE(_, _) => {}
            Op::LD(dst, src) => {
                if let Operand::Register(dst) = dst {
                    if let Operand::Byte(src) = src {
                        match dst {
                            Register::V(dst) => { self.v[dst] = src; }
                            _ => panic!("can only load byte int Vx register"),
                        };
                    }
                    if let Operand::Addr(src) = src {
                        match dst {
                            Register::I => { self.i = src as usize }
                            _ => panic!("cannot load address unless dst is i"),
                        }
                    }
                }

                ProgramCounter::Next
            }
            // Op::ADD(_, _) => {}
            // Op::OR(_, _) => {}
            // Op::AND(_, _) => {}
            // Op::XOR(_, _) => {}
            // Op::SUB(_, _) => {}
            // Op::SHR(_, _) => {}
            // Op::SUBN(_, _) => {}
            // Op::SHL(_, _) => {}
            // Op::RND(_, _) => {}
            // Op::DRAW(_, _, _) => {}
            // Op::SKP(_) => {}
            // Op::SKNP(_) => {}
            // Op::SKIPKEY(_) => {}
            // Op::SKIPNOKEY(_) => {}
            // Op::WAITKEY(_) => {}
            // Op::SPRITECHAR(_) => {}
            // Op::MOVBCD(_) => {}
            // Op::READM(_) => {}
            // Op::WRITEM(_) => {}
            _ => ProgramCounter::Next,
        };

        match pc_change {
            ProgramCounter::Next => { self.pc += 2 }
            ProgramCounter::Jump(addr) => { self.pc = addr }
        }
    }

    fn clear_vram(&mut self) -> ProgramCounter {
        self.mmu.clear_vram();
        ProgramCounter::Next
    }
}

fn main() {
    let mut args = env::args();
    args.next();
    let fp = args.next().unwrap();
    let mut file = File::open(fp).unwrap();
    let mut rom: Vec<u8> = Vec::with_capacity(100);
    file.read_to_end(&mut rom).unwrap();

    let mut mmu = Mmu::new();
    mmu.load_rom(rom).expect("failed to load rom");

    let mut cpu = Cpu::new(mmu);

    loop {
        match cpu.read_instruction() {
            Ok(op) => {
                println!("{:#x}\t{:?}", cpu.pc, op);
                cpu.pc += 2;
                // cpu.execute_instruction(op);
            }
            Err(s) => {
                println!("error reading instruction {}", s);
                break;
            }
        }
    }

    println!("Program exited.")
}
