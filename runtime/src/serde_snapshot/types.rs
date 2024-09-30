use {
    serde::{de, Deserialize, Deserializer},
    std::{fmt, marker::PhantomData},
};

/// Snapshot serde-safe AccountsLtHash
#[cfg_attr(feature = "frozen-abi", derive(AbiExample))]
#[derive(Debug)]
pub struct SerdeAccountsLtHash(
    // serde only has array support up to 32 elements; anything larger needs to be handled manually
    // see https://github.com/serde-rs/serde/issues/1937 for more information
    #[allow(dead_code)] pub [u16; 1024],
);

impl<'de> Deserialize<'de> for SerdeAccountsLtHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrayVisitor {
            element: PhantomData<u16>,
        }
        impl<'de> de::Visitor<'de> for ArrayVisitor {
            type Value = [u16; 1024];
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(concat!("a u16 array of length 1024"))
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut arr = [0u16; 1024];
                for (i, elem) in arr.iter_mut().enumerate() {
                    *elem = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(i, &self))?;
                }
                Ok(arr)
            }
        }

        let visitor = ArrayVisitor {
            element: PhantomData,
        };
        let array = deserializer.deserialize_tuple(1024, visitor)?;
        Ok(Self(array))
    }
}
