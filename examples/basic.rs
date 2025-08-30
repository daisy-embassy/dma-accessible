#![no_std]
#![no_main]
use cortex_m_rt::entry;
use defmt::info;
use dma_accessible::{DmaAccessible, DmaBuffer, Sram1};

use embassy_executor::Spawner;
use embassy_stm32::{
    Peripherals, bind_interrupts,
    i2c::{self, I2c},
    peripherals,
    time::Hertz,
};
use grounded::uninit::GroundedArrayCell;

use defmt_rtt as _;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello, basic test!");
    let p = embassy_stm32::init(Default::default());
    test_dma_buffer_creation(p).await;
    info!("test has completed. no problem!!");
    loop {}
}

async fn test_dma_buffer_creation(p: Peripherals) {
    info!("test_dma_buffer_creation");
    let mut i2c = I2c::new(
        p.I2C2,
        p.PB10,
        p.PB11,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH5,
        Hertz::khz(100),
        Default::default(),
    );
    // Buffer must be placed in a DMA-accessible region (e.g., SRAM1)
    #[unsafe(link_section = ".sram1_bss")]
    static BUFFER: GroundedArrayCell<u8, 1024> = GroundedArrayCell::uninit();

    let mut dma_buffer = DmaBuffer::<u8, 1024, Sram1>::new(&BUFFER, 0);
    assert_eq!(dma_buffer.len(), 1024);
    const ADDRESS: u8 = 0x5F;
    const WHOAMI: u8 = 0x0F;
    assert_eq!(
        i2c.write_read(ADDRESS, &[WHOAMI], &mut dma_buffer).await,
        Err(i2c::Error::Timeout)
    );
    // Additional test logic can be added here
    simple_dma_transfer(dma_buffer);
}

fn simple_dma_transfer<T: DmaAccessible, const LEN: usize>(src: DmaBuffer<u8, LEN, T>) {}
