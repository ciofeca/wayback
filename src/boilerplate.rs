// --- configuration and constants ---
//
const BORDER_SIZE: u32 = 1;
const HEADER_SIZE: u32 = 1 + 16 + 1;

const INACTIVE_BORDER: u32 = 0xFF606060;
const ACTIVE_BORDER: u32 = 0xFF000090;
const RED_BUTTON_REGULAR: u32 = 0xFFB04040;
const RED_BUTTON_HOVER: u32 = 0xFFFF4040;
const GREEN_BUTTON_REGULAR: u32 = 0xFF40B040;
const GREEN_BUTTON_HOVER: u32 = 0xFF40FF40;
const YELLOW_BUTTON_REGULAR: u32 = 0xFFB0B040;
const YELLOW_BUTTON_HOVER: u32 = 0xFFFFFF40;
const YELLOW_BUTTON_DISABLED: u32 = 0xFF808020;

// --- oval matrix for window buttons ---
//
const ALLOW_PIXEL: [[bool; 24]; 16] = [
    [
        false, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, false, false, false, false, false, false, false, false,
    ],
    [
        false, false, false, false, false, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, false, false, false, false, false,
    ],
    [
        false, false, false, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, false, false, false,
    ],
    [
        false, false, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, false, false,
    ],
    [
        false, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, false,
    ],
    [
        false, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, false,
    ],
    [
        true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, true,
    ],
    [
        true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, true,
    ],
    [
        true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, true,
    ],
    [
        true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, true,
    ],
    [
        false, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, false,
    ],
    [
        false, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, false,
    ],
    [
        false, false, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, false, false,
    ],
    [
        false, false, false, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, true, true, false, false, false,
    ],
    [
        false, false, false, false, false, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true, false, false, false, false, false,
    ],
    [
        false, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, false, false, false, false, false, false, false, false,
    ],
];

// --- events the user will get ---
//
pub enum Way {
    Exit,
    Idling {
        msec: u32,
    },
    Refresh {
        width: usize,
        height: usize,
    },
    Resize {
        width: u32,
        height: u32,
    },
    Focus {
        enter: bool,
        hover: bool,
        cause: u32,
    },
    KeyInfo {
        rate: i32,
        delay: i32,
    },
    Key {
        text: String,
        keysym: u32,
        pressed: bool,
    },
    Paste {
        text: String,
    },
    Pointer {
        x: u32,
        y: u32,
    },
    Button {
        but: u32,
        status: bool,
    },
}

// --- crates ---
//
extern crate std;
use self::std::io::{BufWriter, Seek, SeekFrom, Write};
use self::std::option::Option::*;
use self::std::string::String;
use self::std::sync::mpsc::*;
use self::std::sync::{Arc, Mutex};

extern crate byteorder;
use self::byteorder::{NativeEndian, WriteBytesExt};

extern crate smithay_client_toolkit as smith;
use self::smith::Environment;
use self::smith::data_device::{DataDevice, DndEvent, ReadPipe};
use self::smith::keyboard::{keysyms, map_keyboard_auto, Event as KbEvent};
use self::smith::pointer::{AutoPointer, AutoThemer};
use self::smith::utils::{DoubleMemPool, MemPool};
use self::smith::wayland_client::commons::Implementation;
use self::smith::wayland_client::protocol::wl_buffer::RequestsTrait as BufferRequests;
use self::smith::wayland_client::protocol::wl_compositor::RequestsTrait as CompositorRequests;
use self::smith::wayland_client::protocol::wl_display::RequestsTrait as DisplayRequests;
use self::smith::wayland_client::protocol::wl_pointer::RequestsTrait as PointerRequests;
use self::smith::wayland_client::protocol::wl_seat::RequestsTrait as SeatRequests;
use self::smith::wayland_client::protocol::wl_subcompositor::RequestsTrait as SubcompRequests;
use self::smith::wayland_client::protocol::wl_subsurface::RequestsTrait as SubsurfaceRequests;
use self::smith::wayland_client::protocol::wl_surface::RequestsTrait as SurfaceRequests;
use self::smith::wayland_client::protocol::*; //{wl_buffer, wl_seat, wl_shm, wl_surface...
use self::smith::wayland_client::{Display, EventQueue, Proxy};
use self::smith::window::{Event as WEvent, Frame, FrameRequest, Window};

