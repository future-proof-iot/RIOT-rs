//! Types sharedly used by different modules of this crate

/// An own identifier for a security context.
///
/// This is used with EDHOC as C_I when in an initiator role, C_R when in a responder role, and
/// recipient ID in OSCORE.
///
/// This type represents any of the 48 efficient identifiers that use CBOR one-byte integer
/// encodings (see RFC9528 Section 3.3.2), or equivalently the 1-byte long OSCORE identifiers
///
/// Lakers supports a much larger value space for C_x, and coapcore processes larger values
/// selected by the peer -- but on its own, will select only those that fit in this type.
// FIXME Could even limit to positive values if MAX_CONTEXTS < 24
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) struct COwn(u8);

impl COwn {
    /// Maximum length of [`Self::as_slice`].
    ///
    /// This is exposed to allow sizing stack allocated buffers.
    pub(crate) const MAX_SLICE_LEN: usize = 1;

    /// Number of values that [`.not_in_iter()`][Self::not_in_iter] can generate.
    ///
    /// This is exposed to allow for static checks that a security context pool does not exceed
    /// this number.
    pub(crate) const GENERATABLE_VALUES: usize = 48;

    /// Find a value of self that is not found in the iterator.
    ///
    /// This asserts that the iterator is (known to be) short enough that this will always succeed.
    pub fn not_in_iter(iterator: impl Iterator<Item = Self>) -> Self {
        // In theory, this would allow the compiler to see that the unreachable below is indeed
        // unreachable
        assert!(
            iterator
                .size_hint()
                .1
                .is_some_and(|v| v < Self::GENERATABLE_VALUES),
            "Too many slots to reliably assign connection identifier"
        );
        let mut seen_pos = 0u32;
        let mut seen_neg = 0u32;
        for i in iterator {
            let major = i.0 >> 5;
            // Let's not make unsafe assumptions on the own value range
            let target = if major == 0 {
                &mut seen_pos
            } else {
                &mut seen_neg
            };
            // Convenienlty, masking to the minor part puts us in the very range that allows u32
            // shifting
            *target |= 1 << (i.0 & 0x1f);
        }
        // trailing_ones = n implies that bit 1<<n is a zero and thus COwn(n) is free
        let pos_to = seen_pos.trailing_ones();
        if pos_to < 24 {
            return Self(pos_to as u8);
        }
        let neg_to = seen_neg.trailing_ones();
        if neg_to < 24 {
            return Self(0x20 | neg_to as u8);
        }
        unreachable!("Iterator is not long enough to set this many bits.");
    }

    /// Given an OSCORE Key ID (kid), find the corresponding context identifier value
    pub fn from_kid(kid: &[u8]) -> Option<Self> {
        match kid {
            [first] if *first <= 0x17 || (*first >= 0x20 && *first <= 0x37) => Some(Self(*first)),
            _ => None,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        core::slice::from_ref(&self.0)
    }
}

impl From<COwn> for lakers::ConnId {
    fn from(cown: COwn) -> Self {
        lakers::ConnId::from_slice(cown.as_slice())
            .expect("ConnId is always big enough for at least COwn")
    }
}
