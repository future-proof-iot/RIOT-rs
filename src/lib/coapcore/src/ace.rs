//! Experimental types for ACE, COSE and CWT structures
//!
//! On the long run, those might contribute to
//! <https://github.com/namib-project/dcaf-rs/issues/29>.
//!
//! The module is private, but contains a few pub items so that they can be used in the
//! [`authorization_server`][crate::authorization_server] crate on sealed traits.

use defmt_or_log::trace;

use crate::seccontext::COwn;

/// Fixed length of the ACE OSCORE nonce issued by this module.
pub const OWN_NONCE_LEN: usize = 8;

/// Size allocated for the ACE OSCORE nonces chosen by the peers.
const MAX_SUPPORTED_PEER_NONCE_LEN: usize = 16;

/// Maximum size a CWT processed by this module can have (at least when it needs to be copied)
const MAX_SUPPORTED_ACCESSTOKEN_LEN: usize = 256;

/// The content of an application/ace+cbor file.
///
/// Full attribute references are in the [OAuth Parameters CBOR Mappings
/// registry](https://www.iana.org/assignments/ace/ace.xhtml#oauth-parameters-cbor-mappings).
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(minicbor::Decode, minicbor::Encode, Default)]
#[cbor(map)]
#[non_exhaustive]
struct AceCbor<'a> {
    #[cbor(b(1), with = "minicbor::bytes")]
    access_token: Option<&'a [u8]>,
    #[cbor(b(40), with = "minicbor::bytes")]
    nonce1: Option<&'a [u8]>,
    #[cbor(b(42), with = "minicbor::bytes")]
    nonce2: Option<&'a [u8]>,
    #[cbor(b(43), with = "minicbor::bytes")]
    ace_client_recipientid: Option<&'a [u8]>,
    #[cbor(b(44), with = "minicbor::bytes")]
    ace_server_recipientid: Option<&'a [u8]>,
}

/// The content of a POST to the /authz-info endpoint of a client.
///
/// # Open questions
/// Should we subset the type to add more constraints on fields?
///
/// * Pro type alias: Shared parsing code for all cases.
/// * Pro subtype: Easier usability, errors directly from minicbor.
type UnprotectedAuthzInfoPost<'a> = AceCbor<'a>;

/// A COSE header map.
///
/// Full attribute references are in the [COSE Header Parameters
/// registry](https://www.iana.org/assignments/cose/cose.xhtml#header-parameters).
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(minicbor::Decode)]
#[cbor(map)]
#[non_exhaustive]
pub struct HeaderMap<'a> {
    #[n(1)]
    // Might be extended as more exotic algorithms are supported
    pub alg: Option<i32>,
    #[cbor(b(5), with = "minicbor::bytes")]
    pub iv: Option<&'a [u8]>,
}

impl<'a> HeaderMap<'a> {
    /// Merge two header maps, using the latter's value in case of conflict.
    fn updated_with(self, other: Self) -> Self {
        Self {
            alg: self.alg.or(other.alg),
            iv: self.iv.or(other.iv),
        }
    }
}

/// A COSE_Encrypt0 structure as defined in [RFC8152](https://www.rfc-editor.org/rfc/rfc8152)
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(minicbor::Decode)]
#[cbor(tag(16))]
#[non_exhaustive]
struct CoseEncrypt0<'a> {
    #[cbor(b(0), with = "minicbor::bytes")]
    protected: &'a [u8],
    #[b(1)]
    unprotected: HeaderMap<'a>,
    #[cbor(b(2), with = "minicbor::bytes")]
    encrypted: &'a [u8],
}

type EncryptedCwt<'a> = CoseEncrypt0<'a>;

/// A CWT Claims Set.
///
/// Full attribute references are in the [CWT Claims
/// registry](https://www.iana.org/assignments/cwt/cwt.xhtml#claims-registry).
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(minicbor::Decode)]
#[cbor(map)]
#[non_exhaustive]
struct CwtClaimsSet<'a> {
    #[n(4)]
    exp: u64,
    #[n(6)]
    iat: u64,
    #[b(8)]
    cnf: Cnf<'a>,
    #[cbor(b(9), with = "minicbor::bytes")]
    scope: &'a [u8],
}

/// A single CWT Claims Set Confirmation value.
///
/// All possible variants are in the [CWT Confirmation Methods
/// registry](https://www.iana.org/assignments/cwt/cwt.xhtml#confirmation-methods).
///
/// ## Open questions
///
/// This should be an enum, but minicbor-derive can only have them as `array(2)` or using
/// `index_only`. Can this style of an enum be added to minicbor?
///
/// Or is it really an enum? RFC8747 just [talks
/// of](https://www.rfc-editor.org/rfc/rfc8747.html#name-confirmation-claim) "At most one of the
/// `COSE_Key` and `Encrypted_COSE_Key` […] may be present", doesn't rule out that items without
/// key material can't be attached.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(minicbor::Decode)]
#[cbor(map)]
#[non_exhaustive]
struct Cnf<'a> {
    #[b(4)]
    osc: Option<OscoreInputMaterial<'a>>,
}

