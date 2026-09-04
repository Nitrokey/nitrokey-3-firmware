use usb_device::endpoint::{EndpointAddress, EndpointType};

/// USB endpoint descriptor information.
pub struct EndpointDescriptor {
    /// Endpoint address.
    pub address: EndpointAddress,

    /// Endpoint transfer type.
    pub ep_type: EndpointType,

    /// Maximum packet size.
    pub max_packet_size: u16,

    /// Poll interval for interrupt endpoints.
    pub interval: u8,
}

/// Configuration for an endpoint allocation.
pub struct EndpointConfig {
    /// The transfer type of the endpoint to be allocated.
    pub ep_type: EndpointType,

    /// Maximum packet size for the endpoint to be allocated.
    pub max_packet_size: u16,

    /// Poll interval for interrupt endpoints.
    pub interval: u8,

    /// Requests a specific endpoint number. Allocation shall fail if the number is not available.
    pub number: Option<u8>,

    /// Specifies that the endpoint is the "pair" of another endpoint.
    ///
    /// If `ep` is an endpoint in the opposite direction, this means that the endpoint to be
    /// allocated uses the same transfer type as `ep` but in the opposite direction in all alternate
    /// settings for the interface.
    ///
    /// If `ep` is an endpoint in the same direction, this means that the two endpoints belong to
    /// different alternate settings for the interface and may never be enabled at the same time.
    pub pair_of: Option<EndpointAddress>,
}
