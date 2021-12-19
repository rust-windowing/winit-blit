use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_blit::{NativeFormat, PixelBufferTyped};

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Software rendering example")
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        console_error_panic_hook::set_once();

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&window.canvas())
            .unwrap();
    }

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => *control_flow = ControlFlow::Exit,
        Event::RedrawRequested(window_id) => {
            if window_id == window.id() {
                let (width, height): (u32, u32) = window.inner_size().into();
                let mut buffer =
                    PixelBufferTyped::<NativeFormat>::new_supported(width, height, &window);

                for (i, row) in buffer.rows_mut().enumerate() {
                    let value = (i % 256) as u16;
                    for (j, pixel) in row.into_iter().enumerate() {
                        let value = value * (j % 256) as u16 / 256;
                        *pixel = NativeFormat::from_rgb(
                            (256 * value / 256) as u8,
                            (256 * value / 256) as u8,
                            (256 * value / 256) as u8,
                        );
                    }
                }

                buffer.blit(&window).unwrap();
            }
        }
        _ => *control_flow = ControlFlow::Wait,
    });
}
