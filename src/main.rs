use std::fs::File;
use std::io::Read;
use raylib::consts::KeyboardKey;
use raylib::prelude::*;

const SCREEN_WIDTH:  usize = 64;
const SCREEN_HEIGHT: usize = 32;
const RAM_SIZE:      usize = 4096;
const NUM_REGS:      usize = 16;
const STACK_SIZE:    usize = 16;
const NUM_KEYS:      usize = 16;
const SCALE:         i32 = 10;

const FPS:           u32 = 60;

const FONT_SET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

struct Cpu {
    ram:     [u8; RAM_SIZE],
    v:       [u8; NUM_REGS],
    i:       u16,
    pc:      u16,
    sp:      u8,
    stack:   [u16; STACK_SIZE],
    st:      u8,
    dt:      u8,
    display: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    keys:    [bool; NUM_KEYS],
}

fn create_cpu() -> Cpu {
    let mut cpu = Cpu {
        ram:     [0; RAM_SIZE],
        v:       [0; NUM_REGS],
        i:       0,
        pc:      0x200,
        sp:      0,
        stack:   [0; STACK_SIZE],
        st:      0,
        dt:      0,
        display: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
        keys:    [false; NUM_KEYS],
    };

    for i in 0..80 {
        cpu.ram[i] = FONT_SET[i];
    }

    cpu
}

