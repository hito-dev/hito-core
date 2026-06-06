// expose submodules if you want (optional)
pub mod log_backend;

macro_rules! mods { ($($name:ident),*) => { $(mod $name;)* }; }
mods!(time, serial, bluetooth, touch, display);

#[cfg(feature = "simulator")]
mod simulator;
#[cfg(feature = "zephyr")]
mod zephyr;

mod payload_buffer;

// 🔥 re-export all drivers at drivers root
#[allow(unused_imports)]
pub use time::Time;
#[allow(unused_imports)]
pub use serial::UsbSerial;
#[allow(unused_imports)]
pub use bluetooth::Bluetooth;
#[allow(unused_imports)]
pub use touch::Touch;
#[allow(unused_imports)]
pub use display::Display;

// Single macro that takes methods once
// Put this somewhere common (e.g. src/drivers.rs), and ensure it's in scope.
#[macro_export]
macro_rules! driver {
    (
        $(#[$trait_meta:meta])*
        pub trait $trait_name:ident => $impl_ty:ident {
            $(
                $(#[$method_meta:meta])*
                fn $method:ident ( $($arg:ident : $arg_ty:ty),* $(,)? ) $(-> $ret:ty)? ;
            )*
        }
    ) => {
        // 1) Generate the trait exactly as written
        $(#[$trait_meta])*
        pub trait $trait_name {
            $(
                $(#[$method_meta])*
                fn $method($($arg: $arg_ty),*) $(-> $ret)?;
            )*
        }

        // 2) Generate the inherent forwarding impl block
        impl $impl_ty {
            $(
                driver!(@forward
                    $trait_name,
                    $method,
                    ( $($arg : $arg_ty),* )
                    $(-> $ret)?
                );
            )*
        }
    };

    // Method WITH return type
    (@forward
        $trait_name:ident,
        $method:ident,
        ( $($arg:ident : $arg_ty:ty),* )
        -> $ret:ty
    ) => {
        #[inline(always)]
        pub fn $method($($arg: $arg_ty),*) -> $ret {
            <Self as $trait_name>::$method($($arg),*)
        }
    };

    // Method WITHOUT return type (implicitly returns ())
    (@forward
        $trait_name:ident,
        $method:ident,
        ( $($arg:ident : $arg_ty:ty),* )
    ) => {
        #[inline(always)]
        pub fn $method($($arg: $arg_ty),*) {
            <Self as $trait_name>::$method($($arg),*);
        }
    };
}


