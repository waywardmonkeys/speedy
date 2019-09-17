use std::io;
use std::mem;
use std::borrow::{Cow, ToOwned};
use std::ops::Range;

use crate::readable::Readable;
use crate::reader::Reader;

use crate::context::Context;
use crate::utils::as_bytes_mut;
use crate::endianness::Endianness;

#[inline]
pub fn read_vec< 'a, C, R, T >( reader: &mut R, length: usize ) -> io::Result< Vec< T > >
    where C: Context,
          R: Reader< 'a, C >,
          T: Readable< 'a, C >
{
    let mut vec = Vec::with_capacity( length );
    if T::speedy_is_primitive() {
        unsafe {
            vec.set_len( length );
            reader.read_bytes( T::speedy_slice_as_bytes_mut( &mut vec ) )?;
        }
        T::speedy_convert_slice_endianness( reader.endianness(), &mut vec );
    } else {
        for _ in 0..length {
            vec.push( reader.read_value()? );
        }
    }

    Ok( vec )
}

impl< 'a, C: Context > Readable< 'a, C > for bool {
    #[inline]
    fn read_from< R: Reader< 'a, C > >( reader: &mut R ) -> io::Result< Self > {
        let value = reader.read_u8()?;
        if value == 0 {
            Ok( false )
        } else {
            Ok( true )
        }
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        1
    }
}

macro_rules! impl_for_primitive {
    ($type:ty, $getter:ident, $endianness_swap:ident) => {
        impl< 'a, C: Context > Readable< 'a, C > for $type {
            #[inline(always)]
            fn read_from< R: Reader< 'a, C > >( reader: &mut R ) -> io::Result< Self > {
                reader.$getter()
            }

            #[inline]
            fn minimum_bytes_needed() -> usize {
                mem::size_of::< Self >()
            }

            #[doc(hidden)]
            #[inline(always)]
            fn speedy_is_primitive() -> bool {
                true
            }

            #[doc(hidden)]
            #[inline(always)]
            unsafe fn speedy_slice_as_bytes_mut( slice: &mut [Self] ) -> &mut [u8] {
                as_bytes_mut( slice )
            }

            #[doc(hidden)]
            #[inline(always)]
            fn speedy_convert_slice_endianness( endianness: Endianness, slice: &mut [$type] ) {
                endianness.$endianness_swap( slice );
            }
        }
    }
}

impl_for_primitive!( i8, read_i8, swap_slice_i8 );
impl_for_primitive!( i16, read_i16, swap_slice_i16 );
impl_for_primitive!( i32, read_i32, swap_slice_i32 );
impl_for_primitive!( i64, read_i64, swap_slice_i64 );
impl_for_primitive!( u8, read_u8, swap_slice_u8 );
impl_for_primitive!( u16, read_u16, swap_slice_u16 );
impl_for_primitive!( u32, read_u32, swap_slice_u32 );
impl_for_primitive!( u64, read_u64, swap_slice_u64 );
impl_for_primitive!( f32, read_f32, swap_slice_f32 );
impl_for_primitive!( f64, read_f64, swap_slice_f64 );

impl< 'a, C: Context > Readable< 'a, C > for String {
    #[inline]
    fn read_from< R: Reader< 'a, C > >( reader: &mut R ) -> io::Result< Self > {
        let bytes: Vec< u8 > = reader.read_value()?;
        match String::from_utf8( bytes ) {
            Err( error ) => Err( io::Error::new( io::ErrorKind::InvalidData, error ) ),
            Ok( string ) => Ok( string )
        }
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        <Vec< u8 > as Readable< 'a, C >>::minimum_bytes_needed()
    }
}

impl< 'a, C: Context > Readable< 'a, C > for Cow< 'a, str > {
    #[inline]
    fn read_from< R: Reader< 'a, C > >( reader: &mut R ) -> io::Result< Self > {
        let bytes: String = reader.read_value()?;
        Ok( bytes.into() )
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        <String as Readable< 'a, C >>::minimum_bytes_needed()
    }
}

