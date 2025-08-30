#![no_std]
#![no_main]
use cortex_m_rt::entry;
use dma_accessible::{DmaBuffer, Sram1};
use grounded::uninit::GroundedArrayCell;
use panic_halt as _;

#[entry]
fn main() -> ! {
    test_dma_buffer_creation();
    loop {}
}

fn test_dma_buffer_creation() {
    // Buffer must be placed in a DMA-accessible region (e.g., SRAM1)
    #[unsafe(link_section = ".sram1_bss")]
    static BUFFER: GroundedArrayCell<u8, 1024> = GroundedArrayCell::uninit();

    let dma_buffer = DmaBuffer::<u8, 1024, Sram1>::new(&BUFFER, 0);

    // Additional test logic can be added here
    assert_eq!(dma_buffer.len(), 1024);
}
