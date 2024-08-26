use super::emu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use super::Emu;
use rand::Rng;

impl Emu {
    pub fn clear_screen(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    pub fn ret(&mut self) {
        self.pc = self.pop();
    }

    pub fn jmp(&mut self, op: u16) {
        self.pc = op & 0xFFF;
    }

    pub fn call(&mut self, op: u16) {
        self.push(self.pc);
        self.pc = op & 0xFFF;
    }

    pub fn skip_vx_eq_nn(&mut self, op: u16) {
        let x = self.d2 as usize;
        let nn = (op & 0xFF) as u8;
        if self.v_reg[x] == nn {
            self.pc += 2;
        }
    }

    pub fn skip_vx_neq_nn(&mut self, op: u16) {
        let x = self.d2 as usize;
        let nn = (op & 0xFF) as u8;
        if self.v_reg[x] != nn {
            self.pc += 2;
        }
    }

    pub fn skip_vx_eq_vy(&mut self) {
        let x = self.d2 as usize;
        let y = self.d3 as usize;
        if self.v_reg[x] == self.v_reg[y] {
            self.pc += 2;
        }
    }

    pub fn set_vx_to_nn(&mut self, op: u16) {
        let x = self.d2 as usize;
        let nn = (op & 0xFF) as u8;
        self.v_reg[x] = nn;
    }

    pub fn add_vx_to_nn(&mut self, op: u16) {
        let x = self.d2 as usize;
        let nn = (op & 0xFF) as u8;
        self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
    }

    pub fn set_vx_to_vy(&mut self) {
        let x = self.d2 as usize;
        let y = self.d3 as usize;
        self.v_reg[x] = self.v_reg[y];
    }

    pub fn vx_or_vy(&mut self) {
        let x = self.d2 as usize;
        let y = self.d3 as usize;
        self.v_reg[x] |= self.v_reg[y];
    }

    pub fn vx_and_vy(&mut self) {
        let x = self.d2 as usize;
        let y = self.d3 as usize;
        self.v_reg[x] &= self.v_reg[y];
    }

    pub fn vx_xor_vy(&mut self) {
        let x = self.d2 as usize;
        let y = self.d3 as usize;
        self.v_reg[x] ^= self.v_reg[y];
    }

    pub fn add_vx_to_vy(&mut self) {
        let x = self.d2 as usize;
        let y = self.d3 as usize;

        let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
        let new_vf = u8::from(carry);

        self.v_reg[x] = new_vx;
        self.v_reg[0xF] = new_vf;
    }

    pub fn sub_vx_from_vy(&mut self) {
        let x = self.d2 as usize;
        let y = self.d3 as usize;

        let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
        let new_vf = u8::from(!borrow);

        self.v_reg[x] = new_vx;
        self.v_reg[0xF] = new_vf;
    }

    pub fn shr_vx(&mut self) {
        let x = self.d2 as usize;
        let lsb = self.v_reg[x] & 1;
        self.v_reg[x] >>= 1;
        self.v_reg[0xF] = lsb;
    }

    pub fn sub_vy_from_vx(&mut self) {
        let x = self.d2 as usize;
        let y = self.d3 as usize;

        let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
        let new_vf = u8::from(!borrow);

        self.v_reg[x] = new_vx;
        self.v_reg[0xF] = new_vf;
    }

    pub fn shl_vx(&mut self) {
        let x = self.d2 as usize;
        let msb = (self.v_reg[x] >> 7) & 1;
        self.v_reg[x] <<= 1;
        self.v_reg[0xF] = msb;
    }

    pub fn skip_vx_neq_vy(&mut self) {
        let x = self.d2 as usize;
        let y = self.d3 as usize;
        if self.v_reg[x] != self.v_reg[y] {
            self.pc += 2;
        }
    }

    pub fn set_i_to_nnn(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.i_reg = nnn;
    }

    pub fn jmp_v0_nnn(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.pc = u16::from(self.v_reg[0]) + nnn;
    }

    pub fn set_vx_to_rand(&mut self, op: u16) {
        let x = self.d2 as usize;
        let nn = (op & 0xFF) as u8;
        let rng: u8 = rand::thread_rng().gen();
        self.v_reg[x] = rng & nn;
    }

    pub fn draw(&mut self) {
        // Get the (x, y) coords for our sprite
        let x_coord = u16::from(self.v_reg[self.d2 as usize]);
        let y_coord = u16::from(self.v_reg[self.d3 as usize]);
        // The last self.d determines how many rows high our sprite is
        let num_rows = self.d4;

        // Keep track if any pixels were flipped
        let mut flipped = false;
        // Iterate over each row of our sprite
        for y_line in 0..num_rows {
            // Determine which memory address our row's data is stored
            let addr = self.i_reg + y_line;
            let pixels = self.ram[addr as usize];
            // Iterate over each column in our row
            for x_line in 0..8 {
                // Use a mask to fetch current pixel's bit. Only flip if a 1
                if (pixels & (0b1000_0000 >> x_line)) != 0 {
                    // Sprites should wrap around screen, so apply modulo
                    let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                    let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                    // Get our pixel's index in the 1D screen array
                    let idx = x + SCREEN_WIDTH * y;
                    // Check if we're about to flip the pixel and set
                    flipped |= self.screen[idx];
                    self.screen[idx] ^= true;
                }
            }
        }
        // Populate VF register
        if flipped {
            self.v_reg[0xF] = 1;
        } else {
            self.v_reg[0xF] = 0;
        }
    }

    pub fn skip_key_pressed(&mut self) {
        let x = self.d2 as usize;
        let vx = self.v_reg[x];
        let key = self.keys[vx as usize];
        if key {
            self.pc += 2;
        }
    }

    pub fn skip_key_released(&mut self) {
        let x = self.d2 as usize;
        let vx = self.v_reg[x];
        let key = self.keys[vx as usize];
        if !key {
            self.pc += 2;
        }
    }

    pub fn set_vx_to_dt(&mut self) {
        let x = self.d2 as usize;
        self.v_reg[x] = self.dt;
    }

    pub fn wait_key(&mut self) {
        let x = self.d2 as usize;
        let mut pressed = false;
        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.v_reg[x] = i as u8;
                pressed = true;
                break;
            }
        }

