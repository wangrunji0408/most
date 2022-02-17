#![feature(bigint_helper_methods)]
#![feature(portable_simd)]

mod u128x8;
mod u192;
mod u192x8;

pub use self::u128x8::U128x8;
pub use self::u192::U192;
pub use self::u192x8::U192x8;
