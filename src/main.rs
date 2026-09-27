use beryllium::{
    Sdl,
    events::Event,
    init::InitFlags,
    video::{CreateWinArgs, GlProfile},
};
use glow::*;
use std::ffi::{CString, c_void};
fn main() {
    let sdl = Sdl::init(InitFlags::EVERYTHING);
    // for info in sdl.get_renderer_driver_infos().unwrap() {
    //     println!("RendererDriver: {info:?}");
    // }

    #[cfg(not(target_os = "macos"))]
    {
        sdl.set_gl_profile(GlProfile::ES).unwrap();
        sdl.set_gl_context_major_version(3).unwrap();
        sdl.set_gl_context_minor_version(1).unwrap();
    }
    let win = sdl
        .create_gl_window(CreateWinArgs {
            title: "cube",
            width: 800,
            height: 600,
            allow_high_dpi: true,
            borderless: false,
            resizable: false,
        })
        .unwrap();

    println!("GL window size: {:?}", win.get_window_size());
    println!("GL drawable size: {:?}", win.get_drawable_size());
    println!(
        "GL_KHR_debug supported: {}",
        win.supports_extension("GL_KHR_debug")
    );

    // load GL via beryllium's proc address function
    let gl = unsafe {
        glow::Context::from_loader_function(|name: &str| {
            // Convert Rust &str to a nul-terminated C string for SDL's loader.
            let c_name = CString::new(name).unwrap();
            // `get_proc_address` expects a raw C pointer; cast as needed and
            // return a `*const c_void` for glow.
            let addr = win.get_proc_address(c_name.as_ptr() as *const u8);
            addr as *const c_void
        })
    };



    let mut controllers = Vec::new();
    'the_loop: loop {
        // Process events from this frame.
        #[allow(clippy::never_loop)]
        while let Some((event, _timestamp)) = sdl.poll_events() {
            match event {
                Event::Quit => break 'the_loop,
                Event::ControllerAdded { index } => match sdl.open_game_controller(index) {
                    Ok(controller) => {
                        println!(
                            "Opened `{name}` (type: {type_:?}): {mapping}",
                            name = controller.get_name(),
                            type_ = controller.get_type(),
                            mapping = controller.get_mapping_string(),
                        );
                        controllers.push(controller);
                    }
                    Err(msg) => println!("Couldn't open {index}: {msg:?}"),
                },
                Event::JoystickAxis { .. }
                | Event::ControllerAxis { .. }
                | Event::MouseMotion { .. } => (),
                _ => println!("{event:?}"),
            }
        }

        // TODO: post-events drawing

        // TODO: swap buffers.
    }

    // All the cleanup is handled by the various drop impls.
}
