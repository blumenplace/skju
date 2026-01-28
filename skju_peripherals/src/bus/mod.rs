use core::future::Future;

pub trait Bus {    
    fn send(&mut self, bytes_to_send: &[u8]) -> impl Future<Output = ()>;
    fn send_then_read(&mut self, bytes_to_send: &[u8], read_into: &mut[u8]) -> impl Future<Output = ()>;
}
