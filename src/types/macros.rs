#[macro_export]
macro_rules! impl_storable_for {
    ($type:ty) => {
        use ic_stable_structures::{storable::Bound, Storable};
        impl Storable for $type {
            const BOUND: Bound = Bound::Unbounded;

            fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
                use candid::Encode;
                use std::borrow::Cow;
                Cow::Owned(Encode!(&self).expect(concat!("Failed to encode ", stringify!($type))))
            }

            fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
                use candid::Decode;
                Decode!(bytes.as_ref(), Self)
                    .expect(concat!("Failed to decode ", stringify!($type)))
            }
        }
    };
}
