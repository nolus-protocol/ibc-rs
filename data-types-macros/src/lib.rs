#![no_std]

pub use ibc_core_host_types::error::DecodingError;
pub use tendermint::abci::{Event, EventAttribute};

pub trait Newtype {
    type Inner;
}

#[macro_export]
macro_rules! define_attribute {
    ($key:expr => $type:ident ( $inner_type:ty ) {
        friendly_name = $friendly_name:expr,
        into = $into:expr,
        parse = $parse:expr $(,)?
    }) => {
        #[cfg_attr(
            feature = "parity-scale-codec",
            derive(
                parity_scale_codec::Encode,
                parity_scale_codec::Decode,
                scale_info::TypeInfo
            )
        )]
        #[cfg_attr(
            feature = "borsh",
            derive(borsh::BorshSerialize, borsh::BorshDeserialize)
        )]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $type($inner_type);

        impl $type {
            pub const ATTRIBUTE_KEY: &str = $key;

            pub(crate) const FRIENDLY_NAME: &str = $friendly_name;

            pub const fn new(value: $inner_type) -> Self {
                Self(value)
            }
        }

        impl From<$inner_type> for $type {
            fn from(inner: $inner_type) -> Self {
                Self(inner)
            }
        }

        impl From<$type> for $inner_type {
            fn from($type(inner): $type) -> Self {
                inner
            }
        }

        impl $crate::Newtype for $type {
            type Inner = $inner_type;
        }

        impl From<$type> for $crate::EventAttribute {
            fn from($type(inner): $type) -> Self {
                ($key, $into(inner)).into()
            }
        }

        impl ::core::str::FromStr for $type {
            type Err = $crate::DecodingError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $parse(s).map(Self)
            }
        }

        impl TryFrom<$crate::EventAttribute> for $type {
            type Error = <Self as ::core::str::FromStr>::Err;

            fn try_from(value: $crate::EventAttribute) -> Result<Self, Self::Error> {
                let key_str = value
                    .key_str()
                    .map_err(|_| Self::Error::missing_raw_data("attribute key"))?;

                if key_str != Self::ATTRIBUTE_KEY {
                    return Err(Self::Error::MismatchedResourceName {
                        expected: Self::ATTRIBUTE_KEY.to_string(),
                        actual: key_str.to_string(),
                    });
                }

                value
                    .value_str()
                    .map_err(|e| {
                        Self::Error::invalid_raw_data(format_args!("attribute value: {e}"))
                    })?
                    .parse()
            }
        }
    };
}

#[macro_export]
macro_rules! define_event {
    ($event_kind:expr => $type:ident {
        $(
            $(#[$($attribute_meta:meta),+ $(,)?])?
            $attribute:ident : $attribute_type:ty
        ),+ $(,)?
    }) => {
        #[cfg_attr(
            feature = "parity-scale-codec",
            derive(
                parity_scale_codec::Encode,
                parity_scale_codec::Decode,
                scale_info::TypeInfo
            )
        )]
        #[cfg_attr(
            feature = "borsh",
            derive(borsh::BorshSerialize, borsh::BorshDeserialize)
        )]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $type {
            $(
                $(#[$($attribute_meta),+])?
                pub $attribute: $attribute_type,
            )+
        }

        impl $type {
            pub const EVENT_KIND: &str = $event_kind;

            $(
                $(#[$($attribute_meta),+])?
                pub const fn $attribute(&self) -> &<$attribute_type as $crate::Newtype>::Inner {
                    &self.$attribute.0
                }
            )+
        }

        impl From<$type> for $crate::Event {
            fn from(event: $type) -> Self {
                Self {
                    kind: <$type>::EVENT_KIND.into(),
                    attributes: vec![
                        $(event.$attribute.into(),)+
                    ],
                }
            }
        }

        impl<Kind, Attrs, AttrKey, AttrValue> TryFrom<(Kind, Attrs)> for $type
        where
            Kind: Into<String>,
            Attrs: IntoIterator<Item = (AttrKey, AttrValue)>,
            AttrKey: AsRef<str>,
            AttrValue: AsRef<str>,
        {
            type Error = $crate::DecodingError;

            fn try_from((kind, attributes): (Kind, Attrs)) -> Result<Self, Self::Error> {
                {
                    let kind = kind.into();

                    if kind != Self::EVENT_KIND {
                        return Err(Self::Error::MismatchedResourceName {
                            expected: Self::EVENT_KIND.into(),
                            actual: kind,
                        });
                    };
                }

                $(let mut $attribute = None;)+

                for (key, value) in attributes {
                    let key = key.as_ref();

                    match key {
                        $(<$attribute_type>::ATTRIBUTE_KEY => {
                            if $attribute.is_none() {
                                $attribute = Some(value.as_ref().parse()?);
                            } else {
                                return Err(Self::Error::invalid_raw_data(format_args!(
                                    "Duplicate attribute value found for attribute {}!",
                                    <$attribute_type>::FRIENDLY_NAME,
                                )));
                            }
                        })+
                        _ => {}
                    }
                }

                $(let $attribute = $attribute.ok_or(
                    Self::Error::missing_raw_data(format_args!("Attribute {} is not set!", <$attribute_type>::FRIENDLY_NAME))
                )?;)+

                Ok(Self {
                    $($attribute,)+
                })
            }
        }

        impl TryFrom<$crate::Event> for $type {
            type Error = $crate::DecodingError;

            fn try_from(event: $crate::Event) -> Result<Self, Self::Error> {
                (
                    event.kind,
                    event.attributes
                        .iter()
                        .map(|attribute| {
                            Ok((
                                attribute
                                    .key_str()
                                    .map_err(|e| Self::Error::invalid_raw_data(format_args!("attribute key: {e}")))?,
                                attribute
                                    .value_str()
                                    .map_err(|e| Self::Error::invalid_raw_data(format_args!("attribute value: {e}")))?,
                            ))
                        })
                        .collect::<Result<Vec<_>, Self::Error>>()?,
                ).try_into()
            }
        }
    };
}
