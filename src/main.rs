#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

pub mod http;
pub mod usb;

use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::Pio;
use panic_probe as _;
use picoserve::{
    extract::State,
    routing::{get, post},
};
use static_cell::make_static;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<embassy_rp::peripherals::PIO0>;
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
});

#[derive(Clone)]
struct WebState {
    control: &'static core::cell::RefCell<&'static mut cyw43::Control<'static>>,
}

async fn post_led_state(
    led_state: LedState,
    State(state): State<WebState>,
) -> impl picoserve::response::IntoResponse {
    log::info!("Get /led/{led_state}");
    let mut control = state.control.borrow_mut();
    control.gpio_set(0, led_state.into()).await;
    picoserve::response::Response::new(picoserve::response::StatusCode::OK, "")
}

async fn get_static_js_page(
    page: heapless::String<32>,
    _: State<WebState>,
) -> impl picoserve::response::IntoResponse {
    log::info!("Get /static/js/{page}");
    http::js::get_resource(&page)
}

async fn get_static_css_page(
    page: heapless::String<32>,
    _: State<WebState>,
) -> impl picoserve::response::IntoResponse {
    log::info!("Get /static/css/{page}");
    http::css::get_resource(&page)
}

async fn get_html_page(
    page: heapless::String<32>,
    _: State<WebState>,
) -> impl picoserve::response::IntoResponse {
    log::info!("Get /{page}");
    http::html::get_resource(&page)
}

#[embassy_executor::task]
async fn usb_task(usb: embassy_rp::peripherals::USB) {
    let driver = embassy_rp::usb::Driver::new(usb, Irqs);
    let device = usb::Device::new(driver);
    device.run().await
}

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static embassy_net::Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

type AppRouter = impl picoserve::routing::PathRouter<WebState>;

const WEB_TASK_POOL_SIZE: usize = 8;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    id: usize,
    stack: &'static embassy_net::Stack<cyw43::NetDriver<'static>>,
    app: &'static picoserve::Router<AppRouter, WebState>,
    config: &'static picoserve::Config<embassy_time::Duration>,
    control: &'static core::cell::RefCell<&'static mut cyw43::Control<'static>>,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::listen_and_serve_with_state::<WebState, AppRouter>(
        id,
        app,
        config,
        stack,
        port,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
        &WebState { control },
    )
    .await
}

#[derive(Debug, Clone, Copy)]
enum LedState {
    Off,
    On,
}

impl core::str::FromStr for LedState {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(LedState::On),
            "off" => Ok(LedState::Off),
            _ => Err(()),
        }
    }
}

impl core::fmt::Display for LedState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let str = match self {
            LedState::On => "on",
            LedState::Off => "off",
        };

        write!(f, "{}", str)
    }
}

impl Into<bool> for LedState {
    fn into(self) -> bool {
        match self {
            LedState::On => true,
            LedState::Off => false,
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let clock_config = embassy_rp::clocks::ClockConfig::crystal(12_000_000);
    let config = embassy_rp::config::Config::new(clock_config);
    let p = embassy_rp::init(config);

    embassy_rp::install_core0_stack_guard().unwrap();

    spawner.must_spawn(usb_task(p.USB));

    let fw = include_bytes!("../cyw43_fw/43439A0.bin");
    let clm = include_bytes!("../cyw43_fw/43439A0_clm.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.spawn(cyw43_task(runner)).unwrap();

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    control
        .join_wpa2(core::env!("SSID"), core::env!("SSID_PASS"))
        .await
        .unwrap();

    let self_ip = embassy_net::Ipv4Cidr::from_netmask(
        embassy_net::Ipv4Address::new(192, 168, 1, 190),
        embassy_net::Ipv4Address::new(0xff, 0xff, 0xff, 0x00),
    )
    .unwrap();
    let gateway_ip = embassy_net::Ipv4Address::new(192, 168, 1, 1);
    let mut dns_servers = heapless::Vec::new();
    dns_servers
        .push(embassy_net::Ipv4Address::new(8, 8, 8, 8))
        .unwrap();
    dns_servers
        .push(embassy_net::Ipv4Address::new(8, 8, 4, 4))
        .unwrap();
    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: self_ip,
        gateway: Some(gateway_ip),
        dns_servers,
    });

    let resources = make_static!(embassy_net::StackResources::<WEB_TASK_POOL_SIZE>::new());
    let stack = make_static!(embassy_net::Stack::new(_net_device, config, resources, 120));

    spawner.spawn(net_task(stack)).unwrap();

    let control = make_static!(control);
    let control = make_static!(core::cell::RefCell::new(control));

    fn make_app() -> picoserve::Router<AppRouter, WebState> {
        picoserve::Router::new()
            .route(
                picoserve::routing::parse_path_segment::<heapless::String<32>>(),
                get(get_html_page),
            )
            .route(
                (
                    "/static/js",
                    picoserve::routing::parse_path_segment::<heapless::String<32>>(),
                ),
                get(get_static_js_page),
            )
            .route(
                (
                    "/static/css",
                    picoserve::routing::parse_path_segment::<heapless::String<32>>(),
                ),
                get(get_static_css_page),
            )
            .route(
                ("/led", picoserve::routing::parse_path_segment::<LedState>()),
                post(post_led_state),
            )
    }

    let app = make_static!(make_app());

    let config = make_static!(picoserve::Config::new(picoserve::Timeouts {
        start_read_request: Some(embassy_time::Duration::from_secs(5)),
        read_request: Some(embassy_time::Duration::from_secs(1)),
        write: Some(embassy_time::Duration::from_secs(1)),
    })
    .keep_connection_alive());

    for id in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(web_task(id, stack, app, config, control));
    }
}
