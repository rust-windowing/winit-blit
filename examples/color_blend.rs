use winit::{
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_blit::{PixelBufferTyped, BGRA};

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Software rendering example")
        .build(&event_loop)
        .unwrap();

    let red = BGRA::from_rgb(255, 0, 0);
    let green = BGRA::from_rgb(0, 255, 0);
    let blue = BGRA::from_rgb(0, 0, 255);
    let alpha = BGRA::new(0, 0, 0, 255);
    let mut blend_mode = BlendMode::Approx;
    for i in 0..=255 {
        print!("{:x} ", bl(i, 0, 255));
    }
    event_loop.run(move |event, _, control_flow| {
        // println!("{:?}", event);

        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                window_id,
            } if window_id == window.id() => {
                blend_mode = match blend_mode {
                    BlendMode::Approx => BlendMode::Exact,
                    BlendMode::Exact => BlendMode::Naive,
                    BlendMode::Naive => BlendMode::Approx,
                };
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(window_id) => {
                if window_id == window.id() {
                    let (width, height): (u32, u32) = window.inner_size().into();
                    let mut buffer =
                        PixelBufferTyped::<BGRA>::new_supported(width, height, &window);
                    let start = std::time::Instant::now();

                    for (i, row) in buffer.rows_mut().enumerate() {
                        let y = ((i as f32 / height as f32) * 255.0).round() as u8;
                        let t_blend = blend_approx(y, red, green);
                        let b_blend = blend_approx(y, alpha, blue);
                        for (j, pixel) in row.into_iter().enumerate() {
                            // *pixel = x_blend;
                            let x = ((j as f32 / width as f32) * 255.0).round() as u8;
                            *pixel = blend_approx(x, t_blend, b_blend);
                        }
                    }
                    let end = std::time::Instant::now();
                    println!("{:?}", end - start);

                    buffer.blit(&window).unwrap();
                }
            }
            _ => *control_flow = ControlFlow::Wait,
        }
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlendMode {
    Exact,
    Approx,
    Naive,
}

fn blend(i: u8, a: BGRA, b: BGRA) -> BGRA {
    let i = i as f32 / 255.0;
    let a_f = 1.0 - i;
    let b_f = i;
    let bl = |a: u8, b: u8| {
        ((a_f * (a as f32 / 255.0).powf(2.2) + b_f * (b as f32 / 255.0).powf(2.2)).powf(1.0 / 2.2)
            * 255.0) as u8
    };

    BGRA {
        r: bl(a.r, b.r),
        g: bl(a.g, b.g),
        b: bl(a.g, b.b),
        a: bl(a.a, b.a),
    }
}

fn bl(f: u8, a: u8, b: u8) -> u8 {
    let a_linear = POWER_TABLE[a as usize] as u32; //a.pow(3) + 765 * a.pow(2);
    let b_linear = POWER_TABLE[b as usize] as u32; //b.pow(3) + 765 * b.pow(2);
    let f = f as u32;
    let a_f = 255 - f;
    let b_f = f;
    let val = ((a_f * a_linear + b_f * b_linear) / 255) as u16;
    // CORRECTION_TABLE[val as usize]
    interp_correction_table((val >> 8) as u8, val as u8)
}

static POWER_TABLE: &[u16] = &[
    0, 1, 2, 4, 7, 11, 17, 24, 32, 42, 53, 65, 79, 94, 111, 129, 148, 169, 192, 216, 242, 270, 299,
    330, 362, 396, 432, 469, 508, 549, 591, 635, 681, 729, 779, 830, 883, 938, 995, 1053, 1113,
    1175, 1239, 1305, 1373, 1443, 1514, 1587, 1663, 1740, 1819, 1900, 1983, 2068, 2155, 2243, 2334,
    2427, 2521, 2618, 2717, 2817, 2920, 3024, 3131, 3240, 3350, 3463, 3578, 3694, 3813, 3934, 4057,
    4182, 4309, 4438, 4570, 4703, 4838, 4976, 5115, 5257, 5401, 5547, 5695, 5845, 5998, 6152, 6309,
    6468, 6629, 6792, 6957, 7124, 7294, 7466, 7640, 7816, 7994, 8175, 8358, 8543, 8730, 8919, 9111,
    9305, 9501, 9699, 9900, 10102, 10307, 10515, 10724, 10936, 11150, 11366, 11585, 11806, 12029,
    12254, 12482, 12712, 12944, 13179, 13416, 13655, 13896, 14140, 14386, 14635, 14885, 15138,
    15394, 15652, 15912, 16174, 16439, 16706, 16975, 17247, 17521, 17798, 18077, 18358, 18642,
    18928, 19216, 19507, 19800, 20095, 20393, 20694, 20996, 21301, 21609, 21919, 22231, 22546,
    22863, 23182, 23504, 23829, 24156, 24485, 24817, 25151, 25487, 25826, 26168, 26512, 26858,
    27207, 27558, 27912, 28268, 28627, 28988, 29351, 29717, 30086, 30457, 30830, 31206, 31585,
    31966, 32349, 32735, 33124, 33514, 33908, 34304, 34702, 35103, 35507, 35913, 36321, 36732,
    37146, 37562, 37981, 38402, 38825, 39252, 39680, 40112, 40546, 40982, 41421, 41862, 42306,
    42753, 43202, 43654, 44108, 44565, 45025, 45487, 45951, 46418, 46888, 47360, 47835, 48313,
    48793, 49275, 49761, 50249, 50739, 51232, 51728, 52226, 52727, 53230, 53736, 54245, 54756,
    55270, 55787, 56306, 56828, 57352, 57879, 58409, 58941, 59476, 60014, 60554, 61097, 61642,
    62190, 62741, 63295, 63851, 64410, 64971, 65535,
];
static CORRECTION_TABLE: &[u8] = &[
    0, 21, 28, 34, 39, 43, 46, 50, 53, 56, 59, 61, 64, 66, 68, 70, 72, 74, 76, 78, 80, 82, 84, 85,
    87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 103, 105, 106, 107, 109, 110, 111, 112, 114, 115,
    116, 117, 118, 119, 120, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135,
    136, 137, 138, 139, 140, 141, 142, 143, 144, 144, 145, 146, 147, 148, 149, 150, 151, 151, 152,
    153, 154, 155, 156, 156, 157, 158, 159, 160, 160, 161, 162, 163, 164, 164, 165, 166, 167, 167,
    168, 169, 170, 170, 171, 172, 173, 173, 174, 175, 175, 176, 177, 178, 178, 179, 180, 180, 181,
    182, 182, 183, 184, 184, 185, 186, 186, 187, 188, 188, 189, 190, 190, 191, 192, 192, 193, 194,
    194, 195, 195, 196, 197, 197, 198, 199, 199, 200, 200, 201, 202, 202, 203, 203, 204, 205, 205,
    206, 206, 207, 207, 208, 209, 209, 210, 210, 211, 212, 212, 213, 213, 214, 214, 215, 215, 216,
    217, 217, 218, 218, 219, 219, 220, 220, 221, 221, 222, 223, 223, 224, 224, 225, 225, 226, 226,
    227, 227, 228, 228, 229, 229, 230, 230, 231, 231, 232, 232, 233, 233, 234, 234, 235, 235, 236,
    236, 237, 237, 238, 238, 239, 239, 240, 240, 241, 241, 242, 242, 243, 243, 244, 244, 245, 245,
    246, 246, 247, 247, 248, 248, 249, 249, 249, 250, 250, 251, 251, 252, 252, 253, 253, 254, 254,
    255, 255,
];

fn interp_correction_table(index: u8, val: u8) -> u8 {
    if index >= 56 {
        CORRECTION_TABLE[index as usize]
    } else {
        let a = CORRECTION_TABLE[index as usize] as u16;
        let b = CORRECTION_TABLE[index as usize + 1] as u16;
        let f = val as u16;
        let a_f = 255 - f;
        let b_f = f;
        ((a_f * a + b_f * b) / 255) as u8
    }
}

fn blend_approx(f: u8, a: BGRA, b: BGRA) -> BGRA {
    BGRA {
        r: bl(f, a.r, b.r),
        g: bl(f, a.g, b.g),
        b: bl(f, a.b, b.b),
        a: bl(f, a.a, b.a),
    }
}

fn bl_naive(f: u8, a: u8, b: u8) -> u8 {
    let (f, a, b) = (f as u64, a as u64, b as u64);
    let a_f = 255 - f;
    let b_f = f;
    ((a_f * a + b_f * b) / 255) as u8
}

fn blend_naive(f: u8, a: BGRA, b: BGRA) -> BGRA {
    BGRA {
        r: bl_naive(f, a.r, b.r),
        g: bl_naive(f, a.g, b.g),
        b: bl_naive(f, a.b, b.b),
        a: bl_naive(f, a.a, b.a),
    }
}
