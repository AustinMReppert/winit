use winit_core::keyboard::{KeyCode, PhysicalKey};

pub fn physicalkey_to_scancode(physical_key: PhysicalKey) -> Option<u32> {
    None
}

pub fn scancode_to_physicalkey(scancode: u32) -> PhysicalKey {
    PhysicalKey::Code(KeyCode::Unidentified)
}