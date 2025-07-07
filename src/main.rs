


fn main() {}



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

        define!($($tail)*);
    };

    // pub struct Something(...);
    (
        $( #[$meta:meta] )*
        $vis:vis struct $name:ident (
            $(
                $( #[$member_meta:meta] )*
                $member_vis:vis $member_ty:ident
            ),*
        $(,)? );

        $($tail:tt)*
    ) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        $( #[$meta] )*
        $vis struct $name;

        impl $name {
            $(
                $( #[$member_meta] )*
                $member_vis fn $member_name(&self) -> $member_ty {
                    todo!()
                }
            )*
        }

        define!($($tail)*);
    };

    // pub struct Something { pub member_a: u8, };
    (
        $( #[$meta:meta] )*
        $vis:vis struct $name:ident {
            $(
                $( #[$member_meta:meta] )*
                $member_vis:vis $member_name:ident : $member_ty:ident
            ),*
        $(,)? };
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
                    $member_ty
                }
            )*
        }

        define!($($tail)*);
    };
}

define!{
    pub struct Player;
    pub struct Game {
        pub player: Player,
    };
}