// ----------------------------------------------------
// slightly altered Smithay's BasicFrame follows
// ----------------------------------------------------
//
const TOP: usize = 0;
const BOTTOM: usize = 1;
const LEFT: usize = 2;
const RIGHT: usize = 3;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Location {
    None,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
    TopBar,
    Button(UIButton),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum UIButton {
    Minimize,
    Maximize,
    Close,
}

struct Part {
    surface: Proxy<wl_surface::WlSurface>,
    subsurface: Proxy<wl_subsurface::WlSubsurface>,
}

impl Part {
    fn new(
        parent: &Proxy<wl_surface::WlSurface>,
        compositor: &Proxy<wl_compositor::WlCompositor>,
        subcompositor: &Proxy<wl_subcompositor::WlSubcompositor>,
    ) -> Part {
        let surface = compositor.create_surface().unwrap().implement(|_, _| {});
        let subsurface = subcompositor
            .get_subsurface(&surface, parent)
            .unwrap()
            .implement(|_, _| {});
        subsurface.set_desync();
        Part {
            surface,
            subsurface,
        }
    }
}

impl Drop for Part {
    fn drop(&mut self) {
        self.subsurface.destroy();
        self.surface.destroy();
    }
}

struct PointerUserData {
    location: Location,
    position: (f64, f64),
    seat: Proxy<wl_seat::WlSeat>,
}

// --- the core frame ---

struct Inner {
    parts: [Part; 4],
    size: Mutex<(u32, u32)>,
    implem: Mutex<Box<Implementation<u32, FrameRequest> + Send>>,
    maximized: Mutex<bool>,
}

impl Inner {
    fn find_surface(&self, surface: &Proxy<wl_surface::WlSurface>) -> Location {
        if surface.equals(&self.parts[TOP].surface) {
            Location::Top
        } else if surface.equals(&self.parts[BOTTOM].surface) {
            Location::Bottom
        } else if surface.equals(&self.parts[LEFT].surface) {
            Location::Left
        } else if surface.equals(&self.parts[RIGHT].surface) {
            Location::Right
        } else {
            Location::None
        }
    }
}

fn precise_location(old: Location, width: u32, x: f64, y: f64) -> Location {
    match old {
        Location::Top
        | Location::TopRight
        | Location::TopLeft
        | Location::TopBar
        | Location::Button(_) => {
            // top surface
            if x <= BORDER_SIZE as f64 {
                Location::TopLeft
            } else if x >= (width + BORDER_SIZE) as f64 {
                Location::TopRight
            } else if y <= BORDER_SIZE as f64 {
                Location::Top
            } else {
                find_button(x, y, width)
            }
        }
        Location::Bottom | Location::BottomLeft | Location::BottomRight => {
            if x <= BORDER_SIZE as f64 {
                Location::BottomLeft
            } else if x >= (width + BORDER_SIZE) as f64 {
                Location::BottomRight
            } else {
                Location::Bottom
            }
        }
        other => other,
    }
}

fn find_button(x: f64, y: f64, w: u32) -> Location {
    if (w >= 24) && (x > (w + BORDER_SIZE - 24) as f64) && (x <= (w + BORDER_SIZE) as f64)
        && (y <= (BORDER_SIZE + 16) as f64)
    {
        Location::Button(UIButton::Close)
    } else if (w >= 56) && (x > (w + BORDER_SIZE - 56) as f64)
        && (x <= (w + BORDER_SIZE - 32) as f64) && (y <= (BORDER_SIZE + 16) as f64)
    {
        Location::Button(UIButton::Maximize)
    } else if (w >= 88) && (x > (w + BORDER_SIZE - 88) as f64)
        && (x <= (w + BORDER_SIZE - 64) as f64) && (y <= (BORDER_SIZE + 16) as f64)
    {
        Location::Button(UIButton::Minimize)
    } else {
        Location::TopBar
    }
}

/// A minimalistic set of decorations
///
pub struct SimpleFrame {
    inner: Arc<Inner>,
    pools: DoubleMemPool,
    buffers: Vec<Proxy<wl_buffer::WlBuffer>>,
    active: bool,
    hidden: bool,
    pointers: Vec<AutoPointer>,
    themer: AutoThemer,
    surface_version: u32,
}

impl Frame for SimpleFrame {
    type Error = ::std::io::Error;
    fn init(
        base_surface: &Proxy<wl_surface::WlSurface>,
        compositor: &Proxy<wl_compositor::WlCompositor>,
        subcompositor: &Proxy<wl_subcompositor::WlSubcompositor>,
        shm: &Proxy<wl_shm::WlShm>,
        implementation: Box<Implementation<u32, FrameRequest> + Send>,
    ) -> Result<SimpleFrame, ::std::io::Error> {
        let pools = DoubleMemPool::new(&shm)?;
        let parts = [
            Part::new(base_surface, compositor, subcompositor),
            Part::new(base_surface, compositor, subcompositor),
            Part::new(base_surface, compositor, subcompositor),
            Part::new(base_surface, compositor, subcompositor),
        ];
        Ok(SimpleFrame {
            inner: Arc::new(Inner {
                parts: parts,
                size: Mutex::new((1, 1)),
                implem: Mutex::new(implementation),
                maximized: Mutex::new(false),
            }),
            pools,
            buffers: Vec::new(),
            active: false,
            hidden: false,
            pointers: Vec::new(),
            themer: AutoThemer::init(None, compositor.clone(), shm.clone()),
            surface_version: compositor.version(),
        })
    }

    fn new_seat(&mut self, seat: &Proxy<wl_seat::WlSeat>) {
        use self::wl_pointer::Event;
        let inner = self.inner.clone();
        let pointer = self.themer.theme_pointer_with_impl(
            seat.get_pointer().unwrap(),
            move |event, pointer: AutoPointer| {
                let data = unsafe { &mut *(pointer.get_user_data() as *mut PointerUserData) };
                let (width, _) = *(inner.size.lock().unwrap());
                match event {
                    Event::Enter {
                        serial,
                        surface,
                        surface_x,
                        surface_y,
                    } => {
                        data.location = precise_location(
                            inner.find_surface(&surface),
                            width,
                            surface_x,
                            surface_y,
                        );
                        data.position = (surface_x, surface_y);
                        change_pointer(&pointer, data.location, Some(serial));
                    }
                    Event::Leave { serial, .. } => {
                        data.location = Location::None;
                        change_pointer(&pointer, data.location, Some(serial));
                    }
                    Event::Motion {
                        surface_x,
                        surface_y,
                        ..
                    } => {
                        data.position = (surface_x, surface_y);
                        let newpos = precise_location(data.location, width, surface_x, surface_y);
                        if newpos != data.location {
                            match (newpos, data.location) {
                                (Location::Button(_), _) | (_, Location::Button(_)) => {
                                    // pointer movement involves a button, request refresh
                                    inner
                                        .implem
                                        .lock()
                                        .unwrap()
                                        .receive(FrameRequest::Refresh, 0);
                                }
                                _ => (),
                            }
                            // we changed part of the decoration, pointer image
                            // may need to be changed
                            data.location = newpos;
                            change_pointer(&pointer, data.location, None);
                        }
                    }
                    Event::Button {
                        serial,
                        button,
                        state,
                        ..
                    } => {
                        if state == wl_pointer::ButtonState::Pressed && button == 0x110 {
                            // left click
                            let req = request_for_location(
                                data.location,
                                &data.seat,
                                *(inner.maximized.lock().unwrap()),
                            );
                            if let Some(req) = req {
                                inner.implem.lock().unwrap().receive(req, serial);
                            }
                        }
                    }
                    _ => {}
                }
            },
        );
        pointer.set_user_data(Box::into_raw(Box::new(PointerUserData {
            location: Location::None,
            position: (0.0, 0.0),
            seat: seat.clone(),
        })) as *mut ());
        self.pointers.push(pointer);
    }

    fn set_active(&mut self, active: bool) -> bool {
        if self.active != active {
            self.active = active;
            true
        } else {
            false
        }
    }

    fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    fn set_maximized(&mut self, maximized: bool) -> bool {
        let mut my_maximized = self.inner.maximized.lock().unwrap();
        if *my_maximized != maximized {
            *my_maximized = maximized;
            true
        } else {
            false
        }
    }

    fn resize(&mut self, newsize: (u32, u32)) {
        *(self.inner.size.lock().unwrap()) = newsize;
    }

    fn redraw(&mut self) {
        if self.hidden {
            // don't draw the borders
            for p in &self.inner.parts {
                p.surface.attach(None, 0, 0);
                p.surface.commit();
            }
            return;
        }
        let (width, height) = *(self.inner.size.lock().unwrap());
        // destroy current pending buffers
        // TODO: do double-buffering
        for b in self.buffers.drain(..) {
            b.destroy();
        }

        {
            // grab the current pool
            let pool = self.pools.pool();
            // resize the pool as appropriate
            let pxcount =
                2 * height * BORDER_SIZE + (width + 2 * BORDER_SIZE) * (BORDER_SIZE + HEADER_SIZE);
            pool.resize(4 * pxcount as usize)
                .expect("I/O Error while redrawing the borders");

            // Redraw the "grey" borders
            let color = if self.active {
                ACTIVE_BORDER
            } else {
                INACTIVE_BORDER
            };
            let _ = pool.seek(SeekFrom::Start(0));
            // draw the "grey" background
            {
                let mut writer = BufWriter::new(&mut *pool);
                for _ in 0..pxcount {
                    let _ = writer.write_u32::<NativeEndian>(color);
                }
                draw_buttons(
                    &mut writer,
                    width,
                    true,
                    self.pointers
                        .iter()
                        .flat_map(|p| {
                            if p.is_alive() {
                                let data =
                                    unsafe { &mut *(p.get_user_data() as *mut PointerUserData) };
                                Some(data.location)
                            } else {
                                None
                            }
                        })
                        .collect(),
                );
                let _ = writer.flush();
            }

            // Create the buffers
            // -> top-subsurface
            let buffer = pool.buffer(
                0,
                (width + 2 * BORDER_SIZE) as i32,
                HEADER_SIZE as i32,
                4 * (width + 2 * BORDER_SIZE) as i32,
                wl_shm::Format::Argb8888,
            ).implement(|_, _| {});
            self.inner.parts[TOP]
                .subsurface
                .set_position(-(BORDER_SIZE as i32), -(HEADER_SIZE as i32));
            self.inner.parts[TOP].surface.attach(Some(&buffer), 0, 0);
            if self.surface_version >= 4 {
                self.inner.parts[TOP].surface.damage_buffer(
                    0,
                    0,
                    (width + 2 * BORDER_SIZE) as i32,
                    HEADER_SIZE as i32,
                );
            } else {
                // surface is old and does not support damage_buffer, so we damage
                // in surface coordinates and hope it is not rescaled
                self.inner.parts[TOP].surface.damage(
                    0,
                    0,
                    (width + 2 * BORDER_SIZE) as i32,
                    HEADER_SIZE as i32,
                );
            }
            self.inner.parts[TOP].surface.commit();
            self.buffers.push(buffer);
            // -> bottom-subsurface
            let buffer = pool.buffer(
                4 * (HEADER_SIZE * (width + 2 * BORDER_SIZE)) as i32,
                (width + 2 * BORDER_SIZE) as i32,
                BORDER_SIZE as i32,
                4 * (width + 2 * BORDER_SIZE) as i32,
                wl_shm::Format::Argb8888,
            ).implement(|_, _| {});
            self.inner.parts[BOTTOM]
                .subsurface
                .set_position(-(BORDER_SIZE as i32), height as i32);
            self.inner.parts[BOTTOM].surface.attach(Some(&buffer), 0, 0);
            if self.surface_version >= 4 {
                self.inner.parts[BOTTOM].surface.damage_buffer(
                    0,
                    0,
                    (width + 2 * BORDER_SIZE) as i32,
                    BORDER_SIZE as i32,
                );
            } else {
                // surface is old and does not support damage_buffer, so we damage
                // in surface coordinates and hope it is not rescaled
                self.inner.parts[BOTTOM].surface.damage(
                    0,
                    0,
                    (width + 2 * BORDER_SIZE) as i32,
                    BORDER_SIZE as i32,
                );
            }
            self.inner.parts[BOTTOM].surface.commit();
            self.buffers.push(buffer);
            // -> left-subsurface
            let buffer = pool.buffer(
                4 * ((HEADER_SIZE + BORDER_SIZE) * (width + 2 * BORDER_SIZE)) as i32,
                BORDER_SIZE as i32,
                height as i32,
                4 * (BORDER_SIZE as i32),
                wl_shm::Format::Argb8888,
            ).implement(|_, _| {});
            self.inner.parts[LEFT]
                .subsurface
                .set_position(-(BORDER_SIZE as i32), 0);
            self.inner.parts[LEFT].surface.attach(Some(&buffer), 0, 0);
            if self.surface_version >= 4 {
                self.inner.parts[LEFT].surface.damage_buffer(
                    0,
                    0,
                    BORDER_SIZE as i32,
                    height as i32,
                );
            } else {
                // surface is old and does not support damage_buffer, so we damage
                // in surface coordinates and hope it is not rescaled
                self.inner.parts[LEFT]
                    .surface
                    .damage(0, 0, BORDER_SIZE as i32, height as i32);
            }
            self.inner.parts[LEFT].surface.commit();
            self.buffers.push(buffer);
            // -> right-subsurface
            let buffer = pool.buffer(
                4
                    * ((HEADER_SIZE + BORDER_SIZE) * (width + 2 * BORDER_SIZE)
                        + BORDER_SIZE * height) as i32,
                BORDER_SIZE as i32,
                height as i32,
                4 * (BORDER_SIZE as i32),
                wl_shm::Format::Argb8888,
            ).implement(|_, _| {});
            self.inner.parts[RIGHT]
                .subsurface
                .set_position(width as i32, 0);
            self.inner.parts[RIGHT].surface.attach(Some(&buffer), 0, 0);
            if self.surface_version >= 4 {
                self.inner.parts[RIGHT].surface.damage_buffer(
                    0,
                    0,
                    BORDER_SIZE as i32,
                    height as i32,
                );
            } else {
                // surface is old and does not support damage_buffer, so we damage
                // in surface coordinates and hope it is not rescaled
                self.inner.parts[RIGHT]
                    .surface
                    .damage(0, 0, BORDER_SIZE as i32, height as i32);
            }
            self.inner.parts[RIGHT].surface.commit();
            self.buffers.push(buffer);
        }
        // swap the pool
        self.pools.swap();
    }

    fn subtract_borders(&self, width: i32, height: i32) -> (i32, i32) {
        if self.hidden {
            (width, height)
        } else {
            (
                width - 2 * (BORDER_SIZE as i32),
                height - BORDER_SIZE as i32 - HEADER_SIZE as i32,
            )
        }
    }

    fn add_borders(&self, width: i32, height: i32) -> (i32, i32) {
        if self.hidden {
            (width, height)
        } else {
            (
                width + 2 * (BORDER_SIZE as i32),
                height + BORDER_SIZE as i32 + HEADER_SIZE as i32,
            )
        }
    }
}

impl Drop for SimpleFrame {
    fn drop(&mut self) {
        for ptr in self.pointers.drain(..) {
            let _data = unsafe { Box::from_raw(ptr.get_user_data() as *mut PointerUserData) };
            ptr.set_user_data(::std::ptr::null_mut());
            if ptr.version() >= 3 {
                ptr.release();
            }
        }
    }
}

fn change_pointer(pointer: &AutoPointer, location: Location, serial: Option<u32>) {
    let name = match location {
        Location::Top => "top_side",
        Location::TopRight => "top_right_corner",
        Location::Right => "right_side",
        Location::BottomRight => "bottom_right_corner",
        Location::Bottom => "bottom_side",
        Location::BottomLeft => "bottom_left_corner",
        Location::Left => "left_side",
        Location::TopLeft => "top_left_corner",
        _ => "left_ptr",
    };
    let _ = pointer.set_cursor(name, serial);
}

fn request_for_location(
    location: Location,
    seat: &Proxy<wl_seat::WlSeat>,
    maximized: bool,
) -> Option<FrameRequest> {
    use self::smith::wayland_protocols::xdg_shell::client::xdg_toplevel::ResizeEdge;
    match location {
        Location::Top => Some(FrameRequest::Resize(seat.clone(), ResizeEdge::Top)),
        Location::TopLeft => Some(FrameRequest::Resize(seat.clone(), ResizeEdge::TopLeft)),
        Location::Left => Some(FrameRequest::Resize(seat.clone(), ResizeEdge::Left)),
        Location::BottomLeft => Some(FrameRequest::Resize(seat.clone(), ResizeEdge::BottomLeft)),
        Location::Bottom => Some(FrameRequest::Resize(seat.clone(), ResizeEdge::Bottom)),
        Location::BottomRight => Some(FrameRequest::Resize(seat.clone(), ResizeEdge::BottomRight)),
        Location::Right => Some(FrameRequest::Resize(seat.clone(), ResizeEdge::Right)),
        Location::TopRight => Some(FrameRequest::Resize(seat.clone(), ResizeEdge::TopRight)),
        Location::TopBar => Some(FrameRequest::Move(seat.clone())),
        Location::Button(UIButton::Close) => Some(FrameRequest::Close),
        Location::Button(UIButton::Maximize) => if maximized {
            Some(FrameRequest::UnMaximize)
        } else {
            Some(FrameRequest::Maximize)
        },
        Location::Button(UIButton::Minimize) => Some(FrameRequest::Minimize),
        Location::None => None,
    }
}

fn draw_buttons(
    pool: &mut BufWriter<&mut MemPool>,
    width: u32,
    maximizable: bool,
    mouses: Vec<Location>,
) {
    // draw up to 3 buttons, depending on the width of the window
    // color of the button depends on whether a pointer is on it, and the maximizable
    // button can be disabled
    // buttons are 24x16
    let ds = BORDER_SIZE;

    if width >= 24 {
        // draw the red button
        let color = if mouses
            .iter()
            .any(|&l| l == Location::Button(UIButton::Close))
        {
            RED_BUTTON_HOVER
        } else {
            RED_BUTTON_REGULAR
        };
        let _ = pool.seek(SeekFrom::Start(
            4 * ((width + 2 * ds) * ds + width + ds - 24) as u64,
        ));
        for r in 0..16 {
            for c in 0..24 {
                if ALLOW_PIXEL[r][c] {
                    pool.write_u32::<NativeEndian>(color).unwrap();
                } else {
                    pool.seek(SeekFrom::Current(4)).unwrap();
                }
            }
            let _ = pool.seek(SeekFrom::Current(4 * (width + 2 * ds - 24) as i64));
        }
    }

    if width >= 56 {
        // draw the yellow button
        let color = if !maximizable {
            YELLOW_BUTTON_DISABLED
        } else if mouses
            .iter()
            .any(|&l| l == Location::Button(UIButton::Maximize))
        {
            YELLOW_BUTTON_HOVER
        } else {
            YELLOW_BUTTON_REGULAR
        };
        let _ = pool.seek(SeekFrom::Start(
            4 * ((width + 2 * ds) * ds + width + ds - 56) as u64,
        ));
        for r in 0..16 {
            for c in 0..24 {
                if ALLOW_PIXEL[r][c] {
                    pool.write_u32::<NativeEndian>(color).unwrap();
                } else {
                    pool.seek(SeekFrom::Current(4)).unwrap();
                }
            }
            let _ = pool.seek(SeekFrom::Current(4 * (width + 2 * ds - 24) as i64));
        }
    }

    if width >= 88 {
        // draw the green button
        let color = if mouses
            .iter()
            .any(|&l| l == Location::Button(UIButton::Minimize))
        {
            GREEN_BUTTON_HOVER
        } else {
            GREEN_BUTTON_REGULAR
        };
        let _ = pool.seek(SeekFrom::Start(
            4 * ((width + 2 * ds) * ds + width + ds - 88) as u64,
        ));
        for r in 0..16 {
            for c in 0..24 {
                if ALLOW_PIXEL[r][c] {
                    pool.write_u32::<NativeEndian>(color).unwrap();
                } else {
                    pool.seek(SeekFrom::Current(4)).unwrap();
                }
            }
            let _ = pool.seek(SeekFrom::Current(4 * (width + 2 * ds - 24) as i64));
        }
    }
}

// -----------------------------------------------------------------------------------------
// --- a bitmapped font (ASCII characters only), statically embedded in the executable file
// --- if you need something better then goto a font rendering library
// -----------------------------------------------------------------------------------------

pub const FONT_X_SIZE: usize = 17;
pub const FONT_Y_SIZE: usize = 24;

const FONT: [u32; 96 * FONT_Y_SIZE] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 448, 448, 448,
    448, 448, 448, 448, 448, 448, 128, 128, 0, 0, 448, 448, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3696,
    3696, 3696, 1056, 1056, 1056, 1056, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1632, 1632,
    1632, 1632, 1632, 8188, 8188, 1632, 816, 8188, 8188, 816, 816, 816, 816, 816, 0, 0, 0, 0, 0, 0,
    0, 384, 384, 3552, 4080, 3608, 3608, 56, 496, 2016, 3840, 3096, 3128, 3640, 2040, 984, 384,
    384, 384, 384, 0, 0, 0, 0, 0, 0, 480, 1008, 1848, 1560, 1560, 1848, 8176, 2016, 4088, 7392,
    6240, 6240, 7392, 4032, 1920, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4032, 4064, 1584, 48, 48, 96,
    224, 14832, 16312, 3864, 3608, 16368, 15328, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 448, 448, 448, 128,
    128, 128, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6144, 7168, 3584, 3840, 1792,
    1792, 896, 896, 896, 896, 896, 896, 1792, 1792, 3584, 3584, 7168, 6144, 0, 0, 0, 0, 0, 0, 24,
    56, 112, 112, 224, 224, 448, 448, 448, 448, 448, 448, 224, 224, 240, 112, 56, 24, 0, 0, 0, 0,
    0, 0, 384, 384, 384, 7608, 8184, 2016, 960, 960, 1632, 1632, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 384, 384, 384, 384, 384, 16380, 16380, 384, 384, 384, 384, 384, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1792, 768, 896, 384, 384, 192, 192, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 8184, 8184, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 960, 960, 960, 0, 0, 0, 0, 0, 0, 0, 6144, 6144, 7168, 3072, 3584, 1536,
    1536, 768, 768, 384, 384, 192, 192, 96, 96, 112, 48, 56, 24, 24, 0, 0, 0, 0, 0, 0, 960, 2016,
    3120, 3120, 6168, 6168, 6168, 6168, 6168, 6168, 6168, 3120, 3120, 2016, 960, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 256, 480, 504, 440, 384, 384, 384, 384, 384, 384, 384, 384, 384, 8184, 8184, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 992, 4088, 3100, 6156, 6156, 6144, 3072, 1536, 896, 448, 96, 48, 24, 8188,
    8188, 0, 0, 0, 0, 0, 0, 0, 0, 0, 960, 2032, 3632, 3072, 3072, 1536, 960, 1984, 3584, 6144,
    6144, 6144, 7192, 4088, 1008, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1792, 1920, 1920, 1728, 1632, 1632,
    1584, 1584, 1560, 1548, 8188, 8188, 1536, 8128, 8128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4088, 4088,
    24, 24, 24, 984, 4088, 3128, 6144, 6144, 6144, 6144, 3084, 4092, 1008, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 7936, 8128, 224, 112, 48, 24, 984, 4088, 3128, 6168, 6168, 6168, 7216, 4080, 1984, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 8184, 8184, 6168, 7192, 3072, 3072, 3584, 1536, 1536, 1792, 768, 768, 896,
    384, 384, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2016, 4080, 7224, 6168, 6168, 3120, 2016, 2016, 3120,
    6168, 6168, 6168, 7224, 4080, 2016, 0, 0, 0, 0, 0, 0, 0, 0, 0, 992, 4080, 3128, 6168, 6168,
    6168, 7216, 8176, 7104, 6144, 3072, 3584, 1792, 1016, 248, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 960, 960, 960, 0, 0, 0, 0, 0, 960, 960, 960, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3840,
    3840, 3840, 0, 0, 0, 0, 1792, 896, 384, 384, 192, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14336, 15360,
    3840, 960, 240, 60, 15, 60, 240, 960, 3840, 15360, 14336, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 16382, 16382, 0, 0, 16382, 16382, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 30,
    120, 480, 1920, 7680, 30720, 7680, 1920, 480, 120, 30, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 992,
    2032, 3608, 3096, 3096, 3584, 1792, 960, 448, 192, 0, 0, 224, 224, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1984, 4064, 7280, 6192, 7704, 7960, 7064, 6552, 6552, 6552, 7960, 7704, 24, 48, 6256, 8160,
    1984, 0, 0, 0, 0, 0, 0, 0, 0, 504, 1016, 896, 1728, 1728, 3168, 3168, 3120, 8176, 8184, 12312,
    12300, 65087, 65087, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2046, 4094, 7192, 6168, 6168, 7192, 4088,
    8184, 14360, 12312, 12312, 12312, 8190, 4094, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14272, 16368,
    14392, 12312, 12300, 12, 12, 12, 12, 12, 12312, 14392, 8176, 4032, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1022, 4094, 7192, 6168, 12312, 12312, 12312, 12312, 12312, 12312, 6168, 7192, 4094, 2046, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 8190, 8190, 6168, 6168, 6552, 408, 504, 504, 408, 6552, 6168, 6168,
    8190, 8190, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16380, 16380, 12336, 12336, 13104, 816, 1008, 1008,
    816, 816, 48, 48, 1020, 1020, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14272, 16368, 14392, 12312, 12300,
    12, 12, 32524, 32524, 12300, 12316, 14392, 16368, 4032, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32382,
    32382, 6168, 6168, 6168, 6168, 8184, 8184, 6168, 6168, 6168, 6168, 32382, 32382, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 8184, 8184, 384, 384, 384, 384, 384, 384, 384, 384, 384, 384, 8184, 8184, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 32736, 32736, 3072, 3072, 3072, 3072, 3072, 3084, 3084, 3084, 3084,
    1548, 2044, 496, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 31998, 31998, 3096, 1560, 792, 408, 472, 1016,
    1848, 3608, 3096, 7192, 63742, 63742, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 510, 510, 48, 48, 48, 48,
    48, 48, 12336, 12336, 12336, 12336, 16382, 16382, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 61455, 63519,
    14364, 15420, 15420, 13932, 13932, 13260, 13260, 12684, 12300, 12300, 65151, 65151, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 32542, 32542, 6200, 6264, 6392, 6360, 6616, 7064, 6936, 7960, 7704, 7192,
    6398, 6398, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 960, 4080, 7224, 6168, 14364, 12300, 12300, 12300,
    12300, 14364, 6168, 7224, 4080, 960, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4092, 8188, 14384, 12336,
    12336, 12336, 6192, 8176, 2032, 48, 48, 48, 1020, 1020, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 960,
    4080, 7224, 6168, 14364, 12300, 12300, 12300, 12300, 14364, 6168, 7224, 4080, 992, 13280,
    16368, 7216, 0, 0, 0, 0, 0, 0, 0, 2046, 4094, 7192, 6168, 6168, 7192, 4088, 1016, 1816, 3608,
    3096, 7192, 30974, 28926, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7136, 8176, 7224, 6168, 6168, 120,
    1008, 4032, 7680, 6168, 6168, 7224, 4088, 2008, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16380, 16380,
    12684, 12684, 12684, 12684, 384, 384, 384, 384, 384, 384, 4080, 4080, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 32382, 32382, 6168, 6168, 6168, 6168, 6168, 6168, 6168, 6168, 6168, 3120, 4080, 960, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 65278, 65278, 12312, 6192, 6192, 6192, 3168, 3168, 1728, 1728, 1728,
    896, 896, 256, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 130175, 130175, 24588, 24588, 24844, 13208, 13208,
    14040, 14040, 15992, 7280, 7280, 6192, 6192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32382, 32382, 6168,
    3120, 1632, 960, 384, 384, 960, 1632, 3120, 6168, 32382, 32382, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    32318, 32318, 6168, 3120, 1632, 1632, 960, 384, 384, 384, 384, 384, 4080, 4080, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 8184, 8184, 6168, 3096, 1560, 792, 384, 192, 6240, 6192, 6168, 6156, 8188, 8188,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 3968, 3968, 384, 384, 384, 384, 384, 384, 384, 384, 384, 384, 384,
    384, 384, 384, 3968, 3968, 0, 0, 0, 0, 24, 24, 56, 48, 112, 96, 96, 192, 192, 384, 384, 768,
    768, 1536, 1536, 3584, 3072, 7168, 6144, 6144, 0, 0, 0, 0, 0, 0, 496, 496, 384, 384, 384, 384,
    384, 384, 384, 384, 384, 384, 384, 384, 384, 384, 496, 496, 0, 0, 0, 0, 0, 256, 896, 1984,
    3808, 3168, 6192, 12312, 8200, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65535, 65535, 0, 192, 448, 1792, 1536, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1008, 2040, 3072, 3072, 4064,
    4088, 3100, 3084, 3596, 16376, 15856, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30, 30, 24, 24, 2008, 8184,
    6200, 12312, 12312, 12312, 12312, 12312, 6200, 8190, 2014, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 14272, 16368, 14392, 12316, 12300, 12, 12, 12316, 14392, 8176, 4032, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 7680, 7680, 6144, 6144, 7136, 8184, 7192, 6156, 6156, 6156, 6156, 6156, 7192, 32760, 31712,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2016, 8184, 6168, 12300, 16380, 16380, 12, 12, 12312,
    16376, 4064, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16256, 16320, 96, 96, 8188, 8188, 96, 96, 96, 96, 96,
    96, 96, 4092, 4092, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 31712, 32760, 7192, 6156, 6156,
    6156, 6156, 6156, 7192, 8184, 7136, 6144, 6144, 7168, 4080, 1008, 0, 0, 0, 0, 30, 30, 24, 24,
    2008, 4088, 7224, 6168, 6168, 6168, 6168, 6168, 6168, 32382, 32382, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    384, 384, 0, 0, 504, 504, 384, 384, 384, 384, 384, 384, 384, 16380, 16380, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 768, 768, 0, 0, 4088, 4088, 3072, 3072, 3072, 3072, 3072, 3072, 3072, 3072, 3072, 3072,
    3072, 3584, 2040, 504, 0, 0, 0, 0, 60, 60, 48, 48, 7984, 7984, 816, 432, 496, 240, 496, 944,
    1840, 15932, 15932, 0, 0, 0, 0, 0, 0, 0, 0, 0, 504, 504, 384, 384, 384, 384, 384, 384, 384,
    384, 384, 384, 384, 16380, 16380, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7919, 16383, 13212,
    12684, 12684, 12684, 12684, 12684, 12684, 63423, 63423, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    2014, 4094, 7224, 6168, 6168, 6168, 6168, 6168, 6168, 32382, 32382, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 960, 4080, 7224, 14364, 12300, 12300, 12300, 14364, 7224, 4080, 960, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 2014, 8190, 6200, 12312, 12312, 12312, 12312, 12312, 6200, 8184, 2008,
    24, 24, 24, 254, 254, 0, 0, 0, 0, 0, 0, 0, 0, 31712, 32760, 7192, 6156, 6156, 6156, 6156, 6156,
    7192, 8184, 7136, 6144, 6144, 6144, 32512, 32512, 0, 0, 0, 0, 0, 0, 0, 0, 7804, 16252, 13280,
    224, 96, 96, 96, 96, 96, 4092, 4092, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8160, 8176, 6168,
    6168, 504, 4080, 7936, 6168, 7192, 4088, 2040, 0, 0, 0, 0, 0, 0, 0, 0, 0, 48, 48, 48, 48, 4092,
    4092, 48, 48, 48, 48, 48, 48, 14384, 16352, 4032, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7710,
    7710, 6168, 6168, 6168, 6168, 6168, 6168, 7192, 32752, 31712, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 31806, 31806, 6168, 6168, 3120, 3120, 1632, 1632, 2016, 960, 960, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 15390, 15390, 6284, 6604, 6604, 3416, 3960, 3960, 1592, 1584, 1584, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 15996, 15996, 3120, 1632, 960, 384, 960, 1632, 3120, 15996, 15996,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63614, 63614, 12312, 6192, 6192, 3168, 3168, 1728, 1984,
    896, 768, 384, 384, 192, 1020, 1020, 0, 0, 0, 0, 0, 0, 0, 0, 8184, 8184, 3096, 1560, 768, 384,
    192, 6240, 6192, 8184, 8184, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1792, 1920, 384, 384, 384, 384, 384,
    384, 448, 224, 448, 384, 384, 384, 384, 384, 1920, 1792, 0, 0, 0, 0, 0, 0, 384, 384, 384, 384,
    384, 384, 384, 384, 384, 384, 384, 384, 384, 384, 384, 384, 384, 384, 0, 0, 0, 0, 0, 0, 224,
    480, 384, 384, 384, 384, 384, 384, 896, 1792, 896, 384, 384, 384, 384, 384, 480, 224, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 112, 6392, 7644, 3980, 1792, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    16777088, 16777088, 16777088, 16777088, 16777088, 16777088, 16777088, 16777088, 16777088,
    16777088, 16777088, 16777088, 16777088, 16777088, 16777088, 16777088, 16777088, 16777088,
    16777088, 16777088, 16777088, 16777088, 16777088, 16777088,
];

