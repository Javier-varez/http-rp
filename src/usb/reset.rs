use static_cell::make_static;

pub struct ResetInterface {}

impl ResetInterface {
    pub fn new<'d, D>(builder: &mut embassy_usb::Builder<'d, D>) -> Self
    where
        D: embassy_usb::driver::Driver<'d>,
    {
        const CLASS_VENDOR_SPECIFIC: u8 = 0xFF;
        const RESET_IFACE_SUBCLASS: u8 = 0x00;
        const RESET_IFACE_PROTOCOL: u8 = 0x01;
        let mut function = builder.function(
            CLASS_VENDOR_SPECIFIC,
            RESET_IFACE_SUBCLASS,
            RESET_IFACE_PROTOCOL,
        );

        // Control interface
        let mut iface = function.interface();
        let iface_num = iface.interface_number();
        iface.alt_setting(
            CLASS_VENDOR_SPECIFIC,
            RESET_IFACE_SUBCLASS,
            RESET_IFACE_PROTOCOL,
            None,
        );

        drop(function);

        let handler = make_static!(ResetHandler::new(iface_num));
        builder.handler(handler);
        Self {}
    }
}

struct ResetHandler {
    iface_num: embassy_usb::types::InterfaceNumber,
}

impl ResetHandler {
    fn new(iface_num: embassy_usb::types::InterfaceNumber) -> Self {
        Self { iface_num }
    }
}

impl embassy_usb::Handler for ResetHandler {
    fn control_in<'a>(
        &'a mut self,
        req: embassy_usb::control::Request,
        _buf: &'a mut [u8],
    ) -> Option<embassy_usb::control::InResponse<'a>> {
        if !(req.request_type == embassy_usb::control::RequestType::Class
            && req.recipient == embassy_usb::control::Recipient::Interface
            && req.index == u8::from(self.iface_num) as u16)
        {
            return None;
        }
        // we are not expecting any USB IN requests
        Some(embassy_usb::control::InResponse::Rejected)
    }

    fn control_out(
        &mut self,
        req: embassy_usb::control::Request,
        _data: &[u8],
    ) -> Option<embassy_usb::control::OutResponse> {
        if !(req.request_type == embassy_usb::control::RequestType::Class
            && req.recipient == embassy_usb::control::Recipient::Interface
            && req.index == u8::from(self.iface_num) as u16)
        {
            return None;
        }

        const RESET_REQUEST_BOOTSEL: u8 = 0x01;
        const RESET_REQUEST_FLASH: u8 = 0x02;

        match req.request {
            RESET_REQUEST_BOOTSEL => {
                embassy_rp::rom_data::reset_to_usb_boot(0, u32::from(req.value & 0x7F));
                // no-need to accept/reject, we'll reset the device anyway
                unreachable!()
            }
            RESET_REQUEST_FLASH => todo!(),
            _ => {
                // we are not expecting any other USB OUT requests
                Some(embassy_usb::control::OutResponse::Rejected)
            }
        }
    }
}
