/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use errno::errno;
use euclid::{TypedPoint2D, TypedVector2D};
use libc::{c_int, c_long, time_t};
use std::fs::File;
use std::io::Read;
use std::mem::{size_of, transmute, zeroed};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::thread;

pub struct DevicePixel;

#[derive(Debug)]
pub enum TouchpadPressurePhase {
    BeforeClick,
    AfterFirstClick,
    AfterSecondClick,
}

#[derive(Debug)]
pub struct TouchId(pub i32);

#[derive(Debug)]
pub enum TouchEventType {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Debug)]
pub enum MouseWindowEvent {
    Click(MouseButton, TypedPoint2D<f32, DevicePixel>),
    MouseDown(MouseButton, TypedPoint2D<f32, DevicePixel>),
    MouseUp(MouseButton, TypedPoint2D<f32, DevicePixel>),
}

#[derive(Debug)]
pub struct KeyModifiers {
    bits: u8,
}

#[derive(Debug)]
pub enum Key {
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,
    NavigateBackward,
    NavigateForward,
}

#[derive(Debug)]
pub enum KeyState {
    Pressed,
    Released,
    Repeated,
}

#[derive(Debug)]
pub enum WindowEvent {
    Idle,
    Refresh,
    Resize,
    TouchpadPressure(TypedPoint2D<f32, DevicePixel>, f32, TouchpadPressurePhase),
    MouseWindowEventClass(MouseWindowEvent),
    MouseWindowMoveEventClass(TypedPoint2D<f32, DevicePixel>),
    Touch(TouchEventType, TouchId, TypedPoint2D<f32, DevicePixel>),
    Scroll(
        ScrollLocation,
        TypedPoint2D<i32, DevicePixel>,
        TouchEventType,
    ),
    Zoom(f32),
    PinchZoom(f32),
    ResetZoom,
    Quit,
    KeyEvent(Option<char>, Key, KeyState, KeyModifiers),
}

pub struct LayerPixel;

#[derive(Debug)]
pub enum ScrollLocation {
    Delta(TypedVector2D<f32, LayerPixel>),
    Start,
    End,
}

extern "C" {
    // XXX: no variadic form in std libs?
    fn ioctl(fd: c_int, req: c_int, ...) -> c_int;
}

#[repr(C)]
struct linux_input_event {
    sec: time_t,
    msec: c_long,
    evt_type: u16,
    code: u16,
    value: i32,
}

#[repr(C)]
struct linux_input_absinfo {
    value: i32,
    minimum: i32,
    maximum: i32,
    fuzz: i32,
    flat: i32,
    resolution: i32,
}

const IOC_NONE: c_int = 0;
const IOC_WRITE: c_int = 1;
const IOC_READ: c_int = 2;

fn ioc(dir: c_int, ioctype: c_int, nr: c_int, size: c_int) -> c_int {
    dir << 30 | size << 16 | ioctype << 8 | nr
}

fn ev_ioc_g_abs(abs: u16) -> c_int {
    ioc(
        IOC_READ,
        'E' as c_int,
        (0x40 + abs) as i32,
        size_of::<linux_input_absinfo>() as i32,
    )
}

const EV_SYN: u16 = 0;
const EV_ABS: u16 = 3;

const EV_REPORT: u16 = 0;

const ABS_MT_SLOT: u16 = 0x2F;
const ABS_MT_TOUCH_MAJOR: u16 = 0x30;
const ABS_MT_TOUCH_MINOR: u16 = 0x31;
const ABS_MT_WIDTH_MAJOR: u16 = 0x32;
const ABS_MT_WIDTH_MINOR: u16 = 0x33;
const ABS_MT_ORIENTATION: u16 = 0x34;
const ABS_MT_POSITION_X: u16 = 0x35;
const ABS_MT_POSITION_Y: u16 = 0x36;
const ABS_MT_TRACKING_ID: u16 = 0x39;

struct InputSlot {
    tracking_id: i32,
    x: i32,
    y: i32,
}

fn dist(x1: i32, x2: i32, y1: i32, y2: i32) -> f32 {
    let delta_x = (x2 - x1) as f32;
    let delta_y = (y2 - y1) as f32;
    (delta_x * delta_x + delta_y * delta_y).sqrt()
}

