/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Low level Gonk graphics

use libc::{c_char, c_int, c_void, close, size_t};
use std::mem::{size_of, transmute, zeroed};
use std::ptr;

pub const GRALLOC_USAGE_HW_TEXTURE: c_int = 0x00000100;
pub const GRALLOC_USAGE_HW_RENDER: c_int = 0x00000200;
pub const GRALLOC_USAGE_HW_2D: c_int = 0x00000400;
pub const GRALLOC_USAGE_HW_COMPOSER: c_int = 0x00000800;
pub const GRALLOC_USAGE_HW_FB: c_int = 0x00001000;

// system/core/include/cutils/native_handle.h

#[repr(C)]
pub struct native_handle {
    version: c_int,
    num_fds: c_int,
    num_ints: c_int,
    data: [c_int; 0],
}

// system/core/include/system/window.h

#[repr(C)]
pub struct ANativeBase {
    magic: u32,
    version: u32,
    reserved: [isize; 4],
    inc_ref: extern "C" fn(*mut ANativeBase),
    pub dec_ref: extern "C" fn(*mut ANativeBase),
}

#[repr(C)]
pub struct ANativeWindowBuffer {
    common: ANativeBase,
    width: c_int,
    height: c_int,
    stride: c_int,
    format: c_int,
    usage: c_int,
    reserved: [*mut c_void; 2],
    handle: *const native_handle,
    reserved_proc: [*mut c_void; 8],
}

#[repr(C)]
pub struct ANativeWindow {
    pub common: ANativeBase,
    flags: u32,
    min_swap_interval: c_int,
    max_swap_interval: c_int,
    xdpi: f32,
    ydpi: f32,
    oem: [isize; 4],
    set_swap_interval: extern "C" fn(*mut ANativeWindow, c_int) -> c_int,
    //dequeue_buffer_deprecated: extern fn(*mut ANativeWindow, *mut *mut ANativeWindowBuffer) -> c_int,
    //lock_buffer_deprecated: extern fn(*mut ANativeWindow, *mut ANativeWindowBuffer) -> c_int,
    //queue_buffer_deprecated: extern fn(*mut ANativeWindow, *mut ANativeWindowBuffer) -> c_int,
    dequeue_buffer_deprecated: *const c_void,
    lock_buffer_deprecated: *const c_void,
    queue_buffer_deprecated: *const c_void,
    query: extern "C" fn(*const ANativeWindow, c_int, *mut c_int) -> c_int,
    perform: unsafe extern "C" fn(*mut ANativeWindow, c_int, ...) -> c_int,
    //cancel_buffer_deprecated: extern fn(*mut ANativeWindow, *mut ANativeWindowBuffer) -> c_int,
    cancel_buffer_deprecated: *const c_void,
    dequeue_buffer:
        extern "C" fn(*mut ANativeWindow, *mut *mut ANativeWindowBuffer, *mut c_int) -> c_int,
    queue_buffer: extern "C" fn(*mut ANativeWindow, *mut ANativeWindowBuffer, c_int) -> c_int,
    cancel_buffer: extern "C" fn(*mut ANativeWindow, *mut ANativeWindowBuffer, c_int) -> c_int,
}

// hardware/libhardware/include/hardware/hardware.h

#[repr(C)]
pub struct hw_module_methods {
    pub open: extern "C" fn(*const hw_module, *const c_char, *mut *const hw_device) -> c_int,
}

#[repr(C)]
pub struct hw_module {
    tag: u32,
    module_api_version: u16,
    hal_api_version: u16,
    id: *const c_char,
    name: *const c_char,
    author: *const c_char,
    pub methods: *mut hw_module_methods,
    dso: *mut u32,
    reserved: [u32; (32 - 7)],
}

#[repr(C)]
pub struct hw_device {
    tag: u32,
    pub version: u32,
    module: *mut hw_module,
    reserved: [u32; 12],
    close: extern "C" fn(*mut hw_device) -> c_int,
}

#[link(name = "hardware")]
extern "C" {
    pub fn hw_get_module(id: *const c_char, module: *mut *const hw_module) -> c_int;
}

// hardware/libhardware/include/hardware/hwcomposer.h