        if !pressed {
            // Redo opcode
            self.pc -= 2;
        }
    }

    pub fn set_dt_to_vx(&mut self) {
        let x = self.d2 as usize;
        self.dt = self.v_reg[x];
    }

    pub fn set_st_to_vx(&mut self) {
        let x = self.d2 as usize;
        self.st = self.v_reg[x];
    }

    pub fn add_i_to_vx(&mut self) {
        let x = self.d2 as usize;
        let vx = u16::from(self.v_reg[x]);
        self.i_reg = self.i_reg.wrapping_add(vx);
    }

    pub fn set_i_to_font(&mut self) {
        let x = self.d2 as usize;
        let c = u16::from(self.v_reg[x]);
        self.i_reg = c * 5;
    }

    pub fn bcd(&mut self) {
        let x = self.d2 as usize;
        let vx = f32::from(self.v_reg[x]);

        // Fetch the hundreds self.d by dividing by 100 and tossing the decimal
        let hundreds = (vx / 100.0).floor() as u8;
        // Fetch the tens self.d by dividing by 10, tossing the ones self.d and the decimal
        let tens = ((vx / 10.0) % 10.0).floor() as u8;
        // Fetch the ones self.d by tossing the hundreds and the tens
        let ones = (vx % 10.0) as u8;

        self.ram[self.i_reg as usize] = hundreds;
        self.ram[(self.i_reg + 1) as usize] = tens;
        self.ram[(self.i_reg + 2) as usize] = ones;
    }

    pub fn store_sub_v0_from_vx(&mut self) {
        let x = self.d2 as usize;
        let i = self.i_reg as usize;
        for idx in 0..=x {
            self.ram[i + idx] = self.v_reg[idx];
        }
    }

    pub fn load_sub_v0_from_vx(&mut self) {
        let x = self.d2 as usize;
        let i = self.i_reg as usize;
        for idx in 0..=x {
            self.v_reg[idx] = self.ram[i + idx];
        }
    }
}
