mod reset;

use static_cell::make_static;

struct Status {
    config_descriptor: [u8; 128],
    bos_descriptor: [u8; 16],
    msos_descriptor: [u8; 256],
    control_buf: [u8; 64],
}

pub struct Device<'d, D>
where
    D: embassy_usb::driver::Driver<'d>,
{
    device: embassy_usb::UsbDevice<'d, D>,
    _reset_iface: reset::ResetInterface,
}

impl<'d, D> Device<'d, D>
where
    D: embassy_usb::driver::Driver<'d>,
{
    pub fn new(driver: D) -> Self {
        let mut config = embassy_usb::Config::new(0x2e8a, 0x0009);
        config.manufacturer = Some("Javier Alvarez");
        config.product = Some("rp2040");
        config.serial_number = None;
        config.max_power = 100;
        const MAX_PACKET_SIZE: u8 = 64;
        config.max_packet_size_0 = MAX_PACKET_SIZE;

        config.device_class = 0xef;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;

        let status = make_static!(Status {
            config_descriptor: [0; 128],
            bos_descriptor: [0; 16],
            msos_descriptor: [0; 256],
            control_buf: [0; 64],
        });

        let mut builder = embassy_usb::Builder::new(
            driver,
            config,
            &mut status.config_descriptor,
            &mut status.bos_descriptor,
            &mut status.msos_descriptor,
            &mut status.control_buf,
        );

        let reset_iface = reset::ResetInterface::new(&mut builder);

        let device = builder.build();
        Self {
            device,
            _reset_iface: reset_iface,
        }
    }

    pub async fn run(&mut self) -> ! {
        loop {
            self.device.run().await;
        }
    }
}
