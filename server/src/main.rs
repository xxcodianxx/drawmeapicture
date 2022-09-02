use actix_files::NamedFile;
use actix_web::{post, web, App, HttpRequest, HttpServer, Responder, Result};

use colors_transform::Color;
use log::{debug, info, warn};
use rgb565::Rgb565;

use serde::Deserialize;

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Deserialize)]
struct MoveRequest {
    // test: String,
    moves: Vec<(u16, u16, String)>,
}

#[post("/draw")]
async fn draw(
    req: HttpRequest,
    json: web::Json<MoveRequest>,
    screen: web::Data<Arc<Screen>>,
) -> impl Responder {
    let ipaddr = match req.peer_addr() {
        Some(addr) => addr.ip().to_string(),
        None => "???".to_owned(),
    };

    tokio::task::spawn_blocking(move || {
        let mut serial = screen.serial.lock().unwrap();

        info!("Drawing from {}... ({} calls)", ipaddr, json.moves.len());

        serial
            .write_all(&make_drawcall_buffer(1, 1, &[1, 1])) // 0x01 0x01 0x01 0x01 (clear screen)
            .expect("could not write clear screen command, aborting");

        json.moves.iter().for_each(|(x, y, hex_color)| {
            let color = colors_transform::Rgb::from_hex_str(hex_color).unwrap();

            let color = Rgb565::from_rgb888_components(
                color.get_red() as u8,
                color.get_green() as u8,
                color.get_blue() as u8,
            )
            .to_rgb565_le();

            let drawcall = make_drawcall_buffer(*x, *y, &color);

            loop {
                serial
                    .write_all(&drawcall)
                    .expect("could not write drawcall, aborting");

                let mut resp = [0u8];
                serial
                    .read(&mut resp)
                    .expect("could not read drawcall ack, aborting");

                match resp[0] {
                    0x01 => {
                        break;
                    }
                    0x02 => {
                        warn!("write error, retransmit");
                    }
                    v => {
                        warn!("weird resp: {}", v);
                    }
                }
            }
        });

        info!("Drawing done.")
    });

    return "ok";
}

async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("index.html")?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let screen;

    loop {
        match Screen::connect() {
            Ok(s) => {
                screen = Arc::new(s);
                break;
            }
            Err(e) => warn!("Handshake error: {}", e),
        };
    }

    info!("Reported screen size: {}x{}", screen.width, screen.height);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(screen.clone()))
            .service(draw)
            .route("/", web::get().to(index))
    })
    .bind("0.0.0.0:55557")?
    .run()
    .await?;

    Ok(())
}

struct Screen {
    // mutex so 2 people can't write to the port at the same time, wait your turn
    serial: Mutex<Box<dyn serialport::SerialPort>>,
    width: u16,
    height: u16,
}

impl Screen {
    pub fn connect() -> Result<Self, Box<dyn Error>> {
        info!("Opening serial connection, Arduino WILL reset!");

        let mut serial = serialport::new("/dev/ttyACM1", 9600)
            .timeout(Duration::from_secs(5))
            .open()
            .expect("Failed to open port.");

        info!("Serial open, sleeping 5s for safe measure...");

        std::thread::sleep(Duration::from_secs(5));

        info!("Reading handshake...");

        let mut buf = [0u8; 5 + 2 + 2];
        let size = serial.read(&mut buf)?;

        debug!("{:?}", buf);

        if &buf[..5] != "HELLO".as_bytes() {
            Err("Bad hello (probably garbage bytes on wire)")?
        }

        if size != 5 + 2 + 2 {
            Err("Invalid handshake size read")?
        }

        let width = u16::from_be_bytes(buf[5..7].try_into()?);
        let height = u16::from_be_bytes(buf[7..9].try_into()?);

        Ok(Self {
            serial: Mutex::new(serial),
            width,
            height,
        })
    }
}

fn make_drawcall_buffer(x: u16, y: u16, color: &[u8; 2]) -> [u8; 10] {
    let mut buf = [0u8; 10];

    buf[0..2].copy_from_slice(&x.to_le_bytes());
    buf[2..4].copy_from_slice(&y.to_le_bytes());
    buf[4..6].copy_from_slice(color);

    let checksum = crc32fast::hash(&buf[..6]);

    buf[6..10].copy_from_slice(&checksum.to_le_bytes());

    return buf;
}