/// OSCORE_Input_Material.
///
/// All current parameters are described in [Section 3.2.1 of
/// RFC9203](https://datatracker.ietf.org/doc/html/rfc9203#name-the-oscore_input_material); the
/// [OSCORE Security Context Parameters
/// registry](https://www.iana.org/assignments/ace/ace.xhtml#oscore-security-context-parameters)
/// has the full set in case it gets extended.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(minicbor::Decode)]
#[cbor(map)]
#[non_exhaustive]
struct OscoreInputMaterial<'a> {
    #[cbor(b(0), with = "minicbor::bytes")]
    id: &'a [u8],
    #[cbor(b(2), with = "minicbor::bytes")]
    ms: &'a [u8],
}

struct DeriveError;

impl<'a> OscoreInputMaterial<'a> {
    fn derive(
        &self,
        nonce1: &[u8],
        nonce2: &[u8],
        sender_id: &[u8],
        recipient_id: &[u8],
    ) -> Result<liboscore::PrimitiveContext, DeriveError> {
        // We don't process the algorithm fields
        let hkdf = liboscore::HkdfAlg::from_number(5).expect("Default algorithm is supported");
        let aead = liboscore::AeadAlg::from_number(10).expect("Default algorithm is supported");

        // This is the only really custom part of ACE-OSCORE; the rest is just passing around
        // inputs.
        const { assert!(OWN_NONCE_LEN < 256) };
        const { assert!(MAX_SUPPORTED_PEER_NONCE_LEN < 256) };
        let mut combined_salt =
            heapless::Vec::<u8, { 1 + 2 + MAX_SUPPORTED_PEER_NONCE_LEN + 2 + OWN_NONCE_LEN }>::new(
            );
        let mut encoder =
            minicbor::Encoder::new(minicbor_adapters::WriteToHeapless(&mut combined_salt));
        // We don't process the salt field
        encoder
            .bytes(b"")
            .and_then(|encoder| encoder.bytes(nonce1))
            .and_then(|encoder| encoder.bytes(nonce2))
            .map_err(|_| DeriveError)?;

        let immutables = liboscore::PrimitiveImmutables::derive(
            hkdf,
            self.ms,
            &combined_salt,
            None, // context ID field not processed
            aead,
            sender_id,
            recipient_id,
        )
        .map_err(|_| DeriveError)?;

        // It is fresh because it is derived from.
        Ok(liboscore::PrimitiveContext::new_from_fresh_material(
            immutables,
        ))
    }
}

pub struct AceCborAuthzInfoResponse {
    nonce2: [u8; OWN_NONCE_LEN],
    ace_server_recipientid: COwn,
}

impl AceCborAuthzInfoResponse {
    pub fn render<M: coap_message::MutableWritableMessage>(
        &self,
        message: &mut M,
    ) -> Result<(), M::UnionError> {
        let full = AceCbor {
            nonce2: Some(&self.nonce2),
            ace_server_recipientid: Some(self.ace_server_recipientid.as_slice()),
            ..Default::default()
        };

        use coap_message::Code;
        message.set_code(M::Code::new(coap_numbers::code::CHANGED)?);

        const { assert!(OWN_NONCE_LEN < 256) };
        const { assert!(COwn::MAX_SLICE_LEN < 256) };
        let required_len = 1 + 2 + 2 + OWN_NONCE_LEN + 2 + 2 + COwn::MAX_SLICE_LEN;
        let payload = message.payload_mut_with_len(required_len)?;

        let mut cursor = minicbor::encode::write::Cursor::new(payload);
        minicbor::encode(full, &mut cursor).expect("Sufficient size was requested");
        let written = cursor.position();
        message.truncate(written)?;

        Ok(())
    }
}

