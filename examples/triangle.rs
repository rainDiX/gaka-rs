/*
* SPDX-License-Identifier: MIT
*/

use std::{mem, rc::Rc};

use opal::graphics::vulkan::context::VulkanContext;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, DeviceId, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{CursorGrabMode, Window, WindowAttributes, WindowId},
};

/// State of the window.
struct Application {
    modifiers: ModifiersState,
    occluded: bool,

    cursor_position: Option<PhysicalPosition<f64>>,
    cursor_grab: CursorGrabMode,
    cursor_hidden: bool,

    window: Option<Window>,
    context: Option<Rc<VulkanContext>>,
}

impl Application {
    fn new() -> Self {
        Self {
            occluded: Default::default(),
            modifiers: Default::default(),
            cursor_grab: CursorGrabMode::None,
            cursor_position: Default::default(),
            cursor_hidden: Default::default(),
            window: None,
            context: None,
        }
    }

    pub fn minimize(&mut self) {
        match &self.window {
            Some(w) => w.set_minimized(true),
            None => {}
        };
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        //todo!();
    }

    pub fn cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_position = Some(position);
    }

    pub fn cursor_left(&mut self) {
        self.cursor_position = None;
    }

    fn toggle_maximize(&self) {
        match &self.window {
            Some(w) => w.set_maximized(!w.is_maximized()),
            None => {}
        };
    }

    fn toggle_decorations(&self) {
        match &self.window {
            Some(w) => w.set_decorations(!w.is_decorated()),
            None => {}
        };
    }

    fn toggle_resizable(&self) {
        match &self.window {
            Some(w) => w.set_resizable(!w.is_resizable()),
            None => {}
        };
    }

    fn toggle_cursor_visibility(&mut self) {
        self.cursor_hidden = !self.cursor_hidden;
        match &self.window {
            Some(w) => w.set_cursor_visible(!self.cursor_hidden),
            None => {}
        };
    }

    /// Toggle fullscreen.
    fn toggle_fullscreen(&self) {
        match &self.window {
            Some(w) => {
                let fullscreen = if w.fullscreen().is_some() {
                    None
                } else {
                    Some(winit::window::Fullscreen::Borderless(None))
                };
                w.set_fullscreen(fullscreen);
            }
            None => {}
        }
    }

    /// Cycle through the grab modes ignoring errors.
    fn cycle_cursor_grab(&mut self) {
        self.cursor_grab = match self.cursor_grab {
            CursorGrabMode::None => CursorGrabMode::Confined,
            CursorGrabMode::Confined => CursorGrabMode::Locked,
            CursorGrabMode::Locked => CursorGrabMode::None,
        };
        log::info!("Changing cursor grab mode to {:?}", self.cursor_grab);
        match &self.window {
            Some(w) => {
                if let Err(err) = w.set_cursor_grab(self.cursor_grab) {
                    log::error!("Error setting cursor grab: {err}");
                }
            }
            None => {}
        }
    }

    /// Swap the window dimensions with `request_inner_size`.
    fn swap_dimensions(&mut self) {
        match &self.window {
            Some(w) => {
                let old_inner_size = w.inner_size();
                let mut inner_size = old_inner_size;

                mem::swap(&mut inner_size.width, &mut inner_size.height);
                log::info!("Requesting resize from {old_inner_size:?} to {inner_size:?}");

                if let Some(new_inner_size) = w.request_inner_size(inner_size) {
                    if old_inner_size == new_inner_size {
                        log::info!("Inner size change got ignored");
                    } else {
                        self.resize(new_inner_size);
                    }
                } else {
                    log::info!("Request inner size is asynchronous");
                }
            }
            None => {}
        }
    }

    /// Show window menu.
    fn show_menu(&self) {
        if let Some(window) = &self.window {
            if let Some(position) = self.cursor_position {
                window.show_window_menu(position);
            }
        }
    }

    /// Change window occlusion state.
    fn set_occluded(&mut self, occluded: bool) {
        self.occluded = occluded;
        if !occluded {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    /// Draw the window contents.
    fn draw(&mut self) -> Result<(), ()> {
        if self.occluded {
            log::info!("Skipping drawing occluded window");
            return Ok(());
        }

        /*
        const WHITE: u32 = 0xffffffff;
        const DARK_GRAY: u32 = 0xff181818;

        let color = match self.theme {
            Theme::Light => WHITE,
            Theme::Dark => DARK_GRAY,
        };

        let mut buffer = self.surface.buffer_mut()?;
        buffer.fill(color);
        self.window.pre_present_notify();
        buffer.present()?;
        */
        Ok(())
    }
}

impl ApplicationHandler for Application {
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                self.resize(size);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                log::info!("Window={window_id:?} changed scale to {scale_factor}");
            }
            WindowEvent::RedrawRequested => {
                if let Err(_err) = self.draw() {
                    log::error!("Error drawing window");
                }
            }
            WindowEvent::Occluded(occluded) => {
                self.set_occluded(occluded);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
                log::info!("Modifiers changed to {:?}", self.modifiers);
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    log::info!("Mouse wheel Line Delta: ({x},{y})");
                }
                MouseScrollDelta::PixelDelta(px) => {
                    log::info!("Mouse wheel Pixel Delta: ({},{})", px.x, px.y);
                }
            },
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                let mods = self.modifiers;

                // Dispatch actions only on press.
                if event.state.is_pressed() {
                    if let Key::Named(key) = event.logical_key {
                        match key {
                            NamedKey::Escape => event_loop.exit(),
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                log::info!("Moved cursor to {position:?}");
                self.cursor_moved(position);
            }
            WindowEvent::ActivationTokenDone { token: _token, .. } => {
                #[cfg(any(x11_platform, wayland_platform))]
                {
                    startup_notify::set_activation_token_env(_token);
                    if let Err(err) = self.create_window(event_loop, None) {
                        error!("Error creating new window: {err}");
                    }
                }
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        log::info!("Device {device_id:?} event: {event:?}");
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attribs = WindowAttributes::default()
                .with_resizable(false)
                .with_inner_size(PhysicalSize::new(800, 600));

            let window = event_loop
                .create_window(window_attribs)
                .expect("Failed to create window");

            let mut context = Rc::new(
                VulkanContext::new(
                    "Triangle",
                    0,
                    &window.display_handle().unwrap().as_raw(),
                    &window.window_handle().unwrap().as_raw(),
                    None,
                    None,
                )
                .expect("Failed to create vulkan Context"),
            );

            let device = Rc::new(context
                .create_graphic_device_default()
                .expect("Failed to create device"));

            let swapchain = device.create_swapchain(800, 600);

            self.window = Some(window);
            self.context = Some(context);
        }
    }
}

fn main() {
    env_logger::init();
    let mut app = Application::new();
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app).expect("Failed to run app");
}
