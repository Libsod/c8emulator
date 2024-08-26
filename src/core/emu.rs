pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const START_ADDR: u16 = 0x200;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Emu {
    pub pc: u16,
    pub ram: [u8; RAM_SIZE],
    pub screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub v_reg: [u8; NUM_REGS],
    pub i_reg: u16,
    pub sp: u16,
    pub stack: [u16; STACK_SIZE],
    pub keys: [bool; NUM_KEYS],
    pub dt: u8,
    pub st: u8,
    pub d1: u16,
    pub d2: u16,
    pub d3: u16,
    pub d4: u16,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
            d1: 0,
            d2: 0,
            d3: 0,
            d4: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.d1 = 0;
        self.d2 = 0;
        self.d3 = 0;
        self.d4 = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode & execute
        self.execute(op);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = u16::from(self.ram[self.pc as usize]);
        let lower_byte = u16::from(self.ram[(self.pc + 1) as usize]);
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        self.d1 = (op & 0xF000) >> 12;
        self.d2 = (op & 0x0F00) >> 8;
        self.d3 = (op & 0x00F0) >> 4;
        self.d4 = op & 0x000F;

        match (self.d1, self.d2, self.d3, self.d4) {
            // NOP
            (0, 0, 0, 0) => (),

            // CLS
            (0, 0, 0xE, 0) => self.clear_screen(),

            // RET
            (0, 0, 0xE, 0xE) => self.ret(),

            // JMP NNN
            (1, _, _, _) => self.jmp(op),

            // CALL NNN
            (2, _, _, _) => self.call(op),

            // SKIP VX == NN
            (3, _, _, _) => self.skip_vx_eq_nn(op),

            // SKIP VX != NN
            (4, _, _, _) => self.skip_vx_neq_nn(op),

            // SKIP VX == VY
            (5, _, _, _) => self.skip_vx_eq_vy(),

            // VX = NN
            (6, _, _, _) => self.set_vx_to_nn(op),

            // VX += NN
            (7, _, _, _) => self.add_vx_to_nn(op),

            // VX = VY
            (8, _, _, 0) => self.set_vx_to_vy(),

            // VX |= VY
            (8, _, _, 1) => self.vx_or_vy(),

            // VX &= VY
            (8, _, _, 2) => self.vx_and_vy(),

            // VX ^= VY
            (8, _, _, 3) => self.vx_xor_vy(),

            // VX += VY
            (8, _, _, 4) => self.add_vx_to_vy(),

            // VX -= VY
            (8, _, _, 5) => self.sub_vx_from_vy(),

            // VX >>= 1
            (8, _, _, 6) => self.shr_vx(),

            // VX = VY - VX
            (8, _, _, 7) => self.sub_vy_from_vx(),

            // VX <<= 1
            (8, _, _, 0xE) => self.shl_vx(),

            // SKIP VX != VY
            (9, _, _, 0) => self.skip_vx_neq_vy(),

            // I = NNN
            (0xA, _, _, _) => self.set_i_to_nnn(op),

            // JMP V0 + NNN
            (0xB, _, _, _) => self.jmp_v0_nnn(op),

            // VX = rand() & NN
            (0xC, _, _, _) => self.set_vx_to_rand(op),

            // DRAW
            (0xD, _, _, _) => self.draw(),

            // SKIP KEY PRESS
            (0xE, _, 9, 0xE) => self.skip_key_pressed(),

            // SKIP KEY RELEASE
            (0xE, _, 0xA, 1) => self.skip_key_released(),

            // VX = DT
            (0xF, _, 0, 7) => self.set_vx_to_dt(),

            // WAIT KEY
            (0xF, _, 0, 0xA) => self.wait_key(),

            // DT = VX
            (0xF, _, 1, 5) => self.set_dt_to_vx(),

            // ST = VX
            (0xF, _, 1, 8) => self.set_st_to_vx(),

            // I += VX
            (0xF, _, 1, 0xE) => self.add_i_to_vx(),

            // I = FONT
            (0xF, _, 2, 9) => self.set_i_to_font(),

            // BCD
            (0xF, _, 3, 3) => self.bcd(),

            // STORE V0 - VX
            (0xF, _, 5, 5) => self.store_sub_v0_from_vx(),

            // LOAD V0 - VX
            (0xF, _, 6, 5) => self.load_sub_v0_from_vx(),

            (_, _, _, _) => unimplemented!("Unimplemented opcode: {:#04x}", op),
        }
    }
}
