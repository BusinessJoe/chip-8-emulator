use std::fs::File;
use std::io::Read;

macro_rules! join_nibbles {
    ($r0:ident) => {
        u8::from($r0)
    };
    ($r0:ident, $r1:ident) => {
        (u8::from($r0) << 4) | (u8::from($r1))
    };
    ($r0:ident, $r1:ident, $r2:ident) => {
        (u16::from($r0) << 8) | (u16::from($r1) << 4) | (u16::from($r2))
    };
    ($r0:ident, $r1:ident, $r2:ident, $r3:ident) => {
        (u16::from($r0) << 12) | (u16::from($r1) << 8) | (u16::from($r2) << 4) | (u16::from($r3))
    };
}

pub struct Chip8Emulator {
    // The Chip 8 has 4k memory.
    // Bytes 0x000 to 0x1FF were used to store the Chip 8 intepreter itself, but this isn't needed
    // with modern emulators, so this range is usually used to store font data now.
    // The uppermost 256 bytes (0xF00 to 0xFFF) are reserved for display refresh?
    // Bytes 0xEA0 to 0xEFF were reserved for the call stack, internal use, and other variables?
    memory: [u8; 4096],
    // It has 15 8-bit general purpose registers named V0, V1, ..., VE.
    // The 16th VF register is used for the "carry flag" and other instruction specific flags.
    V: [u8; 16],
    // There is an address register I and a program counter pc which range from 0x000 to 0xFFF (12
    // bits).
    I: usize,
    pc: usize,

    // The Chip 8 has 64 by 32 screen of black and white pixels.
    pub screen: [bool; 64 * 32],

    // The Chip 8 has two timer registers which count at 60hz.
    // When set above zero they count back down to zero.
    // The system's buzzer sounds whenever the sound timer reaches zero.
    delay_timer: u8,
    sound_timer: u8,

    // Stack and stack pointer. Used to store locations when jumping or calling a subroutine.
    stack: [u16; 16],
    sp: usize,

    // The Chip 8 uses a hex keyboard for input. This has 16 keys ranging from '0' to 'F'.
    // We can use a boolean array to store the state of each key.
    keys: [bool; 16],
}

