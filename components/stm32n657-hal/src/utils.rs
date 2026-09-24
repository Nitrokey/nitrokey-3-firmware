#[derive(Debug, Clone, Copy)]
pub struct FromUError;

macro_rules! enum_u {
    (
        #[repr($repr:ty)]
        $(#[$outer:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$var_outer:meta])*
                $var:ident = $num:expr
            ),+
            $(,)*
        }
    ) => {
        #[repr($repr)]
        $(#[$outer])*
        $vis enum $name {
            $(
                $(#[$var_outer])*
                $var = $num,
            )*
        }

        impl TryFrom<$repr> for $name {
            type Error = crate::utils::FromUError;
            fn try_from(val: $repr) -> Result<Self, Self::Error> {
                match val {
                    $(
                        $num => Ok($name::$var),
                    )*
                    _ => Err(crate::utils::FromUError)
                }
            }
        }

        impl PartialEq<$repr> for $name {
            fn eq(&self, other: &$repr) -> bool {
                *self as $repr == *other
            }
        }

        impl<T: Into<$name> + Copy> PartialEq<T> for $name {
            fn eq(&self, other: &T) -> bool {
                let other: $name = (*other).into();
                matches!((self,other), $(
                    | ($name::$var, $name::$var)
                )*)
            }
        }

        impl Eq for $name {}
    }
}

pub(crate) use enum_u;
