use core::future::Future;

pub trait Bus {
    fn read(self: &mut Self, register: u8) -> impl Future<Output = u8>;
    fn write(self: &mut Self, register: u8, value: u8) -> impl Future<Output = ()>;
}