impl< 'a, C: Context, T: Readable< 'a, C > > Readable< 'a, C > for Vec< T > {
    #[inline]
    fn read_from< R: Reader< 'a, C > >( reader: &mut R ) -> io::Result< Self > {
        let length = reader.read_u32()? as usize;
        read_vec( reader, length )
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        4
    }
}

impl< 'a, C: Context, T: Readable< 'a, C > > Readable< 'a, C > for Cow< 'a, [T] > where [T]: ToOwned< Owned = Vec< T > > {
    #[inline]
    fn read_from< R: Reader< 'a, C > >( reader: &mut R ) -> io::Result< Self > {
        let bytes: Vec< T > = reader.read_value()?;
        Ok( Cow::Owned( bytes ) )
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        <Vec< T > as Readable< 'a, C >>::minimum_bytes_needed()
    }
}

impl< 'a, C: Context, T: Readable< 'a, C > > Readable< 'a, C > for Range< T > {
    #[inline]
    fn read_from< R: Reader< 'a, C > >( reader: &mut R ) -> io::Result< Self > {
        let start = reader.read_value()?;
        let end = reader.read_value()?;
        Ok( start..end )
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        <T as Readable< 'a, C >>::minimum_bytes_needed() * 2
    }
}

impl< 'a, C: Context, T: Readable< 'a, C > > Readable< 'a, C > for Option< T > {
    #[inline]
    fn read_from< R: Reader< 'a, C > >( reader: &mut R ) -> io::Result< Self > {
        let flag = reader.read_value()?;
        let value = if flag {
            Some( reader.read_value()? )
        } else {
            None
        };

        Ok( value )
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        1
    }
}

impl< 'a, C: Context > Readable< 'a, C > for () {
    #[inline]
    fn read_from< R: Reader< 'a, C > >( _: &mut R ) -> io::Result< Self > {
        Ok(())
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        0
    }
}

macro_rules! impl_for_tuple {
    ($($name:ident),+) => {
        impl< 'a, C: Context, $($name: Readable< 'a, C >),+ > Readable< 'a, C > for ($($name,)+) {
            #[inline]
            fn read_from< R: Reader< 'a, C > >( reader: &mut R ) -> io::Result< Self > {
                $(
                    #[allow(non_snake_case)]
                    let $name = reader.read_value()?;
                )+

                Ok( ($($name,)+) )
            }

            #[inline]
            fn minimum_bytes_needed() -> usize {
                let mut size = 0;
                $(
                    size += <$name as Readable< 'a, C >>::minimum_bytes_needed();
                )+
                size
            }
        }
    }
}

impl_for_tuple!( A0 );
impl_for_tuple!( A0, A1 );
impl_for_tuple!( A0, A1, A2 );
impl_for_tuple!( A0, A1, A2, A3 );
impl_for_tuple!( A0, A1, A2, A3, A4 );
impl_for_tuple!( A0, A1, A2, A3, A4, A5 );
impl_for_tuple!( A0, A1, A2, A3, A4, A5, A6 );
impl_for_tuple!( A0, A1, A2, A3, A4, A5, A6, A7 );
impl_for_tuple!( A0, A1, A2, A3, A4, A5, A6, A7, A8 );
impl_for_tuple!( A0, A1, A2, A3, A4, A5, A6, A7, A8, A9 );
impl_for_tuple!( A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10 );

impl< 'a, C: Context > Readable< 'a, C > for Endianness {
    #[inline]
    fn read_from< R: Reader< 'a, C > >( reader: &mut R ) -> io::Result< Self > {
        let value = reader.read_u8()?;
        match value {
            0 => Ok( Endianness::LittleEndian ),
            1 => Ok( Endianness::BigEndian ),
            _ => Err( io::Error::new( io::ErrorKind::InvalidData, "invalid enum variant" ) )
        }
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        1
    }
}
