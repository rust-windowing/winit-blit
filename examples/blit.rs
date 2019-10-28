use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_blit::{PixelBuffer, PixelBufferFormat};

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
                let mut bitmap = PixelBuffer::new(width, height, PixelBufferFormat::BGRA, &window).unwrap();
                for pixel in bitmap[..].chunks_mut(4) {
                    pixel.copy_from_slice(&[0, 76, 247, 255]);
                }

                bitmap.blit(&window).unwrap();
            },
            _ => *control_flow = ControlFlow::Wait,
        }
    });
}
