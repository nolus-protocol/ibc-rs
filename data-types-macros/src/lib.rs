#![no_std]

pub use ibc_core_host_types::error::DecodingError;

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

        impl $crate::Newtype for $type {
            type Inner = $inner_type;
        }

        impl From<$type> for abci::EventAttribute {
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

        impl TryFrom<abci::EventAttribute> for $type {
            type Error = <Self as ::core::str::FromStr>::Err;

            fn try_from(value: abci::EventAttribute) -> Result<Self, Self::Error> {
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
                    .map_err(|e| Self::Error::invalid_raw_data(format!("attribute value: {e}")))?
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
                $attribute: $attribute_type,
            )+
        }

        impl $type {
            pub const EVENT_KIND: &str = $event_kind;

            pub const fn new($($attribute: <$attribute_type as $crate::Newtype>::Inner,)+) -> Self {
                Self {
                    $($attribute: <$attribute_type>::new($attribute),)+
                }
            }

            $(
                pub const fn $attribute(&self) -> &<$attribute_type as $crate::Newtype>::Inner {
                    &self.$attribute.0
                }
            )+
        }

        impl From<$type> for abci::Event {
            fn from(event: $type) -> Self {
                Self {
                    kind: <$type>::EVENT_KIND.into(),
                    attributes: vec![
                        $(event.$attribute.into(),)+
                    ],
                }
            }
        }

        impl<K, A, AK, AV> TryFrom<(K, A)> for $type
        where
            K: Into<String>,
            A: IntoIterator<Item = (AK, AV)>,
            AK: AsRef<str>,
            AV: AsRef<str>,
        {
            type Error = $crate::DecodingError;

            fn try_from((kind, attributes): (K, A)) -> Result<Self, Self::Error> {
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
                            $attribute = Some(value.as_ref().parse()?);
                        })+
                        _ => {}
                    }
                }

                $(let $attribute = $attribute.ok_or(
                    Self::Error::missing_raw_data(format!("Attribute {} is not set!", <$attribute_type>::FRIENDLY_NAME))
                )?;)+

                Ok(Self {
                    $($attribute,)+
                })
            }
        }

        impl TryFrom<abci::Event> for $type {
            type Error = $crate::DecodingError;

            fn try_from(event: abci::Event) -> Result<Self, Self::Error> {
                (
                    event.kind,
                    event.attributes.iter().map(|attribute| {
                        Ok((
                            attribute
                                .key_str()
                                .map_err(|e| Self::Error::invalid_raw_data(format!("attribute key: {e}")))?,
                            attribute
                                .value_str()
                                .map_err(|e| Self::Error::invalid_raw_data(format!("attribute value: {e}")))?,
                        ))
                    }).collect::<Result<Vec<_>, Self::Error>>()?
                ).try_into()
            }
        }
    };
}
