#[cfg(target_os = "linux")]
pub fn fuck_nvidia() {
    // Disable DMA rendering on Linux + NVIDIA systems (Thanks SpikeHD and Dorion)
    // see: https://github.com/SpikeHD/Dorion/issues/237 and https://github.com/tauri-apps/tauri/issues/9304
    use wgpu::{BackendOptions, Backends, DeviceType, GlBackendOptions, Instance, InstanceDescriptor, InstanceFlags, };

    let instance = Instance::new(&InstanceDescriptor {
        flags: InstanceFlags::empty(),
        backends: Backends::GL | Backends::VULKAN,
        memory_budget_thresholds: Default::default(),
        backend_options: BackendOptions {
            gl: GlBackendOptions::default(),
            dx12: Default::default(),
            noop: Default::default(),
        },
    });

    for adapter in instance.enumerate_adapters(Backends::all()) {
        let info = adapter.get_info();

        match info.device_type {
            DeviceType::DiscreteGpu | DeviceType::IntegratedGpu | DeviceType::VirtualGpu => unsafe {
                if info.name.to_ascii_lowercase().contains("nvidia") {
                    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
                    std::env::set_var("__GL_THREADED_OPTIMIZATIONS", "0");
                    std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
                    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
                    log::debug!("NVIDIA GPU detected, disabling DMABUF rendering!");
                }
            }
            _ => {}
        }
    }
}