// ----------------------------------------------------------------
// here comes the meaty-- erm, 'messy' part
// ----------------------------------------------------------------

pub struct Wayland {
    // the window state
    tx: Sender<Way>,
    rx: Receiver<Way>,
    display: Display,
    event_queue: EventQueue,
    dimensions: (u32, u32),
    pub window: Window<SimpleFrame>,
    pools: DoubleMemPool,
    pub seat: Proxy<wl_seat::WlSeat>,
    buffer: Option<Proxy<wl_buffer::WlBuffer>>,
    next_action: Arc<Mutex<Option<WEvent>>>,
    reader: Arc<Mutex<Option<ReadPipe>>>,
    paper_color: u32,
    ink_color: u32,
}

impl Wayland {
    pub fn new(mut xsize: u32, mut ysize: u32) -> Wayland {
        let (tx, rx) = channel::<Way>();
        let txc = tx.clone();
        let (display, mut event_queue) = Display::connect_to_env().unwrap();
        let env =
            Environment::from_registry(display.get_registry().unwrap(), &mut event_queue).unwrap();
        let surface = env.compositor
            .create_surface()
            .unwrap()
            .implement(|_, _| {});
        let next_action = Arc::new(Mutex::new(None::<WEvent>));
        let wind_action = next_action.clone();

        if xsize == 0 || ysize == 0 {
            env.outputs.with_all(|outputs| {
                for &(_, _, ref info) in outputs {
                    for mode in &info.modes {
                        if mode.is_current && xsize == 0 {
                            xsize = mode.dimensions.0 as u32
                        }
                        if mode.is_current && ysize == 0 {
                            ysize = mode.dimensions.1 as u32
                        }
                    }
                }
            });
        }

        let dimensions = (xsize, ysize);
        let mut window = Window::<SimpleFrame>::init(
            surface,
            dimensions,
            &env.compositor,
            &env.subcompositor,
            &env.shm,
            &env.shell,
            move |evt, ()| {
                let mut next_action = wind_action.lock().unwrap();
                // Keep last event in priority order : Close > Configure > Refresh
                let replace = match (&evt, &*next_action) {
                    (_, &None)
                    | (_, &Some(WEvent::Refresh))
                    | (&WEvent::Configure { .. }, &Some(WEvent::Configure { .. }))
                    | (&WEvent::Close, _) => true,
                    _ => false,
                };
                if replace {
                    *next_action = Some(evt);
                }
            },
        ).expect("Failed to create a Wayland window");

        let pools = DoubleMemPool::new(&env.shm).expect("Failed to create a memory pool !");
        let buffer = None;
        let seat = env.manager
            .instantiate_auto::<wl_seat::WlSeat>()
            .unwrap()
            .implement(move |_, _| {});

        let device =
            DataDevice::init_for_seat(&env.data_device_manager, &seat, |event: DndEvent, ()| {
                match event {
                    DndEvent::Enter {
                        //  we don't accept drag'n'drop
                        offer: Some(offer),
                        ..
                    } => offer.accept(None),
                    _ => (),
                }
            });

        window.new_seat(&seat);

        let themer = AutoThemer::init(None, env.compositor.clone(), env.shm.clone());
        let txpoi = txc.clone();
        let _pointer = themer.theme_pointer_with_impl(
            seat.get_pointer().unwrap(),
            move |event, _| match event {
                wl_pointer::Event::Enter { .. } => txpoi
                    .send(Way::Focus {
                        enter: true,
                        hover: true,
                        cause: 0,
                    })
                    .expect("sending user enter event"),
                wl_pointer::Event::Leave { .. } => txpoi
                    .send(Way::Focus {
                        enter: false,
                        hover: true,
                        cause: 0,
                    })
                    .expect("sending user leave event"),
                wl_pointer::Event::Motion {
                    surface_x,
                    surface_y,
                    ..
                } => txpoi
                    .send(Way::Pointer {
                        x: surface_x as u32,
                        y: surface_y as u32,
                    })
                    .expect("sending pointer event"),
                wl_pointer::Event::Button { button, state, .. } => txpoi
                    .send(Way::Button {
                        but: button,
                        status: state == wl_pointer::ButtonState::Pressed,
                    })
                    .expect("sending mouse button event"),
                _ => {}
            },
        );

        let reader = Arc::new(Mutex::new(None::<ReadPipe>));
        let readerclone = reader.clone();
        let txkeyb = txc.clone();
        let _keyboard = map_keyboard_auto(
            seat.get_keyboard().unwrap(),
            move |event: KbEvent, _| match event {
                KbEvent::Enter { keysyms, .. } => txkeyb
                    .send(Way::Focus {
                        enter: true,
                        hover: false,
                        cause: keysyms.len() as u32,
                    })
                    .expect("sending kbd gain-focus event"),
                KbEvent::Leave { .. } => txkeyb
                    .send(Way::Focus {
                        enter: false,
                        hover: false,
                        cause: 0,
                    })
                    .expect("sending kbd lost-focus event"),
                KbEvent::Key {
                    keysym,
                    utf8,
                    state,
                    modifiers,
                    ..
                } => {
                    let text = if let Some(txt) = utf8 {
                        txt
                    } else {
                        "".to_string()
                    };

                    // special action if Ctrl-V pressed:
                    //
                    if modifiers.ctrl && keysym == keysyms::XKB_KEY_v
                        && state == wl_keyboard::KeyState::Pressed
                    {
                        device.with_selection(|offer| {
                            if let Some(offer) = offer {
                                //print!("\nPASTE: current selection buffer mime types: [ ");
                                let mut has_text = false;
                                offer.with_mime_types(|types| {
                                    for t in types {
                                        //print!("\"{}\", ", t);
                                        if t == "text/plain;charset=utf-8" {
                                            has_text = true;
                                        }
                                    }
                                });
                                //println!("]");
                                if has_text {
                                    //println!("Buffer contains text, going to read it...");
                                    let mut reader = readerclone.lock().unwrap();
                                    *reader = Some(
                                        offer.receive("text/plain;charset=utf-8".into()).unwrap(),
                                    );
                                }
                            } else {
                                //println!("No current selection buffer!");
                            }
                        });
                    }

                    txkeyb
                        .send(Way::Key {
                            text,
                            keysym,
                            pressed: state == wl_keyboard::KeyState::Pressed,
                        })
                        .expect("sending kbd event")
                }
                KbEvent::RepeatInfo { rate, delay } => txkeyb
                    .send(Way::KeyInfo { rate, delay })
                    .expect("sending kbd rate/delay event"),
            },
        ).ok()
            .expect("cannot init keyboard/xkbcommon stuff");

        if !env.shell.needs_configure() {
            window.refresh();
            txc.send(Way::Refresh {
                width: dimensions.0 as usize,
                height: dimensions.1 as usize,
            }).expect("sending configure/refresh")
        }

        // default color scheme: green on white:
        let paper_color: u32 = 0;
        let ink_color: u32 = 0x00ff00;

        Wayland {
            tx,
            rx,
            display,
            event_queue,
            dimensions,
            window,
            pools,
            seat,
            buffer,
            next_action,
            reader,
            paper_color,
            ink_color,
        }
    }

