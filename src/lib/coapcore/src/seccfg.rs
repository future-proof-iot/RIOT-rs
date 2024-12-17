//! Descriptions of ACE Authorization Servers (AS) and other trust anchors, as viewed from the
//! Resource Server (RS) which coapcore runs on.

use crate::ace::HeaderMap;

/// The error type for [`ServerSecurityConfig::decrypt_symmetric_token`] and future similar
/// methods.
#[derive(Debug)]
pub enum DecryptionError {
    /// A key was indicated that is not available.
    NoKeyFound,
    /// Details of the encrypted message msimatch.
    ///
    /// For example, the nonce size could not match the nonce size expected by the indicated key's
    /// algorithm.
    InconsistentDetails,
    /// The decryption itself failed, indicating mismatch of the keys.
    DecryptionError,
}

/// A single or collection of authorization servers that a handler trusts to create ACE tokens.
pub trait ServerSecurityConfig: crate::Sealed {
    /// True if the type will at any time need to process tokens at /authz-info
    ///
    /// This is used by the handler implementation to shortcut through some message processing
    /// paths.
    const PARSES_TOKENS: bool;

    /// The way scopes issued with this system as audience by this AS are expressed here.
    type Scope: crate::scope::Scope;
    // Can't `-> Result<impl ..., _>` here because that would capture lifetimes we don't want
    // captured
    type ScopeGenerator: crate::scope::ScopeGenerator<Scope = Self::Scope>;

    /// Unprotect a symmetriclly encrypted token.
    ///
    /// It would be preferable to return a decryption key and let the `ace` module do the
    /// decryption, but the key is not dyn safe, and [`aead::AeadInPlace`] can not be enum'd around
    /// different potential key types because the associated types are fixed length. (Returning a
    /// key in some COSE crypto abstraction may work better).
    ///
    /// Note that the full AAD (COSE's AAD including the external AAD) is built by the caller; the
    /// headers are only passed in to enable the AS to select the right key.
    ///
    /// The buffer is given as heapless buffer rather than an an [`aead::Buffer`] because the
    /// latter is not on the latest heaples version in its released version.
    ///
    /// On success, the ciphertext_buffer contains the decrypted and verified plaintext.
    #[allow(
        unused_variables,
        reason = "Names are human visible part of API description"
    )]
    // The method is already sealed by the use of a HeaderMap, but that may become more public over
    // time, and that should not impct this method's publicness.
    fn decrypt_symmetric_token<const N: usize>(
        &self,
        headers: &HeaderMap,
        aad: &[u8],
        ciphertext_buffer: &mut heapless::Vec<u8, N>,
        _: crate::PrivateMethod,
    ) -> Result<Self::ScopeGenerator, DecryptionError> {
        Err(DecryptionError::NoKeyFound)
    }

    /// Generates the scope representing unauthenticated access.
    fn nosec_authorization(&self) -> Option<Self::Scope> {
        None
    }

    /// Generates the single scope expected by the `tests/coap` demo.
    ///
    /// FIXME: This should be replaced with configuration in that example.
    fn the_one_known_authorization(&self) -> Option<Self::Scope> {
        None
    }

    /// Render the "not allowed" message in this scenario.
    ///
    /// The default (or any error) renderer produces a generic 4.01 Unauthorized in the handler;
    /// specifics can be useful in ACE scenarios to return a Request Creation Hint.
    fn render_not_allowed<M: coap_message::MutableWritableMessage>(
        &self,
        message: &mut M,
    ) -> Result<(), ()> {
        Err(())
    }
}

/// Type list of authorization servers. Any operation is first tried on the first item, then on the
/// second.
///
/// It's convention to have a single A1 and then another chain in A2 or an [`DenyAll`], but that's
/// mainly becuse that version is easiy to construct
///
/// In case of doubt, the head is used; in particular, it is only the head that gets to render the
/// unauthorized response.
pub struct AsChain<A1, A2, Scope> {
    a1: A1,
    a2: A2,
    _phantom: core::marker::PhantomData<Scope>,
}

impl<A1, A2, Scope> AsChain<A1, A2, Scope> {
    /// Creates a configuration that processes all operations through the `head`, and only if that
    /// fails retries with the `tail`.
    pub fn chain(head: A1, tail: A2) -> Self {
        AsChain {
            a1: head,
            a2: tail,
            _phantom: Default::default(),
        }
    }
}