fn read_input_device(device_path: &Path, sender: &Sender<WindowEvent>) {
    let mut device = match File::open(device_path) {
        Ok(dev) => dev,
        Err(e) => {
            println!("Couldn't open device! {}", e);
            return;
        }
    };
    let fd = device.as_raw_fd();

    let mut x_info: linux_input_absinfo = unsafe { zeroed() };
    let mut y_info: linux_input_absinfo = unsafe { zeroed() };
    unsafe {
        let ret = ioctl(fd, ev_ioc_g_abs(ABS_MT_POSITION_X), &mut x_info);
        if ret < 0 {
            println!("Couldn't get ABS_MT_POSITION_X info {} {}", ret, errno());
        }
    }
    unsafe {
        let ret = ioctl(fd, ev_ioc_g_abs(ABS_MT_POSITION_Y), &mut y_info);
        if ret < 0 {
            println!("Couldn't get ABS_MT_POSITION_Y info {} {}", ret, errno());
        }
    }

    let touch_width = x_info.maximum - x_info.minimum;
    let touch_height = y_info.maximum - y_info.minimum;

    println!(
        "xMin: {}, yMin: {}, touch_width: {}, touch_height: {}",
        x_info.minimum, y_info.minimum, touch_width, touch_height
    );

    let mut buf: [u8; (16 * size_of::<linux_input_event>())] = unsafe { zeroed() };
    let mut slots: [InputSlot; 10] = unsafe { zeroed() };
    for slot in slots.iter_mut() {
        slot.tracking_id = -1;
    }

    let mut last_x = 0;
    let mut last_y = 0;
    let mut first_x = 0;
    let mut first_y = 0;

    let mut last_dist: f32 = 0f32;
    let mut touch_count: i32 = 0;
    let mut current_slot: usize = 0;
    // XXX: Need to use the real dimensions of the screen
    let screen_dist = dist(0, 480, 854, 0);
    loop {
        let read = match device.read(&mut buf) {
            Ok(count) => {
                assert!(
                    count % size_of::<linux_input_event>() == 0,
                    "Unexpected input device read length!"
                );
                count
            }
            Err(e) => {
                println!("Couldn't read device! {}", e);
                return;
            }
        };

        let count = read / size_of::<linux_input_event>();
        let events: *mut linux_input_event = unsafe { transmute(buf.as_mut_ptr()) };
        let mut tracking_updated = false;
        for idx in 0..(count as isize) {
            let event: &linux_input_event = unsafe { transmute(events.offset(idx)) };
            match (event.evt_type, event.code) {
                (EV_SYN, EV_REPORT) => {
                    let slot_a = &slots[0];
                    if tracking_updated {
                        tracking_updated = false;
                        if slot_a.tracking_id == -1 {
                            println!("Touch up");
                            let delta_x = slot_a.x - first_x;
                            let delta_y = slot_a.y - first_y;
                            let dist = delta_x * delta_x + delta_y * delta_y;
                            if dist < 16 {
                                let click_pt = TypedPoint2D::new(slot_a.x as f32, slot_a.y as f32);
                                println!("Dispatching click!");
                                sender
                                    .send(WindowEvent::MouseWindowEventClass(
                                        MouseWindowEvent::MouseDown(MouseButton::Left, click_pt),
                                    ))
                                    .ok()
                                    .unwrap();
                                sender
                                    .send(WindowEvent::MouseWindowEventClass(
                                        MouseWindowEvent::MouseUp(MouseButton::Left, click_pt),
                                    ))
                                    .ok()
                                    .unwrap();
                                sender
                                    .send(WindowEvent::MouseWindowEventClass(
                                        MouseWindowEvent::Click(MouseButton::Left, click_pt),
                                    ))
                                    .ok()
                                    .unwrap();
                            }
                        } else {
                            println!("Touch down");
                            last_x = slot_a.x;
                            last_y = slot_a.y;
                            first_x = slot_a.x;
                            first_y = slot_a.y;
                            if touch_count >= 2 {
                                let slot_b = &slots[1];
                                last_dist = dist(slot_a.x, slot_b.x, slot_a.y, slot_b.y);
                            }
                        }
                    } else {
                        println!("Touch move x: {}, y: {}", slot_a.x, slot_a.y);
                        sender
                            .send(WindowEvent::Scroll(
                                ScrollLocation::Delta(TypedVector2D::new(
                                    (slot_a.x - last_x) as f32,
                                    (slot_a.y - last_y) as f32,
                                )),
                                TypedPoint2D::new(slot_a.x, slot_a.y),
                                TouchEventType::Move,
                            ))
                            .ok()
                            .unwrap();
                        last_x = slot_a.x;
                        last_y = slot_a.y;
                        if touch_count >= 2 {
                            let slot_b = &slots[1];
                            let cur_dist = dist(slot_a.x, slot_b.x, slot_a.y, slot_b.y);
                            println!(
                                "Zooming {} {} {} {}",
                                cur_dist,
                                last_dist,
                                screen_dist,
                                ((screen_dist + (cur_dist - last_dist)) / screen_dist)
                            );
                            sender
                                .send(WindowEvent::Zoom(
                                    (screen_dist + (cur_dist - last_dist)) / screen_dist,
                                ))
                                .ok()
                                .unwrap();
                            last_dist = cur_dist;
                        }
                    }
                }
                (EV_SYN, _) => println!("Unknown SYN code {}", event.code),
                (EV_ABS, ABS_MT_SLOT) => if (event.value as usize) < slots.len() {
                    current_slot = event.value as usize;
                } else {
                    println!("Invalid slot! {}", event.value);
                },
                (EV_ABS, ABS_MT_TOUCH_MAJOR) => (),
                (EV_ABS, ABS_MT_TOUCH_MINOR) => (),
                (EV_ABS, ABS_MT_WIDTH_MAJOR) => (),
                (EV_ABS, ABS_MT_WIDTH_MINOR) => (),
                (EV_ABS, ABS_MT_ORIENTATION) => (),
                (EV_ABS, ABS_MT_POSITION_X) => {
                    slots[current_slot].x = event.value - x_info.minimum;
                }
                (EV_ABS, ABS_MT_POSITION_Y) => {
                    slots[current_slot].y = event.value - y_info.minimum;
                }
                (EV_ABS, ABS_MT_TRACKING_ID) => {
                    let current_id = slots[current_slot].tracking_id;
                    if current_id != event.value && (current_id == -1 || event.value == -1) {
                        tracking_updated = true;
                        if event.value == -1 {
                            touch_count -= 1;
                        } else {
                            touch_count += 1;
                        }
                    }
                    slots[current_slot].tracking_id = event.value;
                }
                (EV_ABS, _) => println!("Unknown ABS code {}", event.code),
                (_, _) => println!("Unknown event type {}", event.evt_type),
            }
        }
    }
}

pub fn run_input_loop(event_sender: &Sender<WindowEvent>) {
    let sender = event_sender.clone();
    thread::spawn(move || {
        // XXX need to scan all devices and read every one.
        let touchinputdev = Path::new("/dev/input/event0");
        read_input_device(&touchinputdev, &sender);
    });
}