    pub fn event(&mut self) -> Way {
        if let Ok(e) = self.rx.try_recv() {
            return e;
        }

        match self.next_action.lock().unwrap().take() {
            Some(WEvent::Close) => self.tx.send(Way::Exit).expect("sending user close event"),
            Some(WEvent::Refresh) => self.window.refresh(),
            Some(WEvent::Configure { new_size, .. }) => {
                if let Some((w, h)) = new_size {
                    self.window.resize(w, h);
                    if self.dimensions.0 != w || self.dimensions.1 != h {
                        self.dimensions = (w, h);
                        self.tx
                            .send(Way::Resize {
                                width: w as u32,
                                height: h as u32,
                            })
                            .expect("sending user resize event")
                    }
                }
                self.window.refresh();
                self.tx
                    .send(Way::Refresh {
                        width: self.dimensions.0 as usize,
                        height: self.dimensions.1 as usize,
                    })
                    .expect("sending refresh for resize")
            }
            None => {}
        }

        self.display.flush().unwrap();
        if let Some(mut rdr) = self.reader.lock().unwrap().take() {
            use std::io::Read;
            let mut text = String::new();
            rdr.read_to_string(&mut text).unwrap();
            return Way::Paste { text };
        }

        // don't block waiting for next event, because main() is actively polling
        //
        if let Some(guard) = self.event_queue.prepare_read() {
            guard.read_events().unwrap();
            self.event_queue.dispatch_pending().unwrap();
            Way::Idling { msec: 10 }
        } else {
            self.event_queue.dispatch_pending().unwrap();
            Way::Idling { msec: 15 }
        }
    }