#[repr(C)]
pub struct hwc_color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct hwc_rect {
    left: c_int,
    top: c_int,
    right: c_int,
    bottom: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct hwc_frect {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

#[repr(C)]
pub struct hwc_region {
    num_rects: i32,
    rects: *const hwc_rect,
}

const HWC_FRAMEBUFFER: i32 = 0;
const HWC_OVERLAY: i32 = 1;
const HWC_BACKGROUND: i32 = 2;
const HWC_FRAMEBUFFER_TARGET: i32 = 3;
const HWC_BLIT: i32 = 4;

const HWC_SKIP_LAYER: u32 = 1;

#[repr(C)]
pub struct hwc_layer {
    composition_type: i32,
    hints: u32,
    flags: u32,
    handle: *const native_handle,
    transform: u32,
    blending: i32,
    source_crop: hwc_frect, // If HWC 1.3, then this takes floats
    display_frame: hwc_rect,
    visible_region_screen: hwc_region,
    acquire_fence_fd: c_int,
    release_fence_fd: c_int,
    plane_alpha: u8,
    pad: [u8; 3],
    surface_damage: hwc_region,
    reserved: [u8; (96 - 84)],
}

#[repr(C)]
pub struct hwc_display_contents {
    retire_fence_fd: c_int,
    // HWC 1.0 not supported
    outbuf: *const u32,
    outbuf_acquire_fence_fd: c_int,
    flags: u32,
    num_hw_layers: size_t,
    hw_layers: [hwc_layer; 2],
}

#[repr(C)]
pub struct hwc_procs {
    invalidate: extern "C" fn(*const hwc_procs),
    vsync: extern "C" fn(*const hwc_procs, c_int, i64),
    hotplug: extern "C" fn(*const hwc_procs, c_int, c_int),
}

pub const HWC_DISPLAY_NO_ATTRIBUTE: u32 = 0;
pub const HWC_DISPLAY_VSYNC_PERIOD: u32 = 1;
pub const HWC_DISPLAY_WIDTH: u32 = 2;
pub const HWC_DISPLAY_HEIGHT: u32 = 3;
pub const HWC_DISPLAY_DPI_X: u32 = 4;
pub const HWC_DISPLAY_DPI_Y: u32 = 5;

pub const HWC_POWER_MODE_OFF: c_int = 0;
pub const HWC_POWER_MODE_DOZE: c_int = 1;
pub const HWC_POWER_MODE_NORMAL: c_int = 2;
pub const HWC_POWER_MODE_DOZE_SUSPEND: c_int = 3;

#[repr(C)]
pub struct hwc_composer_device {
    pub common: hw_device,
    prepare:
        extern "C" fn(*mut hwc_composer_device, size_t, *mut *mut hwc_display_contents) -> c_int,
    set: extern "C" fn(*mut hwc_composer_device, size_t, *mut *mut hwc_display_contents) -> c_int,
    event_control: extern "C" fn(*mut hwc_composer_device, c_int, c_int, c_int) -> c_int,
    pub set_power_mode: extern "C" fn(*mut hwc_composer_device, c_int, c_int) -> c_int,
    query: extern "C" fn(*mut hwc_composer_device, c_int, *mut c_int) -> c_int,
    register_procs: extern "C" fn(*mut hwc_composer_device, *const hwc_procs),
    dump: extern "C" fn(*mut hwc_composer_device, *const c_char, c_int),
    get_display_configs:
        extern "C" fn(*mut hwc_composer_device, c_int, *mut u32, *mut size_t) -> c_int,
    pub get_display_attributes:
        extern "C" fn(*mut hwc_composer_device, c_int, u32, *const u32, *mut i32) -> c_int,
    reserved: [*mut c_void; 4],
}

// system/core/include/system/graphics.h

#[repr(C)]
pub struct android_ycbcr {
    y: *mut c_void,
    cb: *mut c_void,
    cr: *mut c_void,
    ystride: size_t,
    cstride: size_t,
    chroma_step: size_t,
    reserved: [u32; 8],
}

// hardware/libhardware/include/hardware/gralloc.h

#[repr(C)]
pub struct gralloc_module {
    common: hw_module,
    register_buffer: extern "C" fn(*const gralloc_module, *const native_handle) -> c_int,
    unregister_buffer: extern "C" fn(*const gralloc_module, *const native_handle) -> c_int,
    lock: extern "C" fn(
        *const gralloc_module,
        *const native_handle,
        c_int,
        c_int,
        c_int,
        c_int,
        *mut *mut c_void,
    ) -> c_int,
    unlock: extern "C" fn(*const gralloc_module, *const native_handle) -> c_int,
    perform: extern "C" fn(*const gralloc_module, c_int, ...) -> c_int,
    lock_ycbcr: extern "C" fn(
        *const gralloc_module,
        *const native_handle,
        c_int,
        c_int,
        c_int,
        c_int,
        c_int,
        *mut android_ycbcr,
    ) -> c_int,
    reserved: [*mut c_void; 6],
}

#[repr(C)]
pub struct alloc_device {
    common: hw_device,
    alloc: extern "C" fn(
        *mut alloc_device,
        c_int,
        c_int,
        c_int,
        c_int,
        *mut *const native_handle,
        *mut c_int,
    ) -> c_int,
    free: extern "C" fn(*mut alloc_device, *const native_handle) -> c_int,
    dump: Option<extern "C" fn(*mut alloc_device, *mut c_char, c_int)>,
    reserved: [*mut c_void; 7],
}

#[repr(C)]
pub struct GonkNativeWindow {
    pub window: ANativeWindow,
    set_usage: extern "C" fn(*mut GonkNativeWindow, c_int) -> c_int,
    set_format: extern "C" fn(*mut GonkNativeWindow, c_int) -> c_int,
    set_transform: extern "C" fn(*mut GonkNativeWindow, c_int) -> c_int,
    set_dimensions: extern "C" fn(*mut GonkNativeWindow, c_int, c_int) -> c_int,
    api_connect: extern "C" fn(*mut GonkNativeWindow, c_int) -> c_int,
    api_disconnect: extern "C" fn(*mut GonkNativeWindow, c_int) -> c_int,
    count: i32,
    alloc_dev: *mut alloc_device,
    hwc_dev: *mut hwc_composer_device,
    width: i32,
    height: i32,
    format: c_int,
    usage: c_int,
    last_fence: c_int,
    last_idx: i32,
    bufs: [Option<*mut GonkNativeWindowBuffer>; 2],
    fences: [c_int; 2],
}

impl ANativeBase {
    fn magic(a: char, b: char, c: char, d: char) -> u32 {
        (a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | d as u32
    }
}

#[repr(C)]
pub struct GonkNativeWindowBuffer {
    buffer: ANativeWindowBuffer,
    count: i32,
}

#[link(name = "native_window_glue", kind = "static")]
extern "C" {
    fn gnw_perform(win: *mut ANativeWindow, op: c_int, ...) -> c_int;
}

#[link(name = "suspend")]
extern "C" {
    pub fn autosuspend_disable();
}

extern "C" fn set_swap_interval(_base: *mut ANativeWindow, _interval: c_int) -> c_int {
    debug!("set_swap_interval");
    0
}

const NATIVE_WINDOW_WIDTH: c_int = 0;
const NATIVE_WINDOW_HEIGHT: c_int = 1;
const NATIVE_WINDOW_FORMAT: c_int = 2;
const NATIVE_WINDOW_DEFAULT_WIDTH: c_int = 6;
const NATIVE_WINDOW_DEFAULT_HEIGHT: c_int = 7;
const NATIVE_WINDOW_TRANSFORM_HINT: c_int = 8;
const NATIVE_WINDOW_CONSUMER_USAGE_BITS: c_int = 10;
const NATIVE_WINDOW_DEFAULT_DATASPACE: c_int = 12;
const NATIVE_WINDOW_BUFFER_AGE: c_int = 13;

extern "C" fn query(base: *const ANativeWindow, what: c_int, value: *mut c_int) -> c_int {
    debug!("query {}", what);
    unsafe {
        let window: &GonkNativeWindow = transmute(base);
        match what {
            NATIVE_WINDOW_WIDTH => {
                *value = window.width;
                0
            }
            NATIVE_WINDOW_HEIGHT => {
                *value = window.height;
                0
            }
            NATIVE_WINDOW_FORMAT => {
                *value = window.format;
                0
            }
            NATIVE_WINDOW_DEFAULT_WIDTH => {
                *value = window.width;
                0
            }
            NATIVE_WINDOW_DEFAULT_HEIGHT => {
                *value = window.height;
                0
            }
            NATIVE_WINDOW_TRANSFORM_HINT => {
                *value = 0;
                0
            }
            NATIVE_WINDOW_CONSUMER_USAGE_BITS => {
                info!(
                    "Querying NATIVE_WINDOW_CONSUMER_USAGE_BITS -> {}",
                    window.usage
                );
                *value = window.usage;
                0
            }
            NATIVE_WINDOW_DEFAULT_DATASPACE => {
                *value = 0;
                0
            }
            NATIVE_WINDOW_BUFFER_AGE => {
                *value = 0;
                0
            }
            _ => {
                error!("Unsupported query: {}", what);
                -1
            }
        }
    }
}

extern "C" fn dequeue_buffer(
    base: *mut ANativeWindow,
    buf: *mut *mut ANativeWindowBuffer,
    fence: *mut c_int,
) -> c_int {
    info!("dequeue_buffer");
    unsafe {
        let window: &mut GonkNativeWindow = transmute(base);
        debug!(
            "We have {} buffers, last_idx={}",
            window.bufs.len(),
            window.last_idx
        );
        for idx in 0..window.bufs.len() {
            if idx == window.last_idx as usize {
                continue;
            }
            match window.bufs[idx] {
                Some(entry) => {
                    debug!("Buffer {} exists", idx);
                    (*buf) = transmute(entry);
                    window.bufs[idx] = None;
                    *fence = window.fences[idx];
                    window.fences[idx] = -1;
                    return 0;
                }
                None => debug!("Buffer {} is None", idx),
            }
        }
    }
    error!("returning -1!!");
    -1
}

extern "C" fn queue_buffer(
    base: *mut ANativeWindow,
    buf: *mut ANativeWindowBuffer,
    fence: c_int,
) -> c_int {
    info!("queue_buffer");
    unsafe {
        let window: &mut GonkNativeWindow = transmute(base);
        for idx in 0..window.bufs.len() {
            match window.bufs[idx] {
                Some(_) => (),
                None => {
                    window.last_idx = idx as i32;
                    window.bufs[idx] = Some(transmute(buf));
                    window.fences[idx] = window.draw(buf, fence);
                    return 0;
                }
            }
        }
    }
    -1
}

extern "C" fn cancel_buffer(
    base: *mut ANativeWindow,
    buf: *mut ANativeWindowBuffer,
    fence: c_int,
) -> c_int {
    debug!("cancel_buffer");
    unsafe {
        let window: &mut GonkNativeWindow = transmute(base);
        for idx in 0..window.bufs.len() {
            match window.bufs[idx] {
                Some(_) => (),
                None => {
                    window.bufs[idx] = Some(transmute(buf));
                    window.fences[idx] = -1;
                    close(fence);
                    return 0;
                }
            }
        }
    }
    -1
}

extern "C" fn set_usage(window: *mut GonkNativeWindow, usage: c_int) -> c_int {
    info!("Setting usage flags to {}", usage);
    unsafe {
        (*window).usage = usage;
        (*window).alloc_buffers();
    }
    0
}

extern "C" fn set_format(window: *mut GonkNativeWindow, format: c_int) -> c_int {
    debug!("Setting format to {}", format);
    unsafe {
        (*window).format = format;
    }
    0
}

extern "C" fn set_transform(_: *mut GonkNativeWindow, _: c_int) -> c_int {
    debug!("set_transform");
    0
}

extern "C" fn set_dimensions(_: *mut GonkNativeWindow, width: c_int, height: c_int) -> c_int {
    debug!("set_dimensions to {}x{}", width, height);
    0
}

#[allow(unused_variables)]
extern "C" fn api_connect(window: *mut GonkNativeWindow, api: c_int) -> c_int {
    debug!("api_connect");
    0
}

#[allow(unused_variables)]
extern "C" fn api_disconnect(window: *mut GonkNativeWindow, api: c_int) -> c_int {
    debug!("api_disconnect");
    0
}

extern "C" fn gnw_inc_ref(base: *mut ANativeBase) {
    debug!("gnw_inc_ref");
    let win: &mut GonkNativeWindow = unsafe { transmute(base) };
    win.count += 1;
}

extern "C" fn gnw_dec_ref(base: *mut ANativeBase) {
    debug!("gnw_dec_ref");
    let win: &mut GonkNativeWindow = unsafe { transmute(base) };
    win.count -= 1;
    if win.count == 0 {
        unsafe { transmute::<_, Box<GonkNativeWindow>>(base) };
    }
}

impl GonkNativeWindow {
    pub fn new(
        alloc_dev: *mut alloc_device,
        hwc_dev: *mut hwc_composer_device,
        width: i32,
        height: i32,
        usage: c_int,
    ) -> *mut GonkNativeWindow {
        let window = Box::new(GonkNativeWindow {
            window: ANativeWindow {
                common: ANativeBase {
                    magic: ANativeBase::magic('_', 'w', 'n', 'd'),
                    version: size_of::<ANativeBase>() as u32,
                    reserved: unsafe { zeroed() },
                    inc_ref: gnw_inc_ref,
                    dec_ref: gnw_dec_ref,
                },
                flags: 0,
                min_swap_interval: 0,
                max_swap_interval: 0,
                xdpi: 0f32,
                ydpi: 0f32,
                oem: unsafe { zeroed() },
                set_swap_interval: set_swap_interval,
                dequeue_buffer_deprecated: ptr::null(),
                lock_buffer_deprecated: ptr::null(),
                queue_buffer_deprecated: ptr::null(),
                query: query,
                perform: gnw_perform,
                cancel_buffer_deprecated: ptr::null(),
                dequeue_buffer: dequeue_buffer,
                queue_buffer: queue_buffer,
                cancel_buffer: cancel_buffer,
            },
            set_usage: set_usage,
            set_format: set_format,
            set_transform: set_transform,
            set_dimensions: set_dimensions,
            api_connect: api_connect,
            api_disconnect: api_disconnect,
            count: 1,
            alloc_dev: alloc_dev,
            hwc_dev: hwc_dev,
            width: width,
            height: height,
            format: 0,
            usage: usage,
            last_fence: -1,
            last_idx: -1,
            bufs: unsafe { zeroed() },
            fences: [-1, -1],
        });

        unsafe { transmute(window) }
    }

    fn draw(&mut self, buf: *mut ANativeWindowBuffer, fence: c_int) -> c_int {
        let gonkbuf: &mut GonkNativeWindowBuffer = unsafe { transmute(buf) };
        info!(
            "draw {}x{} {}",
            gonkbuf.buffer.width,
            gonkbuf.buffer.height,
            size_of::<hwc_layer>() as i32,
        );
        let rect = hwc_rect {
            left: 0,
            top: 0,
            right: gonkbuf.buffer.width,
            bottom: gonkbuf.buffer.height,
        };
        let mut list = hwc_display_contents {
            retire_fence_fd: -1,
            outbuf: ptr::null(),
            outbuf_acquire_fence_fd: -1,
            flags: 1, /* HWC_GEOMETRY_CHANGED */
            num_hw_layers: 2,
            hw_layers: [
                hwc_layer {
                    composition_type: HWC_FRAMEBUFFER,
                    hints: 0,
                    flags: HWC_SKIP_LAYER,
                    handle: ptr::null(),
                    transform: 0,
                    blending: 0,
                    source_crop: hwc_frect {
                        left: 0.0,
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                    },
                    display_frame: hwc_rect {
                        left: 0,
                        top: 0,
                        right: 0,
                        bottom: 0,
                    },
                    visible_region_screen: hwc_region {
                        num_rects: 0,
                        rects: ptr::null(),
                    },
                    acquire_fence_fd: -1,
                    release_fence_fd: -1,
                    plane_alpha: 0xff,
                    pad: [0, 0, 0],
                    surface_damage: hwc_region {
                        num_rects: 0,
                        rects: ptr::null(),
                    },
                    reserved: [0; 12],
                },
                hwc_layer {
                    composition_type: HWC_FRAMEBUFFER_TARGET,
                    hints: 0,
                    flags: 0,
                    handle: gonkbuf.buffer.handle,
                    transform: 0,
                    blending: 0,
                    source_crop: hwc_frect {
                        left: 0.0,
                        top: 0.0,
                        right: gonkbuf.buffer.width as f32,
                        bottom: gonkbuf.buffer.height as f32,
                    },
                    display_frame: rect,
                    visible_region_screen: hwc_region {
                        num_rects: 1,
                        rects: &rect,
                    },
                    acquire_fence_fd: fence,
                    release_fence_fd: -1,
                    plane_alpha: 0xff,
                    pad: [0, 0, 0],
                    surface_damage: hwc_region {
                        num_rects: 0,
                        rects: ptr::null(),
                    },
                    reserved: [0; 12],
                },
            ],
        };
        unsafe {
            let mut displays: [*mut hwc_display_contents; 1] = [&mut list];
            let prep_res = ((*self.hwc_dev).prepare)(
                self.hwc_dev,
                displays.len() as size_t,
                transmute(displays.as_mut_ptr()),
            );
            info!("hwc.prepare returned {}", prep_res);
            let set_res = ((*self.hwc_dev).set)(
                self.hwc_dev,
                displays.len() as size_t,
                transmute(displays.as_mut_ptr()),
            );
            info!("hwc.set returned {}", set_res);
            if list.retire_fence_fd >= 0 {
                close(list.retire_fence_fd);
            }
        }
        list.hw_layers[1].release_fence_fd
    }

    pub fn alloc_buffers(&mut self) {
        self.bufs[0] = Some(GonkNativeWindowBuffer::new(
            self.alloc_dev,
            self.width,
            self.height,
            self.format,
            self.usage,
        ));
        self.bufs[1] = Some(GonkNativeWindowBuffer::new(
            self.alloc_dev,
            self.width,
            self.height,
            self.format,
            self.usage,
        ));
    }
}

extern "C" fn gnwb_inc_ref(base: *mut ANativeBase) {
    let buf: &mut GonkNativeWindowBuffer = unsafe { transmute(base) };
    buf.count += 1;
}

extern "C" fn gnwb_dec_ref(base: *mut ANativeBase) {
    let buf: &mut GonkNativeWindowBuffer = unsafe { transmute(base) };
    buf.count -= 1;
    if buf.count == 0 {
        unsafe { transmute::<_, Box<GonkNativeWindowBuffer>>(base) };
    }
}

impl GonkNativeWindowBuffer {
    pub fn new(
        dev: *mut alloc_device,
        width: i32,
        height: i32,
        format: c_int,
        usage: c_int,
    ) -> *mut GonkNativeWindowBuffer {
        info!(
            "GonkNativeWindowBuffer::new {}x{} {} {}",
            width, height, format, usage
        );
        let mut buf = Box::new(GonkNativeWindowBuffer {
            buffer: ANativeWindowBuffer {
                common: ANativeBase {
                    magic: ANativeBase::magic('_', 'b', 'f', 'r'),
                    version: size_of::<ANativeBase>() as u32,
                    reserved: unsafe { zeroed() },
                    inc_ref: gnwb_inc_ref,
                    dec_ref: gnwb_dec_ref,
                },
                width: width,
                height: height,
                stride: 0,
                format: format,
                usage: usage,
                reserved: unsafe { zeroed() },
                handle: ptr::null(),
                reserved_proc: unsafe { zeroed() },
            },
            count: 1,
        });

        let ret = unsafe {
            info!("About to call alloc {:?}", (*dev).alloc);

            ((*dev).alloc)(
                dev,
                width,
                height,
                format,
                usage,
                &mut buf.buffer.handle,
                &mut buf.buffer.stride,
            )
        };
        assert!(ret == 0, "Failed to allocate gralloc buffer!");

        unsafe { transmute(buf) }
    }
}