/// An `Either` style type for encapsulating two [`ScopeGenerator`] implementations.
///
/// Other crates should not rely on this (but making it an enum wrapped in a struct for privacy is
/// considered excessive at this point).
#[doc(hidden)]
pub enum EitherScopeGenerator<SG1, SG2, Scope> {
    First(SG1),
    Second(SG2),
    Phantom(core::convert::Infallible, core::marker::PhantomData<Scope>),
}

impl<SG1, SG2, Scope> crate::scope::ScopeGenerator for EitherScopeGenerator<SG1, SG2, Scope>
where
    Scope: crate::scope::Scope,
    SG1: crate::scope::ScopeGenerator,
    SG2: crate::scope::ScopeGenerator,
    SG1::Scope: Into<Scope>,
    SG2::Scope: Into<Scope>,
{
    type Scope = Scope;

    fn from_token_scope(self, bytes: &[u8]) -> Result<Self::Scope, crate::scope::InvalidScope> {
        Ok(match self {
            EitherScopeGenerator::First(gen) => gen.from_token_scope(bytes)?.into(),
            EitherScopeGenerator::Second(gen) => gen.from_token_scope(bytes)?.into(),
            EitherScopeGenerator::Phantom(infallible, _) => match infallible {},
        })
    }
}

impl<A1, A2, Scope> crate::Sealed for AsChain<A1, A2, Scope> {}

impl<A1, A2, Scope> ServerSecurityConfig for AsChain<A1, A2, Scope>
where
    A1: ServerSecurityConfig,
    A2: ServerSecurityConfig,
    Scope: crate::scope::Scope,
    A1::Scope: Into<Scope>,
    A2::Scope: Into<Scope>,
{
    const PARSES_TOKENS: bool = A1::PARSES_TOKENS || A2::PARSES_TOKENS;

    type Scope = Scope;
    type ScopeGenerator = EitherScopeGenerator<A1::ScopeGenerator, A2::ScopeGenerator, Self::Scope>;

    fn decrypt_symmetric_token<const N: usize>(
        &self,
        headers: &HeaderMap,
        aad: &[u8],
        ciphertext_buffer: &mut heapless::Vec<u8, N>,
        _priv: crate::PrivateMethod,
    ) -> Result<Self::ScopeGenerator, DecryptionError> {
        if let Ok(sg) = self
            .a1
            .decrypt_symmetric_token(headers, aad, ciphertext_buffer, _priv)
        {
            return Ok(EitherScopeGenerator::First(sg));
        }
        match self
            .a2
            .decrypt_symmetric_token(headers, aad, ciphertext_buffer, _priv)
        {
            Ok(sg) => Ok(EitherScopeGenerator::Second(sg)),
            Err(e) => Err(e),
        }
    }

    fn render_not_allowed<M: coap_message::MutableWritableMessage>(
        &self,
        message: &mut M,
    ) -> Result<(), ()> {
        self.a1.render_not_allowed(message)
    }
}

/// The default empty configuration that denies all access.
pub struct DenyAll;

impl crate::Sealed for DenyAll {}

impl ServerSecurityConfig for DenyAll {
    const PARSES_TOKENS: bool = false;

    type Scope = core::convert::Infallible;
    type ScopeGenerator = core::convert::Infallible;
}

/// A ScopeGenerator that can be used on [`ServerSecurityConfig`] types that don't process tokens
///
/// Unlike [`core::convert::Infallible`], this produces none of any scope, rather tha none of
/// [`Infallible`][core::convert::Infallible].
pub enum NullGenerator<Scope> {
    _Phantom(core::convert::Infallible, core::marker::PhantomData<Scope>),
}

impl<Scope: crate::scope::Scope> crate::scope::ScopeGenerator for NullGenerator<Scope> {
    type Scope = Scope;

    fn from_token_scope(self, bytes: &[u8]) -> Result<Self::Scope, crate::scope::InvalidScope> {
        match self {
            NullGenerator::_Phantom(infallible, _) => match infallible {},
        }
    }
}

/// An SSC representing unconditionally allowed access, including unencrypted.
pub struct AllowAll;

impl crate::Sealed for AllowAll {}

impl ServerSecurityConfig for AllowAll {
    const PARSES_TOKENS: bool = false;

    type Scope = crate::scope::AllowAll;
    type ScopeGenerator = NullGenerator<Self::Scope>;

