//! Descriptions of ACE Authorization Servers (AS), as viewed from the Resource Server (RS) which
//! coapcore runs on.

use crate::ace::HeaderMap;

#[derive(Debug)]
pub enum DecryptionError {
    NoKeyFound,
    // Nonce size mismatch, message too short, that kind of thing
    InconsistentDetails,
    DecryptionError,
}

/// A single or collection of authorization servers that a handler trusts to create ACE tokens.
pub trait AsDescription {
    /// True if the type will never find a token.
    ///
    /// This is used by the handler implementation to shortcut through some message processing
    /// paths.
    const IS_EMPTY: bool;

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
    // Note that due to the unnameability of the `HeaderMap` type by outside crates, this is
    // effectively sealed, even though there is no need to seal the whole trait.
    //
    // Note that even though this is dressed as a decrypt-then-read-scope step, tricks such as
    // using ACE-OSCORE with constant short tokens that stand in for known contexts still works --
    // as long as the stored data is small enough to fit in the heapless buffer, where nothing
    // keeps the implementation from expanding data rather than trimming off a signature.
    fn decrypt_symmetric_token<const N: usize>(
        &self,
        headers: &HeaderMap,
        aad: &[u8],
        ciphertext_buffer: &mut heapless::Vec<u8, N>,
    ) -> Result<Self::ScopeGenerator, DecryptionError> {
        Err(DecryptionError::NoKeyFound)
    }
}

/// Type list of authorization servers. Any operation is first tried on the first item, then on the
/// second.
///
/// It's convention to have a single A1 and then another chain in A2 or an [`Empty`], but that's
/// mainly becuse that version is easiy to construct
pub struct AsChain<A1, A2, Scope> {
    a1: A1,
    a2: A2,
    _phantom: core::marker::PhantomData<Scope>,
}

impl<A1, A2, Scope> AsChain<A1, A2, Scope> {
    pub fn chain(head: A1, tail: A2) -> Self {
        AsChain {
            a1: head,
            a2: tail,
            _phantom: Default::default(),
        }
    }
}

// FIXME: seal
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

impl<A1, A2, Scope> AsDescription for AsChain<A1, A2, Scope>
where
    A1: AsDescription,
    A2: AsDescription,
    Scope: crate::scope::Scope,
    A1::Scope: Into<Scope>,
    A2::Scope: Into<Scope>,
{
    const IS_EMPTY: bool = A1::IS_EMPTY && A2::IS_EMPTY;

    type Scope = Scope;
    type ScopeGenerator = EitherScopeGenerator<A1::ScopeGenerator, A2::ScopeGenerator, Self::Scope>;

    fn decrypt_symmetric_token<const N: usize>(
        &self,
        headers: &HeaderMap,
        aad: &[u8],
        ciphertext_buffer: &mut heapless::Vec<u8, N>,
    ) -> Result<Self::ScopeGenerator, DecryptionError> {
        if let Ok(sg) = self
            .a1
            .decrypt_symmetric_token(headers, aad, ciphertext_buffer)
        {
            return Ok(EitherScopeGenerator::First(sg));
        }
        match self
            .a2
            .decrypt_symmetric_token(headers, aad, ciphertext_buffer)
        {
            Ok(sg) => Ok(EitherScopeGenerator::Second(sg)),
            Err(e) => Err(e),
        }
    }
}

/// The empty set of authorization servers.
pub struct Empty;

impl AsDescription for Empty {
    const IS_EMPTY: bool = true;

    type Scope = core::convert::Infallible;
    type ScopeGenerator = core::convert::Infallible;
}

/// A transition helper
#[derive(Default)]
pub struct GenerateDefault<Scope>(core::marker::PhantomData<Scope>);

impl<Scope: crate::scope::Scope + Default> AsDescription for GenerateDefault<Scope> {
    const IS_EMPTY: bool = true;

    type Scope = Scope;
    type ScopeGenerator = Self;
}

impl<Scope: crate::scope::Scope + Default> crate::scope::ScopeGenerator for GenerateDefault<Scope> {
    type Scope = Scope;

    fn from_token_scope(self, bytes: &[u8]) -> Result<Self::Scope, crate::scope::InvalidScope> {
        Ok(Default::default())
    }
}

/// A test AS association that does not need to deal with key IDs and just tries a single static
/// key with a single algorithm, and parses the scope in there as AIF.
pub struct StaticSymmetric31 {
    key: &'static [u8; 32],
}

impl StaticSymmetric31 {
    pub fn new(key: &'static [u8; 32]) -> Self {
        Self { key }
    }
}

impl AsDescription for StaticSymmetric31 {
    const IS_EMPTY: bool = false;

    type Scope = crate::scope::AifValue;
    type ScopeGenerator = crate::scope::ParsingAif;

    fn decrypt_symmetric_token<const N: usize>(
        &self,
        headers: &HeaderMap,
        aad: &[u8],
        ciphertext_buffer: &mut heapless::Vec<u8, N>,
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
}