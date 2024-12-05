use embassy_usb::driver::{
    Bus, ControlPipe, Driver, Endpoint, EndpointAddress, EndpointAllocError, EndpointError,
    EndpointIn, EndpointInfo, EndpointOut, EndpointType, Event, Unsupported,
};

/// Driver that implements [`embassy_usb::driver::Driver`].
pub struct UsbDriver;

impl<'a> Driver<'a> for UsbDriver {
    type EndpointOut = DummyEndpointOut;
    type EndpointIn = DummyEndpointIn;
    type ControlPipe = DummyControlPipe;
    type Bus = DummyBus;

    fn alloc_endpoint_out(
        &mut self,
        _ep_type: EndpointType,
        _max_packet_size: u16,
        _interval_ms: u8,
    ) -> Result<Self::EndpointOut, EndpointAllocError> {
        unimplemented!();
    }
    fn alloc_endpoint_in(
        &mut self,
        _ep_type: EndpointType,
        _max_packet_size: u16,
        _interval_ms: u8,
    ) -> Result<Self::EndpointIn, EndpointAllocError> {
        unimplemented!();
    }
    fn start(self, _control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
        unimplemented!();
    }
}

pub struct Peripherals {}

impl Peripherals {
    pub fn new(_peripherals: &mut crate::OptionalPeripherals) -> Self {
        unimplemented!();
    }
}

#[must_use]
pub fn driver(_peripherals: Peripherals) -> UsbDriver {
    unimplemented!();
}

pub struct DummyEndpointOut;

impl Endpoint for DummyEndpointOut {
    fn info(&self) -> &EndpointInfo {
        unimplemented!();
    }
    async fn wait_enabled(&mut self) {
        unimplemented!();
    }
}

impl EndpointOut for DummyEndpointOut {
    async fn read(&mut self, _buf: &mut [u8]) -> Result<usize, EndpointError> {
        unimplemented!();
    }
}

pub struct DummyEndpointIn;

impl Endpoint for DummyEndpointIn {
    fn info(&self) -> &EndpointInfo {
        unimplemented!();
    }
    async fn wait_enabled(&mut self) {
        unimplemented!();
    }
}

impl EndpointIn for DummyEndpointIn {
    async fn write(&mut self, _buf: &[u8]) -> Result<(), EndpointError> {
        unimplemented!();
    }
}

pub struct DummyControlPipe;

impl ControlPipe for DummyControlPipe {
    fn max_packet_size(&self) -> usize {
        unimplemented!();
    }
    async fn setup(&mut self) -> [u8; 8] {
        unimplemented!();
    }
    async fn data_out(
        &mut self,
        _buf: &mut [u8],
        _first: bool,
        _last: bool,
    ) -> Result<usize, EndpointError> {
        unimplemented!();
    }
    async fn data_in(
        &mut self,
        _data: &[u8],
        _first: bool,
        _last: bool,
    ) -> Result<(), EndpointError> {
        unimplemented!();
    }
    async fn accept(&mut self) {
        unimplemented!();
    }
    async fn reject(&mut self) {
        unimplemented!();
    }
    async fn accept_set_address(&mut self, _addr: u8) {
        unimplemented!();
    }
}

pub struct DummyBus;

impl Bus for DummyBus {
    async fn enable(&mut self) {
        unimplemented!();
    }
    async fn disable(&mut self) {
        unimplemented!();
    }
    async fn poll(&mut self) -> Event {
        unimplemented!();
    }
    fn endpoint_set_enabled(&mut self, _ep_addr: EndpointAddress, _enabled: bool) {
        unimplemented!();
    }
    fn endpoint_set_stalled(&mut self, _ep_addr: EndpointAddress, _stalled: bool) {
        unimplemented!();
    }
    fn endpoint_is_stalled(&mut self, _ep_addr: EndpointAddress) -> bool {
        unimplemented!();
    }
    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        unimplemented!();
    }
}
