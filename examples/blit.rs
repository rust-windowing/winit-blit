use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_blit::{PixelBufferTyped, NativeFormat};

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Software rendering example")
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        // println!("{:?}", event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let (width, height): (u32, u32) = window.inner_size().to_physical(window.hidpi_factor()).into();
                let mut buffer = PixelBufferTyped::<NativeFormat>::new_supported(width, height, &window);
                for pixel in buffer.rows_mut().flatten() {
                    *pixel = NativeFormat::from_rgb(247, 76, 0);
                }

                buffer.blit(&window).unwrap();
            },
            _ => *control_flow = ControlFlow::Wait,
        }
    });
}
