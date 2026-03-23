#[cfg(feature = "sv39")]
mod sv39;

pub use sv39::*;

pub fn set_root_page_table(root_table: PhysicalAddress) {
    riscv::registers::Satp::empty()
        .set_mode(riscv::registers::SatpMode::Sv39)
        .set_ppn(root_table.raw())
        .write();
}
