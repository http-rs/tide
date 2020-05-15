use portpicker::pick_unused_port;

pub async fn find_port() -> u16 {
    pick_unused_port().expect("No ports free")
}
