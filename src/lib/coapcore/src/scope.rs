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
}

impl Scope for core::convert::Infallible {
    fn request_is_allowed<M: ReadableMessage>(&self, request: &M) -> bool {
        match *self {}
    }
}

/// A parser for [`Scope`] from a serialized form.
///
/// This is expressed as a type with possibly non-ZST self to allow carrying over data from the
/// configuration of the Authorization Server (AS) that issued the token into its processed form. For
/// example, while many applications can use the zero-sized [`ParsingAif`] implementation, others
/// may build on that and limit the resulting in a multi-AS scenario (when not all ASes may issue
/// all scopes).
pub trait ScopeGenerator: Sized {
    type Scope: Scope;

    fn from_token_scope(self, bytes: &[u8]) -> Result<Self::Scope, InvalidScope>;
}

impl ScopeGenerator for core::convert::Infallible {
    type Scope = core::convert::Infallible;

    fn from_token_scope(self, bytes: &[u8]) -> Result<Self::Scope, InvalidScope> {
        match self {}
    }
}

/// Error type indicating that a scope could not be created from the given token scope.
///
/// As tokens are only accepted from trusted sources, the presence of this error typically
/// indicates a misconfigured trust anchor.
#[derive(Debug, Copy, Clone)]
pub struct InvalidScope;

/// A scope expression that allows all requests.
#[derive(Debug, defmt::Format)]
pub struct AllowAll;

impl Scope for AllowAll {
    fn request_is_allowed<M: ReadableMessage>(&self, _request: &M) -> bool {
        true
    }
}

/// A scope expression that denies all requests.
#[derive(Debug, defmt::Format)]
pub struct DenyAll;

impl Scope for DenyAll {
    fn request_is_allowed<M: ReadableMessage>(&self, _request: &M) -> bool {
        false
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
/// This could be mitigated by sorting the records at construction time.
///
/// This completely disregards proper URI splitting; this works for very simple URI references in
/// the AIF. This could be mitigated by switching to a CRI based model.
#[derive(Debug, defmt::Format)]
pub struct AifValue([u8; AIF_SCOPE_MAX_LEN]);

impl TryFrom<&[u8]> for AifValue {
    // compatible with heapless's
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() >= AIF_SCOPE_MAX_LEN {
            return Err(());
        }
        let mut new = [0; AIF_SCOPE_MAX_LEN];
        new[..value.len()].copy_from_slice(&value);
        Ok(AifValue(new))
    }
}

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

    fn is_admin(&self) -> bool {
        self.0[0] >= 0x83
    }
}

/// A scope generator that parses the scope's bytes as AIF, accepting any value within that model.
pub struct ParsingAif;

impl ScopeGenerator for ParsingAif {
    type Scope = AifValue;

    fn from_token_scope(self, bytes: &[u8]) -> Result<Self::Scope, InvalidScope> {
        let mut buffer = [0; AIF_SCOPE_MAX_LEN];

        if bytes.len() <= buffer.len() {
            buffer[..bytes.len()].copy_from_slice(bytes);
        } else {
            return Err(InvalidScope);
        }

        let mut decoder = minicbor::Decoder::new(bytes);
        for item in decoder
            .array_iter::<(&str, u32)>()
            .map_err(|_| InvalidScope)?
        {
            let (path, mask) = item.map_err(|_| InvalidScope)?;
            if !path.starts_with("/") {
                return Err(InvalidScope);
            }
        }

        Ok(AifValue(buffer))
    }
}

/// A scope that can use multiple backends, erasing its type.
///
/// (Think "`dyn Scope`" but without requiring dyn compatibility).
///
/// This is useful when combining multiple authentication methods, eg. allowing ACE tokens (that
/// need an [`AifValue`] to express their arbitrary scopes) as well as a configured admin key (that
/// has "all" permission, which are not expressible in an [`AifValue`].
#[derive(Debug, defmt::Format)]
pub enum UnionScope {
    AifValue(AifValue),
    AllowAll,
    DenyAll,
}

impl Scope for UnionScope {
    fn request_is_allowed<M: ReadableMessage>(&self, request: &M) -> bool {
        match self {
            UnionScope::AifValue(v) => v.request_is_allowed(request),
            UnionScope::AllowAll => AllowAll.request_is_allowed(request),
            UnionScope::DenyAll => DenyAll.request_is_allowed(request),
        }
    }

    fn is_admin(&self) -> bool {
        match self {
            UnionScope::AifValue(v) => v.is_admin(),
            UnionScope::AllowAll => AllowAll.is_admin(),
            UnionScope::DenyAll => DenyAll.is_admin(),
        }
    }
}

impl From<AifValue> for UnionScope {
    fn from(value: AifValue) -> Self {
        UnionScope::AifValue(value)
    }
}

impl From<AllowAll> for UnionScope {
    fn from(_value: AllowAll) -> Self {
        UnionScope::AllowAll
    }
}

impl From<DenyAll> for UnionScope {
    fn from(_value: DenyAll) -> Self {
        UnionScope::DenyAll
    }
}

impl From<core::convert::Infallible> for UnionScope {
    fn from(value: core::convert::Infallible) -> Self {
        match value {}
    }
}
