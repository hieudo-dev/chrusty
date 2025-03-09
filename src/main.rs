#![allow(warnings)]
use std::os::unix::raw::gid_t;

use cssom::{CSSProperty, ColorData};
use engine::parse_to_layout;
use layout::{BoxType, Dimensions, LayoutBox, Rect};
use parser::{CSSParser, IParser};

use crate::utils::minify;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod cssom;
mod dom;
mod engine;
mod layout;
mod parser;
mod style;
mod utils;

fn main() {
    // Initialize logger (optional, for debugging)
    env_logger::init();

    // Create the event loop and window.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Chrusty")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    // Create a pixel buffer.
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(800, 600, surface_texture).unwrap();

    let mut layout_root = parse_to_layout(
        "
        <div id='1'>
            <div id='2'></div>
            <div class='text'></div>
        </div>
        ",
        "
        #1 {
            padding: 20px;
            background: rgb(20,200,20);
        }

        #2 {
            height: 100px;
            background: rgb(0,70,180);
        }

        .text {
            width: 100px;
            background: rgb(0,0,0);
            padding: 16px;
        }
        ",
    );

    layout_root.layout(&Dimensions {
        boundingRect: Rect {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        },
        padding: Default::default(),
        content: Default::default(),
    });

    let mut loaded = false;
    // Start the event loop.
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::RedrawRequested(_) => {
                if loaded {
                    return;
                }

                loaded = true;
                let frame = pixels.get_frame_mut();
                // Clear the frame with a white background.
                for pixel in frame.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
                }

                // Draw the layout tree onto the frame.
                draw_layout_box(frame, &layout_root, 800, 600);

                // Render the frame.
                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            _ => (),
        }
        // Request redraw on every iteration.
        window.request_redraw();
    });
}

/// Recursively draws a layout box and its children.
/// The color is chosen based on the box type.
fn draw_layout_box(frame: &mut [u8], layout_box: &LayoutBox, frame_width: u32, frame_height: u32) {
    let rect = &layout_box.dimensions.boundingRect;
    // Convert f32 to u32 (in a real engine you might want to round or scale).
    let x = rect.x as u32;
    let y = rect.y as u32;
    let width = rect.width as u32;
    let height = rect.height as u32;

    println!("{:?}", (rect));

    // Choose a color based on the box type.
    let (r, g, b) = match &layout_box.box_type {
        BoxType::Block(b) | BoxType::Inline(b) => {
            match b.get_computed_value(&CSSProperty::Background) {
                Some(cssom::CSSValue::Color(ColorData::Rgb(r, g, b))) => (r, g, b),
                x => panic!("error {:#?}", x),
            }
        }
        BoxType::Anonymous => (220, 220, 220), // light gray for anonymous containers
    };

    draw_rectangle(
        frame,
        x,
        y,
        width,
        height,
        r,
        g,
        b,
        frame_width,
        frame_height,
    );

    // Recursively draw children.
    for child in &layout_box.children {
        draw_layout_box(frame, child, frame_width, frame_height);
    }
}

/// Draws a filled rectangle into the frame buffer.
/// (x, y) is the top-left corner, and the color is provided as (r, g, b).
fn draw_rectangle(
    frame: &mut [u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    r: u8,
    g: u8,
    b: u8,
    frame_width: u32,
    frame_height: u32,
) {
    for j in y..(y + height) {
        for i in x..(x + width) {
            if i < frame_width && j < frame_height {
                let idx = ((j * frame_width + i) * 4) as usize;
                frame[idx] = r;
                frame[idx + 1] = g;
                frame[idx + 2] = b;
                frame[idx + 3] = 0xff; // opaque
            }
        }
    }
}
