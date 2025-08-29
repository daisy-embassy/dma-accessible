#![no_std]

use core::marker::PhantomData;
use core::ptr::NonNull;

// DMAアクセス可能なメモリ領域を表すトレイト
pub trait DmaAccessible {
    const START_ADDR: usize;
    const END_ADDR: usize;
}

// 具体的なメモリ領域の型（STM32H750の例）
pub struct Sram1;
pub struct Dtcm;
pub struct Itcm;

impl DmaAccessible for Sram1 {
    const START_ADDR: usize = 0x3000_0000; // SRAM1開始アドレス (RM0433, p. 131)
    const END_ADDR: usize = 0x3002_0000;   // SRAM1終了アドレス 
}

impl DmaAccessible for Dtcm {
    const START_ADDR: usize = 0x2000_0000; // DTCM開始アドレス (RM0433, p. 131)
    const END_ADDR: usize = 0x2001_0000;   // DTCM終了アドレス 
}

impl DmaAccessible for Itcm {
    const START_ADDR: usize = 0x0000_0000; // ITCM開始アドレス (RM0433, p. 131)
    const END_ADDR: usize = 0x0001_0000;   // ITCM終了アドレス
}

// DMAバッファのラッパー型
pub struct DmaBuffer<T, Region: DmaAccessible> {
    ptr: NonNull<T>,
    len: usize,
    _region: PhantomData<Region>,
}

impl<T, Region: DmaAccessible> DmaBuffer<T, Region> {
    // 安全なコンストラクタ: 特定の領域に配置されたバッファのみ受け付け
    pub fn new(buffer: &'static mut [T]) -> Self {
        let addr = buffer.as_ptr() as usize;
        // コンパイル時/ランタイムでアドレス範囲チェック
        assert!(
            addr >= Region::START_ADDR && (addr + core::mem::size_of_val(buffer)) <= Region::END_ADDR,
            "Buffer not in DMA-accessible region"
        );
        
        let len = buffer.len();
        Self {
            ptr: NonNull::from(buffer).cast(),
            len,
            _region: PhantomData,
        }
    }

    // embassyのDMA転送用に借用を提供（転送中はバッファを変更不可）
    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    // 内部ポインタ取得（embassyのDMA関数に渡す用; unsafeだが型で保証済み）
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
