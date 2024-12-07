//! Descriptions of ACE Authorization Servers (AS), as viewed from the Resource Server (RS) which
//! coapcore runs on.

#[cfg(feature = "acetoken")]
use crate::ace::HeaderMap;

#[cfg(not(feature = "acetoken"))]
mod dummy {
    /// Unnameable type to stand in for `ace::HeaderMap` when the ACE token feature is disable. The
    /// type is needed to allow still having the [`AsDescription`][super::AsDescription] trait
    /// public, because not having that item would vary the generics of the whole crate.
    pub struct HeaderMap {
        _private: (),
    }
}

#[cfg(not(feature = "acetoken"))]
use dummy::HeaderMap;

#[derive(Debug)]
pub enum DecryptionError {
    NoKeyFound,
    // Nonce size mismatch, message too short, that kind of thing
    InconsistentDetails,
    DecryptionError,
}

/// A single or collection of authorization servers that a handler trusts to create ACE tokens.
pub trait AsDescription {
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
    ///
    /// # Evolution
    ///
    /// * Rather than report OK, might this return a closure that maps the decrypted token to an
    ///   application scope? (This would be useful if one wanted to have an AS that uses different
    ///   scope expressions, or is not authorized to issue the full value space of the
    ///   applications' scope type).
    #[allow(
        unused_variables,
        reason = "Names are human visible part of API description"
    )]
    // Note that due to the unnameability of the `HeaderMap` type by outside crates, this is
    // effectively sealed, even though there is no need to seal the whole trait.
    fn decrypt_symmetric_token<const N: usize>(
        &self,
        headers: &HeaderMap,
        aad: &[u8],
        ciphertext_buffer: &mut heapless::Vec<u8, N>,
    ) -> Result<(), DecryptionError> {
        Err(DecryptionError::NoKeyFound)
    }
}

/// Type list of authorization servers. Any operation is first tried on the first item, then on the
/// second.
///
/// It's convention to have a single A1 and then another chain in A2 or an [`Empty`], but that's
/// mainly becuse that version is easiy to construct
pub struct AsChain<A1: AsDescription, A2: AsDescription> {
    a1: A1,
    a2: A2,
}

impl<A1: AsDescription, A2: AsDescription> AsChain<A1, A2> {
    pub fn chain(head: A1, tail: A2) -> Self {
        AsChain { a1: head, a2: tail }
    }
}

impl<A1: AsDescription, A2: AsDescription> AsDescription for AsChain<A1, A2> {
    fn decrypt_symmetric_token<const N: usize>(
        &self,
        headers: &HeaderMap,
        aad: &[u8],
        ciphertext_buffer: &mut heapless::Vec<u8, N>,
    ) -> Result<(), DecryptionError> {
        if self
            .a1
            .decrypt_symmetric_token(headers, aad, ciphertext_buffer)
            .is_ok()
        {
            return Ok(());
        }
        self.a2
            .decrypt_symmetric_token(headers, aad, ciphertext_buffer)
    }
}

/// The empty set of authorization servers.
pub struct Empty;

impl AsDescription for Empty {}

/// A test AS association that does not need to deal with key IDs and just tries a single static
/// key with a single algorithm.
#[cfg(feature = "acetoken")]
pub struct StaticSymmetric31 {
    key: &'static [u8; 32],
}

#[cfg(feature = "acetoken")]
impl StaticSymmetric31 {
    pub fn new(key: &'static [u8; 32]) -> Self {
        Self { key }
    }
}

#[cfg(feature = "acetoken")]
impl AsDescription for StaticSymmetric31 {
    fn decrypt_symmetric_token<const N: usize>(
        &self,
        headers: &HeaderMap,
        aad: &[u8],
        ciphertext_buffer: &mut heapless::Vec<u8, N>,
    ) -> Result<(), DecryptionError> {
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

        Ok(())
    }
}
