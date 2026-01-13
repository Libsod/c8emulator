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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let emu = Emu::new();

        // Проверка счетчика команд
        assert_eq!(emu.pc, 0x200, "PC должен быть инициализирован на 0x200");

        // Проверка регистров
        for i in 0..16 {
            assert_eq!(emu.v_reg[i], 0, "Регистр V{} должен быть 0", i);
        }

        // Проверка регистра I
        assert_eq!(emu.i_reg, 0, "Регистр I должен быть 0");

        // Проверка стека
        assert_eq!(emu.sp, 0, "Указатель стека должен быть 0");

        // Проверка таймеров
        assert_eq!(emu.dt, 0, "Delay timer должен быть 0");
        assert_eq!(emu.st, 0, "Sound timer должен быть 0");

        // Проверка экрана
        for pixel in emu.screen.iter() {
            assert_eq!(*pixel, false, "Все пиксели должны быть выключены");
        }

        // Проверка загрузки шрифтов
        assert_eq!(emu.ram[0], 0xF0, "Первый байт шрифта '0' должен быть 0xF0");
        assert_eq!(emu.ram[5], 0x20, "Первый байт шрифта '1' должен быть 0x20");
    }

    #[test]
    fn test_load_rom() {
        let mut emu = Emu::new();

        // Создаем тестовый ROM
        let test_rom: Vec<u8> = vec![0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD];

        // Сохраняем первый байт шрифта для проверки
        let font_byte = emu.ram[0];

        // Загружаем ROM
        emu.load(&test_rom);

        // Проверяем, что ROM загружен правильно
        assert_eq!(emu.ram[0x200], 0x12, "Первый байт ROM");
        assert_eq!(emu.ram[0x201], 0x34, "Второй байт ROM");
        assert_eq!(emu.ram[0x202], 0x56, "Третий байт ROM");
        assert_eq!(emu.ram[0x203], 0x78, "Четвертый байт ROM");
        assert_eq!(emu.ram[0x204], 0xAB, "Пятый байт ROM");
        assert_eq!(emu.ram[0x205], 0xCD, "Шестой байт ROM");

        // Проверяем, что шрифты не были перезаписаны
        assert_eq!(emu.ram[0], font_byte, "Шрифты должны остаться неизменными");

        // Проверяем, что память после ROM остается нулевой
        assert_eq!(emu.ram[0x206], 0, "Память после ROM должна быть 0");
    }

    #[test]
    fn test_arithmetic_instructions() {
        let mut emu = Emu::new();

        // Тест ADD Vx, byte (7XNN)
        emu.v_reg[1] = 10;
        emu.execute(0x7105); // ADD V1, 5
        assert_eq!(emu.v_reg[1], 15, "V1 должен быть 15 после ADD");

        // Тест переполнения с wrapping
        emu.v_reg[2] = 255;
        emu.execute(0x7201); // ADD V2, 1
        assert_eq!(emu.v_reg[2], 0, "V2 должен быть 0 после переполнения");

        // Тест ADD Vx, Vy (8XY4) с флагом переноса
        emu.v_reg[3] = 200;
        emu.v_reg[4] = 100;
        emu.d1 = 8;
        emu.d2 = 3;
        emu.d3 = 4;
        emu.d4 = 4;
        emu.add_vx_to_vy();
        assert_eq!(emu.v_reg[3], 44, "V3 = 200 + 100 = 44 (с переносом)");
        assert_eq!(emu.v_reg[0xF], 1, "VF должен быть 1 (перенос)");

        // Тест SUB Vx, Vy (8XY5) без заема
        emu.v_reg[5] = 10;
        emu.v_reg[6] = 3;
        emu.d1 = 8;
        emu.d2 = 5;
        emu.d3 = 6;
        emu.d4 = 5;
        emu.sub_vx_from_vy();
        assert_eq!(emu.v_reg[5], 7, "V5 = 10 - 3 = 7");
        assert_eq!(emu.v_reg[0xF], 1, "VF должен быть 1 (нет заема)");

        // Тест SUB с заемом
        emu.v_reg[7] = 3;
        emu.v_reg[8] = 10;
        emu.d1 = 8;
        emu.d2 = 7;
        emu.d3 = 8;
        emu.d4 = 5;
        emu.sub_vx_from_vy();
        assert_eq!(emu.v_reg[7], 249, "V7 = 3 - 10 = 249 (с заемом)");
        assert_eq!(emu.v_reg[0xF], 0, "VF должен быть 0 (заем)");
    }

    #[test]
    fn test_draw_sprite() {
        let mut emu = Emu::new();

        // Подготовка тестового спрайта (крестик 3x3)
        let sprite: [u8; 3] = [
            0b01000000, // .#......
            0b11100000, // ###.....
            0b01000000, // .#......
        ];

        // Загружаем спрайт в память
        let sprite_addr = 0x300;
        for (i, byte) in sprite.iter().enumerate() {
            emu.ram[sprite_addr + i] = *byte;
        }

        // Устанавливаем координаты и адрес спрайта
        emu.v_reg[0] = 10; // X
        emu.v_reg[1] = 5; // Y
        emu.i_reg = sprite_addr as u16;

        // Отрисовываем спрайт
        emu.d1 = 0xD;
        emu.d2 = 0;
        emu.d3 = 1;
        emu.d4 = 3;
        emu.draw();

        // Проверяем, что пиксели установлены
        let idx1 = 11 + 5 * 64; // (11, 5) - верхний пиксель
        let idx2 = 10 + 6 * 64; // (10, 6) - левый пиксель средней строки
        let idx3 = 11 + 6 * 64; // (11, 6) - центральный пиксель
        let idx4 = 12 + 6 * 64; // (12, 6) - правый пиксель средней строки

        assert!(emu.screen[idx1], "Верхний пиксель должен быть включен");
        assert!(emu.screen[idx2], "Левый пиксель должен быть включен");
        assert!(emu.screen[idx3], "Центральный пиксель должен быть включен");
        assert!(emu.screen[idx4], "Правый пиксель должен быть включен");
        assert_eq!(emu.v_reg[0xF], 0, "VF должен быть 0 (нет столкновения)");

        // Отрисовываем спрайт еще раз (должен стереться)
        emu.draw();
        assert!(!emu.screen[idx1], "Пиксель должен быть выключен после XOR");
        assert_eq!(emu.v_reg[0xF], 1, "VF должен быть 1 (столкновение)");
    }

    #[test]
    fn test_stack_operations() {
        let mut emu = Emu::new();

        // Сохраняем начальный PC
        let initial_pc = emu.pc;

        // Тест CALL
        emu.execute(0x2345); // CALL 0x345
        assert_eq!(emu.sp, 1, "SP должен увеличиться на 1");
        assert_eq!(emu.stack[0], initial_pc, "В стеке должен быть сохранен PC");
        assert_eq!(emu.pc, 0x345, "PC должен указывать на 0x345");

        // Выполняем еще один CALL (вложенный)
        emu.execute(0x2567); // CALL 0x567
        assert_eq!(emu.sp, 2, "SP должен увеличиться до 2");
        assert_eq!(emu.stack[1], 0x345, "В стеке должен быть PC=0x345");
        assert_eq!(emu.pc, 0x567, "PC должен указывать на 0x567");

        // Тест RET (возврат из вложенного вызова)
        emu.execute(0x00EE); // RET
        assert_eq!(emu.sp, 1, "SP должен уменьшиться до 1");
        assert_eq!(emu.pc, 0x345, "PC должен вернуться к 0x345");

        // Еще один RET (возврат из первого вызова)
        emu.execute(0x00EE); // RET
        assert_eq!(emu.sp, 0, "SP должен вернуться к 0");
        assert_eq!(
            emu.pc, initial_pc,
            "PC должен вернуться к начальному значению"
        );
    }

    #[test]
    fn test_keyboard_operations() {
        let mut emu = Emu::new();

        // Устанавливаем клавишу
        emu.keypress(5, true);
        assert!(emu.keys[5], "Клавиша 5 должна быть нажата");

        // Тест skip if key pressed
        emu.v_reg[0] = 5;
        let pc_before = emu.pc;
        emu.d1 = 0xE;
        emu.d2 = 0;
        emu.d3 = 9;
        emu.d4 = 0xE;
        emu.skip_key_pressed();
        assert_eq!(emu.pc, pc_before + 2, "PC должен увеличиться на 2");

        // Тест skip if key not pressed
        emu.keypress(5, false);
        emu.v_reg[1] = 5;
        let pc_before = emu.pc;
        emu.d1 = 0xE;
        emu.d2 = 1;
        emu.d3 = 0xA;
        emu.d4 = 1;
        emu.skip_key_released();
        assert_eq!(emu.pc, pc_before + 2, "PC должен увеличиться на 2");

        // Тест wait for key (без нажатой клавиши)
        let pc_before = emu.pc;
        emu.d1 = 0xF;
        emu.d2 = 2;
        emu.d3 = 0;
        emu.d4 = 0xA;
        emu.wait_key();
        assert_eq!(emu.pc, pc_before - 2, "PC должен уменьшиться (повтор)");

        // Тест wait for key (с нажатой клавишей)
        emu.keypress(7, true);
        emu.pc = pc_before;
        emu.wait_key();
        assert_eq!(emu.v_reg[2], 7, "V2 должен содержать код клавиши 7");
        assert_eq!(emu.pc, pc_before, "PC не должен измениться");
    }

    #[test]
    fn test_timers() {
        let mut emu = Emu::new();

        emu.dt = 5;
        emu.st = 3;

        emu.tick_timers();
        assert_eq!(emu.dt, 4, "DT должен уменьшиться на 1");
        assert_eq!(emu.st, 2, "ST должен уменьшиться на 1");

        emu.tick_timers();
        emu.tick_timers();
        assert_eq!(emu.dt, 2, "DT должен стать 2");
        assert_eq!(emu.st, 0, "ST должен стать 0");

        emu.tick_timers();
        assert_eq!(emu.st, 0, "ST не должен уменьшаться ниже 0");
    }

    #[test]
    fn test_bitwise_instructions() {
        let mut emu = Emu::new();

        // OR (8XY1)
        emu.v_reg[0] = 0x0F;
        emu.v_reg[1] = 0xF0;
        emu.d1 = 8;
        emu.d2 = 0;
        emu.d3 = 1;
        emu.d4 = 1;
        emu.vx_or_vy();
        assert_eq!(emu.v_reg[0], 0xFF, "0x0F | 0xF0 должно быть 0xFF");

        // AND (8XY2)
        emu.v_reg[2] = 0x0F;
        emu.v_reg[3] = 0xF0;
        emu.d1 = 8;
        emu.d2 = 2;
        emu.d3 = 3;
        emu.d4 = 2;
        emu.vx_and_vy();
        assert_eq!(emu.v_reg[2], 0x00, "0x0F & 0xF0 должно быть 0x00");

        // XOR (8XY3)
        emu.v_reg[4] = 0xAA;
        emu.v_reg[5] = 0x55;
        emu.d1 = 8;
        emu.d2 = 4;
        emu.d3 = 5;
        emu.d4 = 3;
        emu.vx_xor_vy();
        assert_eq!(emu.v_reg[4], 0xFF, "0xAA ^ 0x55 должно быть 0xFF");

        // Shift Right (8XY6)
        emu.v_reg[6] = 0b0000_0111; // 7, LSB is 1
        emu.d1 = 8;
        emu.d2 = 6;
        emu.d3 = 0;
        emu.d4 = 6;
        emu.shr_vx();
        assert_eq!(emu.v_reg[6], 3, "7 >> 1 должно быть 3");
        assert_eq!(emu.v_reg[0xF], 1, "VF должен быть равен LSB (1)");

        // Shift Left (8XYE)
        emu.v_reg[7] = 0b1000_0001; // MSB is 1
        emu.d1 = 8;
        emu.d2 = 7;
        emu.d3 = 0;
        emu.d4 = 0xE;
        emu.shl_vx();
        assert_eq!(emu.v_reg[7], 0b0000_0010, "Shift left check");
        assert_eq!(emu.v_reg[0xF], 1, "VF должен быть равен MSB (1)");
    }
}