    fn nosec_authorization(&self) -> Option<Self::Scope> {
        Some(crate::scope::AllowAll)
    }
}

/// A scope that provides a non-trivial implementation of
/// [`ServerSecurityConfig::the_one_known_authorization()`], see there.
///
/// FIXME: Like that function, this should be moved into the demo.
pub struct GenerateArbitrary;

impl crate::Sealed for GenerateArbitrary {}

impl ServerSecurityConfig for GenerateArbitrary {
    const PARSES_TOKENS: bool = false;

    type Scope = crate::scope::AifValue;
    type ScopeGenerator = NullGenerator<crate::scope::AifValue>;

    fn nosec_authorization(&self) -> Option<Self::Scope> {
        use cbor_macro::cbor;
        let slice: &[u8] = &cbor!([["/.well-known/core", 1], ["/poem", 1]]);
        crate::scope::AifValue::try_from(slice).ok()
    }

    fn the_one_known_authorization(&self) -> Option<Self::Scope> {
        use cbor_macro::cbor;
        let slice: &[u8] = &cbor!([
                ["/stdout", 17 / GET and FETCH /],
                ["/.well-known/core", 1],
                ["/poem", 1]
        ]);
        crate::scope::AifValue::try_from(slice).ok()
    }
}

/// A test SSC association that does not need to deal with key IDs and just tries a single static
/// key with a single algorithm, and parses the scope in there as AIF.
///
/// It sends a static response (empty slice is a fine default) on unauthorized responses.
pub struct StaticSymmetric31 {
    key: &'static [u8; 32],
    unauthorized_response: &'static [u8],
}

impl StaticSymmetric31 {
    pub fn new(key: &'static [u8; 32], unauthorized_response: &'static [u8]) -> Self {
        Self {
            key,
            unauthorized_response,
        }
    }
}
impl crate::Sealed for StaticSymmetric31 {}

impl ServerSecurityConfig for StaticSymmetric31 {
    const PARSES_TOKENS: bool = true;

    type Scope = crate::scope::AifValue;
    type ScopeGenerator = crate::scope::ParsingAif;

    fn decrypt_symmetric_token<const N: usize>(
        &self,
        headers: &HeaderMap,
        aad: &[u8],
        ciphertext_buffer: &mut heapless::Vec<u8, N>,
        _: crate::PrivateMethod,
    ) -> Result<Self::ScopeGenerator, DecryptionError> {
        use ccm::aead::AeadInPlace;
        use ccm::KeyInit;

        // FIXME: should be something Aes256Ccm::TagLength
        const TAG_SIZE: usize = 16;
        const NONCE_SIZE: usize = 13;

        pub type Aes256Ccm = ccm::Ccm<aes::Aes256, ccm::consts::U16, ccm::consts::U13>;
        let cipher = Aes256Ccm::new(self.key.into());

        let nonce: &[u8; NONCE_SIZE] = headers
            .iv
            .ok_or_else(|| {
                defmt_or_log::error!("Decryption IV");
                DecryptionError::InconsistentDetails
            })?
            .try_into()
            .map_err(|_| {
                defmt_or_log::error!("IV length mismatch");
                DecryptionError::InconsistentDetails
            })?;

        let ciphertext_len = ciphertext_buffer
            .len()
            .checked_sub(TAG_SIZE)
            .ok_or_else(|| {
                defmt_or_log::error!("Ciphertext too short for tag");
                DecryptionError::InconsistentDetails
            })?;

        let (ciphertext, tag) = ciphertext_buffer.split_at_mut(ciphertext_len);

        cipher
            .decrypt_in_place_detached(nonce.into(), aad, ciphertext, ccm::Tag::from_slice(tag))
            .map_err(|_| {
                defmt_or_log::error!("Decryption failed");
                DecryptionError::DecryptionError
            })?;

        ciphertext_buffer.truncate(ciphertext_len);

        Ok(crate::scope::ParsingAif)
    }

    fn render_not_allowed<M: coap_message::MutableWritableMessage>(
        &self,
        message: &mut M,
    ) -> Result<(), ()> {
        use coap_message::Code;
        message.set_code(M::Code::new(coap_numbers::code::UNAUTHORIZED).map_err(|_| ())?);
        message
            .set_payload(self.unauthorized_response)
            .map_err(|_| ())?;
        Ok(())
    }
}