/// Given an application/ace+cbor payload as is posted to an /authz-info endpoint, decrypt all
/// that's needed for the ACE-OSCORE profile.
///
/// This needs to be provided with
///
/// * the request's `payload`
/// * a list of recognized `authorities` (Authorization Servers) to authenticate the token
/// * a random nonce2
/// * a callback that, once the peer's recipient ID is known, chooses an own recipient ID
///   (because it's up to the pool of security contexts to pick one, and the peers can not pick
///   identical ones)
/// * a callback that parses the scope into the client's concept of a scope
///
/// ## Caveats
///
/// * Currently, this hardcodes the key.
///
/// * This allocates on the stack for two fields: the AAD and the token's plaintext. Both will
///   eventually need to be configurable.
///
///   Alternatives to allocation are streaming AADs for the AEAD traits, and coap-handler offering
///   an exclusive reference to the incoming message.
///
/// * Should the scope parser be provided with anything else, in particular maybe the AS's
///   identity?
///
/// * Instead of the random nonce2, it would be preferable to pass in an RNG -- but some owners of
///   an RNG may have a hard time lending out an exclusive reference to it for the whole function
///   call duration.
pub fn process_acecbor_authz_info<Scope>(
    payload: &[u8],
    authorities: &impl crate::authorization_server::AsDescription,
    nonce2: [u8; OWN_NONCE_LEN],
    server_recipient_id: impl FnOnce(&[u8]) -> COwn,
    parse_scope: impl FnOnce(&[u8]) -> Scope,
) -> Result<(AceCborAuthzInfoResponse, Scope, liboscore::PrimitiveContext), minicbor::decode::Error>
{
    trace!("Processing authz_info {:#02x}", payload);

    let decoded: UnprotectedAuthzInfoPost = minicbor::decode(payload)?;
    // FIXME: The `..` should be "all others are None"; se also comment on UnprotectedAuthzInfoPost
    // on type alias vs new type
    let AceCbor {
        access_token: Some(access_token),
        nonce1: Some(nonce1),
        ace_client_recipientid: Some(ace_client_recipientid),
        ..
    } = decoded
    else {
        return Err(minicbor::decode::Error::message("Missing fields"));
    };

    trace!("Decodeded application/ace+cbor: {}", decoded);

    let encrypt0: EncryptedCwt = minicbor::decode(access_token)?;

    trace!("Token decoded as Encrypt0: {}", encrypt0);

    // Could have the extra exception for empty byte strings expressing the empty map, but we don't
    // encounter this here
    let protected: HeaderMap = minicbor::decode(encrypt0.protected)?;

    trace!("Protected decoded as header map: {}", protected);

    let headers = encrypt0.unprotected.updated_with(protected);

    // Can't go through liboscore's decription backend b/c that expects unprotect-in-place; doing
    // somethign more custom on a bounded copy instead, and this is part of where dcaf on alloc
    // could shine by getting an exclusive copy of something in RAM

    if headers.alg != Some(31) {
        return Err(minicbor::decode::Error::message("unknown algorithm"));
    }

    #[derive(minicbor::Encode)]
    struct Encrypt0<'a> {
        #[n(0)]
        context: &'static str,
        #[cbor(b(1), with = "minicbor::bytes")]
        protected: &'a [u8],
        #[cbor(b(2), with = "minicbor::bytes")]
        external_aad: &'a [u8],
    }
    let aad = Encrypt0 {
        context: "Encrypt0",
        protected: encrypt0.protected,
        external_aad: &[],
    };
    let mut aad_encoded = heapless::Vec::<u8, MAX_SUPPORTED_ACCESSTOKEN_LEN>::new();
    minicbor::encode(&aad, minicbor_adapters::WriteToHeapless(&mut aad_encoded))
        .map_err(|_| minicbor::decode::Error::message("AAD too long"))?;
    trace!("Serialized AAD: {:#02x}", aad_encoded);

    let mut ciphertext_buffer =
        heapless::Vec::<u8, MAX_SUPPORTED_ACCESSTOKEN_LEN>::from_slice(encrypt0.encrypted)
            .map_err(|_| minicbor::decode::Error::message("Token too long to decrypt"))?;

    authorities
        .decrypt_symmetric_token(&headers, &aad_encoded, &mut ciphertext_buffer)
        .map_err(|_| minicbor::decode::Error::message("Decryption failed"))?;

    let claims: CwtClaimsSet = minicbor::decode(ciphertext_buffer.as_slice())?;
    trace!("Decrypted CWT claims: {}", claims);

    let scope = parse_scope(claims.scope);

    let Cnf { osc: Some(osc) } = claims.cnf else {
        return Err(minicbor::decode::Error::message(
            "osc field missing in cnf.",
        ));
    };

    let ace_server_recipientid = server_recipient_id(ace_client_recipientid);

    let derived = osc
        .derive(
            nonce1,
            &nonce2,
            ace_client_recipientid,
            ace_server_recipientid.as_slice(),
        )
        // And at latest herer it's *definitely* not a minicbor error any more
        .map_err(|_| minicbor::decode::Error::message("OSCORE derivation failed"))?;

    let response = AceCborAuthzInfoResponse {
        nonce2,
        ace_server_recipientid,
    };

    Ok((response, scope, derived))
}
