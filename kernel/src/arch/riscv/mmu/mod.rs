#[cfg(feature = "sv39")]
mod sv39;

#[cfg(feature = "sv39")]
pub use sv39::*;

pub fn set_root_page_table_pa(root_table: PhysicalAddress) {
    riscv::write_satp_tlb_flush(
        riscv::registers::Satp::empty()
            .set_mode(riscv::registers::SatpMode::Sv39)
            .set_ppn(root_table.raw() as usize),
    );
}

pub fn set_root_page_table(val: usize) {
    riscv::write_satp_tlb_flush(riscv::registers::Satp::new(val));
}

pub fn pa_to_satp(root_table: PhysicalAddress) -> usize {
    riscv::registers::Satp::empty()
        .set_mode(riscv::registers::SatpMode::Sv39)
        .set_ppn(root_table.raw() as usize)
        .raw() as usize
}
