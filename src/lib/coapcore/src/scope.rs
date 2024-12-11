//! Expressions for access policy as evaluated for a particular security context.

use coap_message::{MessageOption, ReadableMessage};

/// A data item representing the server access policy as evaluated for a particular security context.
pub trait Scope: Sized + core::fmt::Debug + defmt::Format {
    /// Returns true if a request may be performed by the bound security context.
    fn request_is_allowed<M: ReadableMessage>(&self, request: &M) -> bool;

    /// Returns true if a bound security context should be preferably retained when hitting
    /// resource limits.
    fn is_admin(&self) -> bool {
        false
    }

    fn nosec_authorization() -> Self;

    fn unauthenticated_edhoc_user_authorization() -> Self {
        Self::nosec_authorization()
    }

    fn the_one_known_authorization() -> Self {
        Self::nosec_authorization()
    }
}

#[derive(Debug, defmt::Format)]
pub struct AllowAll;

impl Scope for AllowAll {
    fn request_is_allowed<M: ReadableMessage>(&self, _request: &M) -> bool {
        true
    }

    fn nosec_authorization() -> Self {
        Self
    }
}

#[derive(Debug, defmt::Format)]
pub struct DenyAll;

impl Scope for DenyAll {
    fn request_is_allowed<M: ReadableMessage>(&self, _request: &M) -> bool {
        false
    }

    fn nosec_authorization() -> Self {
        Self
    }
}

const AIF_SCOPE_MAX_LEN: usize = 64;

/// A representation of an RFC9237 using the REST-specific model.
///
/// It is aribtrarily limited in length; future versions may give more flexibility, eg. by
/// referring to data in storage.
///
/// This type is constrained to valid CBOR representations of the REST-specific model; it may panic
/// if that constraint is not upheld.
///
/// ## Caveats
///
/// Using this is not very efficient; worst case, it iterates over all options for all AIF entries.
///
/// This completely disregards proper URI splitting; this works for very simple URI references in
/// the AIF.
#[derive(Debug, defmt::Format)]
pub struct AifValue([u8; AIF_SCOPE_MAX_LEN]);

impl Scope for AifValue {
    fn request_is_allowed<M: ReadableMessage>(&self, request: &M) -> bool {
        let code: u8 = request.code().into();
        let (codebit, false) = 1u32.overflowing_shl(
            u32::from(code)
                .checked_sub(1)
                .expect("Request codes are != 0"),
        ) else {
            return false;
        };
        let mut decoder = minicbor::Decoder::new(&self.0);
        'outer: for item in decoder.array_iter::<(&str, u32)>().unwrap() {
            let (path, perms) = item.unwrap();
            if perms & codebit == 0 {
                continue;
            }
            // BIG FIXME: We're iterating over options without checking for critical options. If the
            // resource handler router consumes any different set of options, that disagreement might
            // give us a security issue.
            let mut pathopts = request
                .options()
                .filter(|o| o.number() == coap_numbers::option::URI_PATH)
                .peekable();
            if path == "/" && pathopts.peek().is_none() {
                // Special case: For consistency should be a single empty option.
                return true;
            }
            if !path.starts_with("/") {
                panic!("Invalid AIF");
            }
            let mut remainder = &path[1..];
            while remainder != "" {
                let (next_part, next_remainder) = match remainder.split_once('/') {
                    Some((next_part, next_remainder)) => (next_part, next_remainder),
                    None => (remainder, ""),
                };
                let Some(this_opt) = pathopts.next() else {
                    // Request path is shorter than this AIF record
                    continue 'outer;
                };
                if this_opt.value() != next_part.as_bytes() {
                    // Request path is just different from this AIF record
                    continue 'outer;
                }
                remainder = next_remainder;
            }
            if pathopts.next().is_none() {
                // Request path is longer than this AIF record
                return true;
            }
        }
        // No matches found
        false
    }

    // FIXME: So far, this emulates the old AifStaticRest; these functions will need to go somewhere else.
    fn nosec_authorization() -> Self {
        let mut value = [0; AIF_SCOPE_MAX_LEN];
        use cbor_macro::cbor;
        let allowed = cbor!([["/.well-known/core", 1], ["/poem", 1]]);
        value[..allowed.len()].copy_from_slice(&allowed);
        Self(value)
    }

    fn the_one_known_authorization() -> Self {
        let mut value = [0; AIF_SCOPE_MAX_LEN];
        use cbor_macro::cbor;
        let allowed =
            cbor!([["/stdout", 17 / GET and FETCH /], ["/.well-known/core", 1], ["/poem", 1]]);
        value[..allowed.len()].copy_from_slice(&allowed);
        Self(value)
    }

    fn is_admin(&self) -> bool {
        self.0[0] >= 0x83
    }
}
