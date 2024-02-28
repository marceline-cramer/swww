use raw_window_handle::*;
use smithay_client_toolkit::{
    compositor::CompositorState,
    shell::{
        wlr_layer::{Anchor, Layer, LayerShell},
        WaylandSurface,
    },
};
use wayland_client::{globals::GlobalList, Connection, Proxy, QueueHandle};
use wgpu::*;

use crate::Daemon;

/// Hardware-accelerated Wayland state.
pub struct GpuSurface {
    device: Device,
    queue: Queue,
    surface: Surface,
    adapter: Adapter,
    width: u32,
    height: u32,
}

impl GpuSurface {
    pub fn new(
        conn: &Connection,
        compositor: &CompositorState,
        globals: &GlobalList,
        qh: &QueueHandle<Daemon>,
    ) -> Self {
        let layer_shell = LayerShell::bind(globals, qh).expect("failed to create layer shell");

        let surface = compositor.create_surface(&qh);

        let layer =
            layer_shell.create_layer_surface(qh, surface, Layer::Background, Some("swww"), None);
        layer.set_anchor(Anchor::all());
        layer.set_exclusive_zone(-1);
        layer.commit();

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let handle = {
            let mut handle = WaylandDisplayHandle::empty();
            handle.display = conn.backend().display_ptr() as *mut _;
            let display_handle = RawDisplayHandle::Wayland(handle);

            let mut handle = WaylandWindowHandle::empty();
            handle.surface = layer.wl_surface().id().as_ptr() as *mut _;
            let window_handle = RawWindowHandle::Wayland(handle);

            /// https://github.com/rust-windowing/raw-window-handle/issues/49
            struct DisplayAndWindow(RawDisplayHandle, RawWindowHandle);

            unsafe impl HasRawDisplayHandle for DisplayAndWindow {
                fn raw_display_handle(&self) -> RawDisplayHandle {
                    self.0
                }
            }

            unsafe impl HasRawWindowHandle for DisplayAndWindow {
                fn raw_window_handle(&self) -> RawWindowHandle {
                    self.1
                }
            }

            DisplayAndWindow(display_handle, window_handle)
        };

        let surface = unsafe { instance.create_surface(&handle).unwrap() };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("Failed to find suitable adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None))
            .expect("Failed to request device");

        Self {
            device,
            queue,
            surface,
            adapter,
            width: 256,
            height: 256,
        }
    }
}
