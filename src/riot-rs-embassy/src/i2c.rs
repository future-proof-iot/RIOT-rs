macro_rules! impl_async_i2c_for_driver_enum {
    ($driver_enum:ident, $( $peripheral:ident ),*) => {
        impl embedded_hal_async::i2c::I2c for $driver_enum {
            async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(i2c) => i2c.twim.read(address, read).await, )*
                }
            }

            async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(i2c) => i2c.twim.write(address, write).await, )*
                }
            }

            async fn write_read(
                &mut self,
                address: u8,
                write: &[u8],
                read: &mut [u8],
            ) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(i2c) => i2c.twim.write_read(address, write, read).await, )*
                }
            }

            async fn transaction(
                &mut self,
                address: u8,
                operations: &mut [Operation<'_>],
            ) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(i2c) => i2c.twim.transaction(address, operations).await, )*
                }
            }
        }
    }
}
pub(crate) use impl_async_i2c_for_driver_enum;
