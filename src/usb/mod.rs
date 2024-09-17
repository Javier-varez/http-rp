mod reset;

use embassy_usb::class::cdc_acm::{self, CdcAcmClass};
use static_cell::make_static;

const MAX_PACKET_SIZE: u8 = 64;

struct State {
    config_descriptor: [u8; 128],
    bos_descriptor: [u8; 16],
    msos_descriptor: [u8; 256],
    control_buf: [u8; 64],
}

impl State {
    fn new() -> Self {
        State {
            config_descriptor: [0; 128],
            bos_descriptor: [0; 16],
            msos_descriptor: [0; 256],
            control_buf: [0; 64],
        }
    }
}

pub struct Device<'d, D>
where
    D: embassy_usb::driver::Driver<'d>,
{
    device: embassy_usb::UsbDevice<'d, D>,
    cdc_acm: CdcAcmClass<'d, D>,
    _reset_iface: reset::ResetInterface,
}

impl<'d, D> Device<'d, D>
where
    D: embassy_usb::driver::Driver<'d>,
    Self: 'd,
{
    pub fn new(driver: D) -> Self {
        let mut config = embassy_usb::Config::new(0x2e8a, 0x0009);
        config.manufacturer = Some("Javier Alvarez");
        config.product = Some("http-rp");
        config.serial_number = None;
        config.max_power = 100;
        config.max_packet_size_0 = MAX_PACKET_SIZE;

        config.device_class = 0xef;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;

        let state = make_static!(State::new());

        let mut builder = embassy_usb::Builder::new(
            driver,
            config,
            &mut state.config_descriptor,
            &mut state.bos_descriptor,
            &mut state.msos_descriptor,
            &mut state.control_buf,
        );

        let _reset_iface = reset::ResetInterface::new(&mut builder);

        let cdc_state = make_static!(cdc_acm::State::new());

        // SAFETY: Turns out that I don't have a way to convince the compiler that the
        // state is statically allocated, and that the lifetime parameter of the State
        // being 'static is actually ok for the device, which should also be static in
        // practice.
        let cdc_state: &'d mut cdc_acm::State<'d> = unsafe {
            let ptr: *mut cdc_acm::State<'static> = cdc_state;
            let ptr = ptr as *mut cdc_acm::State<'d>;
            &mut *ptr
        };

        let cdc_acm = CdcAcmClass::new(&mut builder, cdc_state, MAX_PACKET_SIZE.into());

        let device = builder.build();

        Self {
            device,
            cdc_acm,
            _reset_iface,
        }
    }

    pub async fn run(self) -> ! {
        let Self {
            mut device,
            cdc_acm,
            ..
        } = self;
        let run_device = async move {
            loop {
                device.run().await;
            }
        };
        let logger = embassy_usb_logger::with_class!(1024, log::LevelFilter::Info, cdc_acm);
        let joined = embassy_futures::join::join(run_device, logger);
        joined.await;
        // Both embassy_usb_logger::with_class and run_device never finish execution.
        unreachable!()
    }
}
