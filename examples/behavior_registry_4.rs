


fn main() {}



pub unsafe trait Object {
    fn raw() -> Self;
}

#[macro_export]
macro_rules! define {
    () => {};

    // pub struct Something;
    (
        $( #[$meta:meta] )*             // #[derive(...)]
        $vis:vis struct $name:ident;    // pub struct Something;
        $($tail:tt)*                    // ...
    ) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        $( #[$meta] )*
        $vis struct $name;

        unsafe impl Object for $name {
            fn raw() -> Self {
                $name
            }
        }

        define!($($tail)*);
    };

    // pub struct Something(pub u8);
    (
        $( #[$meta:meta] )*
        $vis:vis struct $name:ident (
            $(
                $( #[$member_meta:meta] )*
                $member_vis:vis $member_ty:ty
            ),*
        $(,)? );

        $($tail:tt)*
    ) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        $( #[$meta] )*
        $vis struct $name;

        unsafe impl Object for $name {
            fn raw() -> Self {
                $name
            }
        }

        define!($($tail)*);
    };


    // pub struct Something { pub member_a: u8, };
    (
        $( #[$meta:meta] )*
        $vis:vis struct $name:ident {
            $(
                $( #[$member_meta:meta] )*
                $member_vis:vis $member_name:ident : $member_ty:ty = $member_init:expr
            ),*
            $(,)?
        };
        $($tail:tt)*
    ) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        $( #[$meta] )*
        $vis struct $name;

        impl $name {
            $(
                $( #[$member_meta] )*
                #[inline]
                $member_vis fn $member_name(&self) -> $member_ty {
                    <$member_ty>::raw()
                }
            )*
        }

        define!($($tail)*);
    };
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_works() {
        define!{
            pub struct Player(pub u8);
            pub struct Game {
                player: Player = Player::new(1),
            };
        }

        impl Player {
            pub fn something(&self) {
                println!("tests::macro_works::something");
            }
        }

        Game.player().something();
    }
}
