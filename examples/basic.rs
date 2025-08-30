#![no_std]
#![no_main]
use cortex_m_rt::entry;
use defmt::info;
use dma_accessible::{DmaBuffer, Sram1};
use grounded::uninit::GroundedArrayCell;

use defmt_rtt as _;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Hello, basic test!");
    test_dma_buffer_creation();
    info!("test has completed. no problem!!");
    loop {}
}

fn test_dma_buffer_creation() {
    info!("test_dma_buffer_creation");
    // Buffer must be placed in a DMA-accessible region (e.g., SRAM1)
    #[unsafe(link_section = ".sram1_bss")]
    static BUFFER: GroundedArrayCell<u8, 1024> = GroundedArrayCell::uninit();

    let dma_buffer = DmaBuffer::<u8, 1024, Sram1>::new(&BUFFER, 0);

    // Additional test logic can be added here
    assert_eq!(dma_buffer.len(), 1024);
}