    pub fn delay(&self, millis: u32) {
        self::std::thread::sleep(self::std::time::Duration::from_millis(millis as u64))
    }

    pub fn paper(&mut self, color: u32) {
        self.paper_color = color
    }

    pub fn ink(&mut self, color: u32) {
        self.ink_color = color
    }

    pub fn cls(&mut self) {
        let screensize = self.dimensions.0 as usize * self.dimensions.1 as usize;
        let color = self.paper_color;
        self.pixels(0, 0, color, screensize)
    }

    pub fn plot(&mut self, x: usize, y: usize) {
        let color = self.ink_color;
        self.pixels(x, y, color, 1)
    }

    pub fn print(&mut self, x: usize, y: usize, s: &str) {
        if s.len() > 0 && (x <= (self.dimensions.0 as usize - FONT_X_SIZE))
            && (y <= (self.dimensions.1 as usize - FONT_Y_SIZE))
        {
            let chars = if (s.len() * FONT_X_SIZE) > (self.dimensions.0 as usize - x) {
                (self.dimensions.0 as usize - x) / FONT_X_SIZE
            } else {
                s.len()
            };

            printstring(
                self.pools.pool(),
                &mut self.buffer,
                self.window.surface(),
                self.dimensions.0,
                self.dimensions.1,
                x,
                y,
                self.paper_color | 0xff000000,
                self.ink_color | 0xff000000,
                &s[0..chars],
            )
        }
    }

