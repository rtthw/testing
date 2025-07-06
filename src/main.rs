


fn main() {}



macro_rules! define {
    () => {};

    // pub Something;
    (
        $( #[$meta:meta] )*     // #[derive(...)]
        $vis:vis $name:ident;   // pub Something;
        $($tail:tt)*            // ...
    ) => {
        $( #[$meta] )*
        $vis struct $name;
        define!($($tail)*);
    };

    // pub Something(...);
    (
        $( #[$meta:meta] )*
        $vis:vis $name:ident (
            $(
                $( #[$field_meta:meta] )*
                $field_vis:vis $field_ty:ty
            ),*
        $(,)? );

        $($tail:tt)*
    ) => {
        $( #[$meta] )*
        $vis struct $name (
            $(
                $( #[$field_meta] )*
                $field_vis $field_ty
            ),*
        );
        define!($($tail)*);
    };

    // pub Something { field_a: u8, }
    (
        $( #[$meta:meta] )*
        $vis:vis $name:ident {
            $(
                $( #[$field_meta:meta] )*
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),*
        $(,)? };
        $($tail:tt)*
    ) => {
        $( #[$meta] )*
        $vis struct $name {
            $(
                $( #[$field_meta] )*
                $field_vis $field_name : $field_ty
            ),*
        }
        define!($($tail)*);
    };
}

define!{
    Player;
    pub Game {
        player: Player,
    };
}
