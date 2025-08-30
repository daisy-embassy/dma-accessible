#![no_std]
#![no_main]
use defmt::info;
use dma_accessible::{DmaAccessible, DmaBuffer, Dtcm, Itcm, Sram1};

use embassy_executor::Spawner;
use embassy_stm32::{
    Peripherals, bind_interrupts,
    i2c::{self, I2c, Master},
    mode::Async,
    peripherals,
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
        Default::default(),
    );

    {
        info!("sram test");
        #[unsafe(link_section = ".sram1_bss")]
        static BUFFER_SRAM: GroundedArrayCell<u8, 1024> = GroundedArrayCell::uninit();
        let dma_buffer = DmaBuffer::<u8, 1024, Sram1>::new(&BUFFER_SRAM, 0);
        simple_dma_transfer(dma_buffer, &mut i2c).await;
    }
    {
        info!("itcm test");
        #[unsafe(link_section = ".itcm_bss")]
        static BUFFER_ITCM: GroundedArrayCell<u8, 1024> = GroundedArrayCell::uninit();
        let dma_buffer = DmaBuffer::<u8, 1024, Itcm>::new(&BUFFER_ITCM, 0);
        simple_dma_transfer(dma_buffer, &mut i2c).await;
    }
    {
        info!("dtcm test");
        #[unsafe(link_section = ".dtcm_bss")]
        static BUFFER_DTCM: GroundedArrayCell<u8, 1024> = GroundedArrayCell::uninit();
        let dma_buffer = DmaBuffer::<u8, 1024, Dtcm>::new(&BUFFER_DTCM, 0);
        simple_dma_transfer(dma_buffer, &mut i2c).await;
    }
}

async fn simple_dma_transfer<T: DmaAccessible, const LEN: usize>(
    mut src: DmaBuffer<u8, LEN, T>,
    i2c: &mut I2c<'_, Async, Master>,
) {
    const ADDRESS: u8 = 0x5F;
    const WHOAMI: u8 = 0x0F;
    assert_eq!(
        i2c.write_read(ADDRESS, &[WHOAMI], &mut src).await,
        Err(i2c::Error::Timeout)
    );
}