const chip8_fontset: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl Chip8Emulator {
    pub fn new() -> Self {
        Self {
            memory: [0; 4096],
            V: [0; 16],
            I: 0,
            pc: 0,
            screen: [false; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            keys: [false; 16],
        }
    }

    /// Initialize memory and registers
    pub fn initialize(&mut self) {
        self.pc = 0x200;
        self.I = 0;
        self.sp = 0;

        self.screen = [false; 64 * 32];
        self.stack = [0; 16];
        self.V = [0; 16];
        self.memory = [0; 4096];

        // Load fontset
        for i in 0..80 {
            self.memory[i] = chip8_fontset[i];
        }

        // Reset timers
        self.delay_timer = 0;
        self.sound_timer = 0;
    }

    pub fn load_game(&mut self, game_name: &str) -> std::io::Result<()> {
        let file = File::open(game_name)?;
        for (i, byte) in file.bytes().enumerate() {
            self.memory[i + 0x200] = byte.unwrap();
        }
        Ok(())
    }

    fn split_opcode(value: u16) -> (u8, u8, u8, u8) {
        (
            ((value >> 12) & 0xF).try_into().unwrap(),
            ((value >> 8) & 0xF).try_into().unwrap(),
            ((value >> 4) & 0xF).try_into().unwrap(),
            ((value >> 0) & 0xF).try_into().unwrap(),
        )
    }

    pub fn emulate_cycle(&mut self) {
        // Fetch and execute opcode
        let opcode_value = u16::from(self.memory[usize::from(self.pc)]) << 8
            | u16::from(self.memory[usize::from(self.pc + 1)]);

        // println!("pc: {:#X}, opcode: {:#06X}, stack: {:?}", self.pc, opcode_value, self.stack);

        match Self::split_opcode(opcode_value) {
            (0x0, 0x0, 0xE, 0x0) => self.clear_screen(),
            (0x0, 0x0, 0xE, 0xE) => self.return_subroutine(),
            (0x0, r2, r1, r0) => self.machine_code_routine(join_nibbles!(r2, r1, r0)),
            (0x1, r2, r1, r0) => self.goto(join_nibbles!(r2, r1, r0)),
            (0x2, r2, r1, r0) => self.call_subroutine(join_nibbles!(r2, r1, r0)),
            (0x3, x, c1, c0) => self.skip_const_eq(x, join_nibbles!(c1, c0)),
            (0x4, x, c1, c0) => self.skip_const_neq(x, join_nibbles!(c1, c0)),
            (0x5, x, y, 0x0) => self.skip_reg_eq(x, y),
            (0x6, x, c1, c0) => self.set_const(x, join_nibbles!(c1, c0)),
            (0x7, x, c1, c0) => self.add_const(x, join_nibbles!(c1, c0)),
            (0x8, x, y, 0x0) => self.set(x, y),
            (0x8, x, y, 0x1) => self.or(x, y),
            (0x8, x, y, 0x2) => self.and(x, y),
            (0x8, x, y, 0x3) => self.xor(x, y),
            (0x8, x, y, 0x4) => self.add(x, y),
            (0x8, x, y, 0x5) => self.sub(x, y),
            (0x8, x, y, 0x6) => self.div_2(x, y),
            (0x8, x, y, 0x7) => self.diff(x, y),
            (0x8, x, y, 0xE) => self.mul_2(x, y),
            (0x9, x, y, 0x0) => self.skip_reg_neq(x, y),
            (0xA, r2, r1, r0) => self.set_i(join_nibbles!(r2, r1, r0)),
            (0xB, r2, r1, r0) => self.jump_offset(join_nibbles!(r2, r1, r0)),
            (0xC, x, c1, c0) => self.rand(x, join_nibbles!(c1, c0)),
            (0xD, x, y, c) => self.draw(x, y, c),
            (0xE, x, 0x9, 0xE) => self.skip_if_key(x),
            (0xE, x, 0xA, 0x1) => self.skip_if_nkey(x),
            (0xF, x, 0x0, 0x7) => self.get_delay(x),
            (0xF, x, 0x0, 0xA) => self.get_key(x),
            (0xF, x, 0x1, 0x5) => self.set_delay(x),
            (0xF, x, 0x1, 0x8) => self.set_sound(x),
            (0xF, x, 0x1, 0xE) => self.inc_i(x),
            (0xF, x, 0x2, 0x9) => self.set_i_sprite(x),
            (0xF, x, 0x3, 0x3) => self.bcd(x),
            (0xF, x, 0x5, 0x5) => self.reg_dump(x),
            (0xF, x, 0x6, 0x5) => self.reg_load(x),
            _ => panic!("{:#06X} is not a recognized opcode (pc: {:#X})", opcode_value, self.pc),
        }

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            if self.sound_timer == 0 {
                println!("BEEP");
            }
        }
    }

    pub fn set_keys(&self) {}

    fn clear_screen(&mut self) {
        todo!()
    }

    fn return_subroutine(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp].into();
        self.pc += 2;
    }

    fn machine_code_routine(&mut self, address: u16) {
        todo!()
    }

    fn goto(&mut self, address: u16) {
        self.pc = usize::from(address);
    }

    fn call_subroutine(&mut self, address: u16) {
        self.stack[self.sp] = self.pc.try_into().unwrap();
        self.sp += 1;
        self.pc = address.into();
    }

    fn skip_const_eq(&mut self, reg: u8, c: u8) {
        if (self.V[usize::from(reg)] == c) {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn skip_const_neq(&mut self, reg: u8, c: u8) {
        if (self.V[usize::from(reg)] != c) {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn skip_reg_eq(&mut self, reg: u8, reg2: u8) {
        todo!()
    }

    fn set_const(&mut self, reg: u8, c: u8) {
        self.V[usize::from(reg)] = c;
        self.pc += 2;
    }

    fn add_const(&mut self, reg: u8, c: u8) {
        let reg = usize::from(reg);
        self.V[reg] = self.V[reg].overflowing_add(c).0;
        self.pc += 2;
    }

    fn set(&mut self, reg: u8, reg2: u8) {
        let reg = usize::from(reg);
        let reg2 = usize::from(reg2);
        self.V[reg] = self.V[reg2];
        self.pc += 2;
    }

    fn or(&mut self, reg: u8, reg2: u8) {
        let reg = usize::from(reg);
        let reg2 = usize::from(reg2);
        self.V[reg] |= self.V[reg2];
        self.pc += 2;
    }

    fn and(&mut self, reg: u8, reg2: u8) {
        let reg = usize::from(reg);
        let reg2 = usize::from(reg2);
        self.V[reg] &= self.V[reg2];
        self.pc += 2;
    }

    fn xor(&mut self, reg: u8, reg2: u8) {
        todo!()
    }

    fn add(&mut self, reg: u8, reg2: u8) {
        let reg = usize::from(reg);
        let reg2 = usize::from(reg2);
        match self.V[reg].overflowing_add(self.V[reg2]) {
            (val, false) => {
                self.V[reg] = val;
                self.V[0xF] = 0;
            },
            (val, true) => {
                self.V[reg] = val;
                self.V[0xF] = 1;
            }
        }
        self.pc += 2;
    }

    fn sub(&mut self, reg: u8, reg2: u8) {
        let reg = usize::from(reg);
        let reg2 = usize::from(reg2);
        match self.V[reg].overflowing_sub(self.V[reg2]) {
            (val, false) => {
                self.V[reg] = val;
                self.V[0xF] = 0;
            },
            (val, true) => {
                self.V[reg] = val;
                self.V[0xF] = 1;
            }
        }
        self.pc += 2;
    }

    fn div_2(&mut self, reg: u8, _reg2: u8) {
        todo!()
    }

    fn diff(&mut self, reg: u8, reg2: u8) {
        todo!()
    }

    fn mul_2(&mut self, reg: u8, _reg2: u8) {
        todo!()
    }

    fn skip_reg_neq(&mut self, reg: u8, reg2: u8) {
        let reg = usize::from(reg);
        let reg2 = usize::from(reg2);
        if (self.V[reg] != self.V[reg2]) {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn set_i(&mut self, address: u16) {
        self.I = address.into();
        self.pc += 2;
    }

    fn jump_offset(&mut self, address: u16) {
        todo!()
    }

    fn rand(&mut self, reg: u8, c: u8) {
        self.V[usize::from(reg)] = rand::random::<u8>() & c;
        self.pc += 2;
    }

    fn draw(&mut self, x: u8, y: u8, height: u8) {
        let x = usize::from(self.V[usize::from(x)]);
        let y = usize::from(self.V[usize::from(y)]);
        let height = usize::from(height);

        self.V[0xF] = 0;
        for yline in 0..height {
            let pixel = self.memory[self.I + yline];
            for xline in 0..8 {
                if (pixel & (0x80 >> xline)) != 0 {
                    let idx = x + xline + ((y + yline) * 64);
                    if self.screen[idx] {
                        self.V[0xF] = 1;
                    }
                    self.screen[idx] ^= true;
                }
            }
        }

        self.pc += 2;
    }

    fn skip_if_key(&mut self, reg: u8) {
        todo!()
    }

    fn skip_if_nkey(&mut self, reg: u8) {
        if !self.keys[usize::from(self.V[usize::from(reg)])] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn get_delay(&mut self, reg: u8) {
        self.V[usize::from(reg)] = self.delay_timer;
        self.pc += 2;
    }

    fn get_key(&mut self, reg: u8) {
        todo!()
    }

    fn set_delay(&mut self, reg: u8) {
        let reg = usize::from(reg);
        self.delay_timer = self.V[reg];
        self.pc += 2;
    }

    fn set_sound(&mut self, reg: u8) {
        let reg = usize::from(reg);
        self.sound_timer = self.V[reg];
        self.pc += 2;
    }

    fn inc_i(&mut self, reg: u8) {
        todo!()
    }

    fn set_i_sprite(&mut self, reg: u8) {
        self.I = (reg * 5).into();
        self.pc += 2;
    }

    fn bcd(&mut self, reg: u8) {
        let reg = usize::from(reg);
        self.memory[self.I] = self.V[reg] / 100;
        self.memory[self.I + 1] = (self.V[reg] / 10) % 10;
        self.memory[self.I + 2] = (self.V[reg] % 100) % 10;
        self.pc += 2;
    }

    fn reg_dump(&mut self, reg: u8) {
        todo!()
    }

    fn reg_load(&mut self, reg: u8) {
        let reg = usize::from(reg);
        for i in 0..=reg {
            self.V[i] = self.memory[self.I + i];
        }
        self.pc += 2;
    }
}

