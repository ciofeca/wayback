mod boilerplate;
use boilerplate::Way::*; // Exit, Idling, Refresh...

fn main() {
    let mut w = boilerplate::Wayland::new(640, 480); // use 0,0 to start maximized
    loop {
        match w.event() {
            Exit => break,

            Idling { msec } => w.delay(msec), // wait for the suggested delay

            Refresh { width, height } => {
                if width != 0 {
                    w.paper(0x0e0000); // usual 0xRRGGBB coding
                    w.ink(0x00ff00);
                    w.cls();
                    w.print(0, 0, &format!("Welcome to pid {}", std::process::id()));
                    let bottom_row = height - boilerplate::FONT_Y_SIZE;
                    w.print(0, bottom_row, "Smithay-powered Wayland client");
                }
            }

            Resize { width, height } => println!("- resize {}x{}", width, height),

            Focus {
                enter: true,
                hover, // true if pointer was hovering, false if from keyboard
                cause,
            } => w.print(0, 32, &format!("enter:  {} / {}  ", hover, cause)),

            Focus {
                enter: false,
                hover,
                cause,
            } => w.print(0, 32, &format!("leave:  {} / {}  ", hover, cause)),

            KeyInfo { rate, delay } => w.print(0, 128, &format!("kconf:  {}ms/{}ms", rate, delay)),

            Key {
                text, // utf8
                keysym,
                pressed,
            } => {
                w.print(0, 96, &format!("keycode {:04x}: {} ", keysym, pressed));
                if text.len() > 0 {
                    println!("- key: [{}]", text)
                }
            }

            Paste { text } => println!("Pastebuffer: {:?}", text),

            Pointer { x, y } => w.plot(x as usize, y as usize),

            Button { but, status } => w.print(0, 64, &format!("button: {:04x}: {} ", but, status)),
        }
    }
}
