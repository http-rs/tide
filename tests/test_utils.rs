use portpicker::pick_unused_port;

/// Find an unused port.
#[allow(dead_code)]
pub async fn find_port() -> u16 {
    pick_unused_port().expect("No ports free")
}