fn cpu_tick(cpu: &mut Cpu) {
        let opcode = (cpu.ram[cpu.pc as usize] as u16) << 8
            | (cpu.ram[(cpu.pc + 0x1) as usize] as u16);
        cpu.pc += 2;

        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let addr = opcode & 0x0FFF;

        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    //CLS
                    0x00E0 => { cpu.display = [0; SCREEN_WIDTH * SCREEN_HEIGHT]; }
                    //RET
                    0x00EE => {
                        if cpu.sp < 1 { return; }
                        cpu.sp -= 1;
                        cpu.pc = cpu.stack[cpu.sp as usize];
                    }
                    _ => {}
                }
            }
            //JP addr
            0x1000 => { cpu.pc = addr; }
            //CALL addr
            0x2000 => {
                if cpu.sp as usize >= STACK_SIZE { return; }
                cpu.stack[cpu.sp as usize] = cpu.pc;
                cpu.sp += 1;
                cpu.pc = addr;
            }
            //JP V0, addr
            0xB000 => { cpu.pc = addr + cpu.v[0] as u16; }
            //LD Vx, value
            0x6000 => { cpu.v[x] = kk; }
            //ADD Vx, byte
            0x7000 => { cpu.v[x] = cpu.v[x].wrapping_add(kk); }
            //LD I, addr
            0xA000 => { cpu.i = addr; }
            //SE Vx, byte
            0x3000 => { if cpu.v[x] == kk { cpu.pc += 2; } }
            //DRW Vx, Vy, N
            0xD000 => {
                let x_coord = cpu.v[x] as usize;
                let y_coord = cpu.v[y] as usize;
                let height = n;

                cpu.v[0xF] = 0;
                for row in 0..height {
                    let sprite_byte = cpu.ram[(cpu.i as usize) + row];
                    for col in 0..8 {
                        let sprite_pixel = (sprite_byte >> (7 - col)) & 1;
                        if sprite_pixel == 1 {
                            let px = (x_coord + col) % SCREEN_WIDTH;
                            let py = (y_coord + row) % SCREEN_HEIGHT;
                            let pixel_index = py * SCREEN_WIDTH + px;
                            if cpu.display[pixel_index] == 1 {
                                cpu.v[0xF] = 1;
                            }
                            cpu.display[pixel_index] ^= 1;
                        }
                    }
                }
            }
            //SNE Vx, byte
            0x4000 => { if cpu.v[x] != kk { cpu.pc += 2; } }
            //SE Vx, Vy
            0x5000 => { if cpu.v[x] == cpu.v[y] { cpu.pc += 2; } }
            //SNE Vx, Vy
            0x9000 => { if cpu.v[x] != cpu.v[y] { cpu.pc += 2; } }
            0x8000 => {
                match opcode & 0x000F {
                    //LD Vx, Vy
                    0 => { cpu.v[x] = cpu.v[y]; }
                    //OR Vx, Vy
                    1 => { cpu.v[x] |= cpu.v[y]; }
                    //AND Vx, Vy
                    2 => { cpu.v[x] &= cpu.v[y]; }
                    //XOR Vx, Vy
                    3 => { cpu.v[x] ^= cpu.v[y]; }
                    //ADD Vx, Vy
                    4 => {
                        let (res, carry) = cpu.v[x].overflowing_add(cpu.v[y]);
                        cpu.v[x] = res;
                        cpu.v[0xF] = if carry { 1 } else { 0 };
                    }
                    //SUB Vx, Vy
                    5 => {
                        let (res, borrow) = cpu.v[x].overflowing_sub(cpu.v[y]);
                        cpu.v[x] = res;
                        cpu.v[0xF] = if borrow { 0 } else { 1 };
                    }
                    //SHR Vx
                    6 => { cpu.v[0xF] = cpu.v[x] & 1; cpu.v[x] >>= 1; }
                    //SUBN Vx, Vy
                    7 => {
                        let (res, borrow) = cpu.v[y].overflowing_sub(cpu.v[x]);
                        cpu.v[x] = res;
                        cpu.v[0xF] = if borrow { 0 } else { 1 };
                    }
                    //SHL Vx
                    0xE => { cpu.v[0xF] = (cpu.v[x] >> 7) & 1; cpu.v[x] <<= 1; }
                    _ => {}
                }
            }
            //RND Vx, byte
            0xC000 => { cpu.v[x] = rand::random::<u8>() & kk; }
            0xE000 => {
                match opcode & 0x00FF {
                    //SKP Vx
                    0x9E => { if cpu.keys[cpu.v[x] as usize] { cpu.pc += 2; } }
                    //SKNP Vx
                    0xA1 => { if !cpu.keys[cpu.v[x] as usize] { cpu.pc += 2; } }
                    _ => {}
                }
            }
            0xF000 => {
                match opcode & 0x00FF {
                    //LD Vx, DT
                    0x07 => { cpu.v[x] = cpu.dt; }
                    //LD DT, Vx
                    0x15 => { cpu.dt = cpu.v[x]; }
                    //LD ST, Vx
                    0x18 => { cpu.st = cpu.v[x]; }
                    //ADD I, Vx
                    0x1E => { cpu.i += cpu.v[x] as u16; }
                    //LD F, Vx
                    0x29 => { cpu.i = cpu.v[x] as u16 * 5; }
                    //LD B, Vx
                    0x33 => {
                        let digit = cpu.v[x];
                        cpu.ram[cpu.i as usize] = digit / 100;
                        cpu.ram[cpu.i as usize + 1] = (digit / 10) % 10;
                        cpu.ram[cpu.i as usize + 2] = digit % 10;
                    }
                    //LD [I], Vx
                    0x55 => { for idx in 0..=x { cpu.ram[cpu.i as usize + idx] = cpu.v[idx]; } }
                    //LD Vx, [I]
                    0x65 => { for idx in 0..=x { cpu.v[idx] = cpu.ram[cpu.i as usize + idx]; } }
                    //LD Vx, K
                    0x0A => {
                        let mut any_key = false;
                        for i in 0..cpu.keys.len() {
                            if cpu.keys[i] {
                                any_key = true;
                                cpu.v[x] = i as u8;
                            }
                        }
                        if !any_key { cpu.pc -= 2; }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
}

fn main() {
    let mut cpu = create_cpu();

    let mut rom = File::open("pong.ch8").expect("ROM file not found");

    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer).unwrap();

    for (i, &byte) in buffer.iter().enumerate() {

        if 0x200 + i < RAM_SIZE {
            cpu.ram[0x200 + i] = byte;
        }
    }

    let (mut rl, thread) = raylib::init()
        .size((SCREEN_WIDTH as i32) * SCALE, (SCREEN_HEIGHT as i32) * SCALE)
        .title("chip-8")
        .build();
    rl.set_target_fps(FPS);

    while !rl.window_should_close() {

        cpu.keys[0x1] = rl.is_key_down(KeyboardKey::KEY_ONE);
        cpu.keys[0x2] = rl.is_key_down(KeyboardKey::KEY_TWO);
        cpu.keys[0x3] = rl.is_key_down(KeyboardKey::KEY_THREE);
        cpu.keys[0xC] = rl.is_key_down(KeyboardKey::KEY_FOUR);

        cpu.keys[0x4] = rl.is_key_down(KeyboardKey::KEY_Q);
        cpu.keys[0x5] = rl.is_key_down(KeyboardKey::KEY_W);
        cpu.keys[0x6] = rl.is_key_down(KeyboardKey::KEY_E);
        cpu.keys[0xD] = rl.is_key_down(KeyboardKey::KEY_R);

        cpu.keys[0x7] = rl.is_key_down(KeyboardKey::KEY_A);
        cpu.keys[0x8] = rl.is_key_down(KeyboardKey::KEY_S);
        cpu.keys[0x9] = rl.is_key_down(KeyboardKey::KEY_D);
        cpu.keys[0xE] = rl.is_key_down(KeyboardKey::KEY_F);

        cpu.keys[0xA] = rl.is_key_down(KeyboardKey::KEY_Z);
        cpu.keys[0x0] = rl.is_key_down(KeyboardKey::KEY_X);
        cpu.keys[0xB] = rl.is_key_down(KeyboardKey::KEY_C);
        cpu.keys[0xF] = rl.is_key_down(KeyboardKey::KEY_V);

        for _ in 0..10 {
            cpu_tick(&mut cpu);
        }
        if cpu.dt > 0 {
            cpu.dt -= 1;
        }
        if cpu.st > 0 {
            cpu.st -= 1;
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let index = y * SCREEN_WIDTH + x;
                if cpu.display[index] != 0 {
                    d.draw_rectangle(
                        (x as i32) * SCALE,
                        (y as i32) * SCALE,
                        SCALE,
                        SCALE,
                        Color::GREEN,
                    );
                }
            }
        }
    }
}

