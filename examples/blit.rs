use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState},
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
                event: WindowEvent::KeyboardInput{input: KeyboardInput{state: ElementState::Pressed, ..}, ..},
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
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let (width, height): (u32, u32) = window.inner_size().to_physical(window.hidpi_factor()).into();
                let mut buffer = PixelBufferTyped::<BGRA>::new_supported(width, height, &window);
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
            },
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
    let bl = |a: u8, b: u8| ((a_f * (a as f32 / 255.0).powf(2.2) + b_f * (b as f32 / 255.0).powf(2.2)).powf(1.0/2.2) * 255.0) as u8;

    BGRA {
        r: bl(a.r, b.r),
        g: bl(a.g, b.g),
        b: bl(a.g, b.b),
        a: bl(a.a, b.a),
    }
}

fn bl(f: u8, a: u8, b: u8) -> u8 {
    let a_linear = POWER_TABLE[a as usize] as u32;//a.pow(3) + 765 * a.pow(2);
    let b_linear = POWER_TABLE[b as usize] as u32;//b.pow(3) + 765 * b.pow(2);
    let f = f as u32;
    let a_f = 255 - f;
    let b_f = f;
    let val = (
        (
            a_f * a_linear +
            b_f * b_linear
        ) / (64770 >> 8)
    ) as u16;
    // CORRECTION_TABLE[val as usize]
    interp_correction_table((val >> 8) as u8, val as u8)
}

static POWER_TABLE: &[u16] = &[
    0, 1, 2, 6, 12, 18, 27, 36, 48, 61, 75, 91, 109, 128, 149, 171, 195, 220, 247, 276, 306, 338,
    371, 407, 443, 482, 522, 563, 607, 652, 698, 747, 797, 848, 901, 957, 1013, 1072, 1132, 1194,
    1257, 1323, 1390, 1458, 1529, 1601, 1675, 1751, 1829, 1908, 1989, 2072, 2157, 2243, 2332, 2422,
    2514, 2608, 2703, 2801, 2900, 3001, 3104, 3209, 3316, 3424, 3534, 3647, 3761, 3877, 3995, 4115,
    4237, 4361, 4486, 4614, 4743, 4875, 5008, 5143, 5281, 5420, 5561, 5704, 5850, 5997, 6146, 6297,
    6450, 6605, 6763, 6922, 7083, 7246, 7412, 7579, 7749, 7920, 8093, 8269, 8447, 8627, 8808, 8992,
    9178, 9366, 9557, 9749, 9944, 10140, 10339, 10540, 10743, 10948, 11155, 11365, 11576, 11790,
    12006, 12224, 12445, 12667, 12892, 13119, 13348, 13580, 13813, 14049, 14288, 14528, 14770,
    15015, 15263, 15512, 15764, 16018, 16274, 16532, 16793, 17056, 17322, 17590, 17860, 18132,
    18407, 18684, 18963, 19245, 19529, 19816, 20104, 20396, 20689, 20985, 21284, 21584, 21888,
    22193, 22501, 22812, 23125, 23440, 23757, 24078, 24400, 24725, 25053, 25383, 25715, 26050,
    26388, 26728, 27070, 27415, 27762, 28112, 28465, 28820, 29177, 29537, 29900, 30265, 30633,
    31003, 31376, 31751, 32129, 32510, 32893, 33279, 33667, 34058, 34452, 34848, 35246, 35648,
    36052, 36459, 36868, 37280, 37695, 38112, 38532, 38955, 39380, 39808, 40239, 40673, 41109,
    41548, 41989, 42434, 42881, 43330, 43783, 44238, 44696, 45157, 45621, 46087, 46556, 47028,
    47503, 47980, 48461, 48944, 49429, 49918, 50410, 50904, 51401, 51901, 52404, 52910, 53419,
    53930, 54445, 54962, 55482, 56005, 56531, 57060, 57591, 58126, 58663, 59204, 59747, 60294,
    60843, 61395, 61950, 62508, 63069, 63633, 64200, 64770,
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
