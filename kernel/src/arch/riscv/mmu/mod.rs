#[cfg(feature = "sv39")]
mod sv39;

pub use sv39::*;

pub fn set_root_page_table(root_table: PhysicalAddress) {
    riscv::write_satp(
        riscv::registers::Satp::empty()
            .set_mode(riscv::registers::SatpMode::Sv39)
            .set_ppn(root_table.raw()),
    );
}

pub fn pa_to_satp(root_table: PhysicalAddress) -> usize {
    riscv::registers::Satp::empty()
        .set_mode(riscv::registers::SatpMode::Sv39)
        .set_ppn(root_table.raw())
        .raw() as usize
}
