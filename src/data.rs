use std::io::{Read, Seek, Write};

pub trait DataPoint {
    fn get_size(&self) -> u64;
    fn get_hash(&self) -> u64;
    fn write_out<W: Write + Seek>(&self, writer: &mut W) -> std::io::Result<()>;
    fn read_in<R: Read + Seek>(&mut self, reader: &mut R) -> std::io::Result<()>;
}

#[macro_export]
macro_rules! datapoint {
    ($($vis:vis struct $name:ident $block:tt)*) => {
        use std::io::{Read, Write, Seek};
        $(
            $crate::_internal_struct_impl!($vis, $name, $block);
        )*
    };
}

#[macro_export]
macro_rules! _internal_struct_impl {
    ($vis:vis, $name:ident, $block:tt) => {
        #[derive(Copy, Clone, Debug, Default, PartialEq)]
        $vis struct $name $block
        impl DataPoint for $name {
            $crate::_internal_impl_get_size!($block);
            $crate::_internal_impl_get_hash!($block);
            $crate::_internal_impl_write_out!($block);
            $crate::_internal_impl_read_in!($block);
        }
    };
}

#[macro_export]
macro_rules! _internal_impl_get_size {
    ({$($field:ident : $type:ty,)*}) => {
        fn get_size(&self) -> u64 {
            0
            $(
               + self.$field.get_size()
            )*
        }
    };
}

#[macro_export]
macro_rules! _internal_impl_get_hash {
    ({$($field:ident : $type:ty,)*}) => {
        fn get_hash(&self) -> u64 {
            1_u64
            $(
                .wrapping_mul(self.$field.get_hash())
            )*
        }
    };
}

#[macro_export]
macro_rules! _internal_impl_write_out {
    ({$($field:ident : $type:ty,)*}) => {
        fn write_out<W: Write + Seek>(&self, writer: &mut W) -> std::io::Result<()> {
            $(
                self.$field.write_out(writer)?;
            )*
            Ok(())
        }
    };
}

#[macro_export]
macro_rules! _internal_impl_read_in {
    ({$($field:ident : $type:ty,)*}) => {
        fn read_in<R: Read + Seek>(&mut self, reader: &mut R) -> std::io::Result<()> {
            $(
                self.$field.read_in(reader)?;
            )*
            Ok(())
        }
    };
}

macro_rules! _internal_datapoint_impl {
    ($impl_type:ty, $seed:literal) => {
        impl DataPoint for $impl_type {
            fn get_size(&self) -> u64 {
                std::mem::size_of::<Self>() as u64
            }

            fn get_hash(&self) -> u64 {
                $seed * 0x100000001b3
            }

            fn write_out<W: Write + Seek>(&self, writer: &mut W) -> std::io::Result<()> {
                writer.write_all(&self.to_le_bytes())
            }

            fn read_in<R: Read + Seek>(&mut self, reader: &mut R) -> std::io::Result<()> {
                let mut buf = [0u8; std::mem::size_of::<Self>()];
                reader.read_exact(&mut buf)?;
                *self = Self::from_le_bytes(buf);
                Ok(())
            }
        }

        _internal_array_impl!($impl_type);
    };
}

macro_rules! _internal_array_impl {
    ($impl_type:ty) => {
        impl<const N: usize> DataPoint for [$impl_type; N] {
            fn get_size(&self) -> u64 {
                std::mem::size_of::<Self>() as u64
            }

            fn get_hash(&self) -> u64 {
                self.iter().fold(1_u64, |a, i| a.wrapping_mul(i.get_hash()))
            }

            fn write_out<W: Write + Seek>(&self, writer: &mut W) -> std::io::Result<()> {
                for i in self.iter() {
                    i.write_out(writer)?;
                }
                Ok(())
            }

            fn read_in<R: Read + Seek>(&mut self, reader: &mut R) -> std::io::Result<()> {
                for i in self.iter_mut() {
                    i.read_in(reader)?;
                }
                Ok(())
            }
        }
    };
}

_internal_datapoint_impl!(i8, 1087);
_internal_datapoint_impl!(u8, 3119);
_internal_datapoint_impl!(i16, 4909);
_internal_datapoint_impl!(u16, 6113);
_internal_datapoint_impl!(i32, 8191);
_internal_datapoint_impl!(u32, 18181);
_internal_datapoint_impl!(i64, 21169);
_internal_datapoint_impl!(u64, 37199);
_internal_datapoint_impl!(i128, 60493);
_internal_datapoint_impl!(u128, 93911);
_internal_datapoint_impl!(f32, 131071);
_internal_datapoint_impl!(f64, 524287);