    fn pixels(&mut self, x: usize, y: usize, color: u32, rept: usize) {
        pixfill(
            self.pools.pool(),
            &mut self.buffer,
            self.window.surface(),
            self.dimensions.0,
            self.dimensions.1,
            x,
            y,
            color | 0xff000000,
            rept,
        )
    }
}

fn printstring(
    pool: &mut MemPool,
    buffer: &mut Option<Proxy<wl_buffer::WlBuffer>>,
    surface: &Proxy<wl_surface::WlSurface>,
    xsize: u32,
    ysize: u32,
    x: usize,
    y: usize,
    paper: u32,
    ink: u32,
    text: &str,
) {
    if let Some(b) = buffer.take() {
        b.destroy();
    }

    let bytes = (4 * xsize * ysize) as usize;
    pool.resize(bytes).expect("cannot resize the memory pool");

    let block_size = (text.len() * FONT_X_SIZE) as i64;
    let jump = 4 * (xsize as i64 - block_size);

    let start_pos = 4 * (y * xsize as usize + x) as u64;
    let _ = pool.seek(SeekFrom::Start(start_pos));
    {
        let mut writer = BufWriter::new(&mut *pool);

        for scanline in 0..FONT_Y_SIZE {
            for c in text.bytes() {
                let char_index: usize = if c < 32 || c > 127 {
                    63 - 32
                } else {
                    c as usize - 32
                };
                let bitmap = FONT[char_index * FONT_Y_SIZE + scanline];

                for i in 0..FONT_X_SIZE {
                    let color = if ((bitmap >> i) & 1) == 0 { paper } else { ink };
                    let _ = writer.write_u32::<NativeEndian>(color);
                }
            }

            if jump > 0 {
                let _ = writer.seek(SeekFrom::Current(jump));
            }
        }
    }

    let new_buffer = pool.buffer(
        0,
        xsize as i32,
        ysize as i32,
        4 * xsize as i32,
        wl_shm::Format::Argb8888,
    ).implement(|_, _| {});

    surface.attach(Some(&new_buffer), 0, 0);
    surface.commit();
    *buffer = Some(new_buffer);
}

fn pixfill(
    pool: &mut MemPool,
    buffer: &mut Option<Proxy<wl_buffer::WlBuffer>>,
    surface: &Proxy<wl_surface::WlSurface>,
    xsize: u32,
    ysize: u32,
    x: usize,
    y: usize,
    color: u32,
    reps: usize,
) {
    if let Some(b) = buffer.take() {
        b.destroy();
    }

    let bytes = (4 * xsize * ysize) as usize;
    pool.resize(bytes).expect("cannot resize the memory pool");

    let pos = 4 * (y * xsize as usize + x) as u64;
    let _ = pool.seek(SeekFrom::Start(pos));
    {
        let mut writer = BufWriter::new(&mut *pool);
        for _ in 0..reps {
            let _ = writer.write_u32::<NativeEndian>(color);
        }
    }

    let new_buffer = pool.buffer(
        0,
        xsize as i32,
        ysize as i32,
        4 * xsize as i32,
        wl_shm::Format::Argb8888,
    ).implement(|_, _| {});

    surface.attach(Some(&new_buffer), 0, 0);
    surface.commit();
    *buffer = Some(new_buffer);
}
