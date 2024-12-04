use coap_message::{
    error::RenderableOnMinimal, Code, MessageOption, MinimalWritableMessage,
    MutableWritableMessage, ReadableMessage,
};
use coap_message_utils::{Error as CoAPError, OptionsExt as _};
use defmt_or_log::{debug, error, info, warn, Debug2Format};

// If this exceeds 47, COwn will need to be extended.
const MAX_CONTEXTS: usize = 4;

/// A pool of security contexts shareable by several users inside a thread.
#[expect(private_interfaces, reason = "should be addressed eventually")]
pub type SecContextPool<Crypto> =
    crate::oluru::OrderedPool<SecContextState<Crypto>, MAX_CONTEXTS, LEVEL_COUNT>;

/// An own identifier for a security context
///
/// This is used as C_I when in an initiator role, C_R when in a responder role, and recipient ID
/// in OSCORE.
///
/// This type represents any of the 48 efficient identifiers that use CBOR one-byte integer
/// encodings (see RFC9528 Section 3.3.2), or equivalently the 1-byte long OSCORE identifiers
///
/// Lakers supports a much larger value space for C_x, and coapcore processes larger values
/// selected by the peer -- but on its own, will select only those that fit in this type.
// FIXME Could even limit to positive values if MAX_CONTEXTS < 24
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Debug)]
struct COwn(u8);

impl COwn {
    /// Find a value of self that is not found in the iterator.
    ///
    /// This asserts that the iterator is (known to be) short enough that this will always succeed.
    fn not_in_iter(iterator: impl Iterator<Item = Self>) -> Self {
        // In theory, this would allow the compiler to see that the unreachable below is indeed
        // unreachable
        assert!(
            iterator.size_hint().1.is_some_and(|v| v < 48),
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
    fn from_kid(kid: &[u8]) -> Option<Self> {
        match kid {
            [first] if *first <= 0x17 || (*first >= 0x20 && *first <= 0x37) => Some(Self(*first)),
            _ => None,
        }
    }
}

impl From<COwn> for lakers::ConnId {
    fn from(cown: COwn) -> Self {
        lakers::ConnId::from_slice(&[cown.0])
            .expect("ConnId is always big enough for at least COwn")
    }
}

/// Copy of the OSCORE option
type OscoreOption = heapless::Vec<u8, 16>;

/// A representation of an RFC9237 using the REST-specific model in a CRI variation (Toid =
/// [*path], Tperm = u32).
///
/// FIXME: At the moment, this always represents an authorization that allows everything, and only
/// has runtime information about whether or not stdout is allowed. On the long run, this will
/// likely be a CBOR item with pre-verified structure.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
struct AifStaticRest {
    may_use_stdout: bool,
}

impl AifStaticRest {
    fn request_is_allowed<M: ReadableMessage>(&self, request: &M) -> bool {
        // BIG FIXME: We're iterating over options without checking for critical options. If the
        // resource handler router consumes any different set of options, that disagreement might
        // give us a security issue.
        //
        // and FIXME this is using block-lists, but at this point it should be obvious that this is
        // just a stupid stand-in.

        let mut uri_path_options = request
            .options()
            .filter(|o| o.number() == coap_numbers::option::URI_PATH);
        if uri_path_options
            .next()
            .is_some_and(|o| o.value() == b"stdout")
            && uri_path_options.next().is_none()
        {
            self.may_use_stdout
        } else {
            true
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
struct SecContextState<Crypto: lakers::Crypto> {
    // FIXME: Should also include timeout. How do? Store expiry, do raytime in not-even-RTC mode,
    // and whenever there is a new time stamp from AS, remove old ones?
    authorization: AifStaticRest,
    protocol_stage: SecContextStage<Crypto>,
}

impl<Crypto: lakers::Crypto> Default for SecContextState<Crypto> {
    fn default() -> Self {
        Self {
            authorization: AifStaticRest {
                may_use_stdout: false,
            },
            protocol_stage: SecContextStage::Empty,
        }
    }
}

#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "requiring more memory during connection setup is expected, but the complexity of an inhmogenous pool is currently impractical"
)]
enum SecContextStage<Crypto: lakers::Crypto> {
    Empty,

    // if we have time to spare, we can have empty-but-prepared-with-single-use-random-key entries
    // :-)

    // actionable in response building
    EdhocResponderProcessedM1 {
        responder: lakers::EdhocResponderProcessedM1<Crypto>,
        // May be removed if lakers keeps access to those around if they are set at this point at
        // all
        c_r: COwn,
        c_i: lakers::ConnId,
    },
    //
    EdhocResponderSentM2 {
        responder: lakers::EdhocResponderWaitM3<Crypto>,
        c_r: COwn,
        c_i: lakers::ConnId,
    },

    // FIXME: Also needs a flag for whether M4 was received; if not, it's GC'able
    Oscore(liboscore::PrimitiveContext),
}

#[cfg(feature = "defmt")]
impl<Crypto: lakers::Crypto> defmt::Format for SecContextStage<Crypto> {
    fn format(&self, f: defmt::Formatter) {
        match self {
            SecContextStage::Empty => defmt::write!(f, "Empty"),
            SecContextStage::EdhocResponderProcessedM1 { c_r, .. } => {
                defmt::write!(f, "EdhocResponderProcessedM1 {{ c_r: {:?}, ... }}", c_r)
            }
            SecContextStage::EdhocResponderSentM2 { c_r, .. } => {
                defmt::write!(f, "EdhocResponderSentM2 {{ c_r: {:?}, ... }}", c_r)
            }
            SecContextStage::Oscore(primitive_context) => defmt::write!(
                f,
                "Oscore(with recipient_id {:?})",
                primitive_context.recipient_id()
            ),
        }
    }
}

impl<Crypto: lakers::Crypto> core::fmt::Display for SecContextState<Crypto> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        use SecContextStage::*;
        match &self.protocol_stage {
            Empty => f.write_str("empty"),
            EdhocResponderProcessedM1 { c_r, .. } => write!(f, "ProcessedM1, C_R = {:?}", c_r),
            EdhocResponderSentM2 { c_r, .. } => write!(f, "SentM3, C_R = {:?}", c_r),
            Oscore(ctx) => write!(
                f,
                "OSCORE, C_R = {:?}",
                COwn::from_kid(ctx.recipient_id()).unwrap()
            ),
        }?;
        write!(
            f,
            " authorized to read stdin? {}",
            self.authorization.may_use_stdout
        )?;
        Ok(())
    }
}

const LEVEL_ADMIN: usize = 0;
const LEVEL_AUTHENTICATED: usize = 1;
const LEVEL_ONGOING: usize = 2;
const LEVEL_EMPTY: usize = 3;
const LEVEL_COUNT: usize = 4;

impl<Crypto: lakers::Crypto> crate::oluru::PriorityLevel for SecContextState<Crypto> {
    fn level(&self) -> usize {
        match &self.protocol_stage {
            SecContextStage::Empty => LEVEL_EMPTY,
            SecContextStage::EdhocResponderProcessedM1 { .. } => {
                // If this is ever tested, means we're outbound message limited, so let's try to
                // get one through rather than pointlessly sending errors
                LEVEL_ONGOING
            }
            SecContextStage::EdhocResponderSentM2 { .. } => {
                // So far, the peer didn't prove they have anything other than entropy (maybe not
                // even that)
                LEVEL_ONGOING
            }
            SecContextStage::Oscore(_) => {
                if self.authorization.may_use_stdout {
                    LEVEL_ADMIN
                } else {
                    LEVEL_AUTHENTICATED
                }
            }
        }
    }
}

impl<Crypto: lakers::Crypto> SecContextState<Crypto> {
    fn corresponding_cown(&self) -> Option<COwn> {
        match &self.protocol_stage {
            SecContextStage::Empty => None,
            // We're keeping a c_r in there assigned early so that we can find the context when
            // building the response; nothing in the responder is tied to c_r yet.
            SecContextStage::EdhocResponderProcessedM1 { c_r, .. } => Some(*c_r),
            SecContextStage::EdhocResponderSentM2 { c_r, .. } => Some(*c_r),
            SecContextStage::Oscore(ctx) => COwn::from_kid(ctx.recipient_id()),
        }
    }
}

/// A CoAP handler wrapping inner resources, and adding EDHOC and OSCORE support.
///
/// While the EDHOC part could be implemented as a handler that is to be added into the tree, the
/// OSCORE part needs to wrap the inner handler anyway, and EDHOC and OSCORE are intertwined rather
/// strongly in processing the EDHOC option.
pub struct OscoreEdhocHandler<
    'a,
    H: coap_handler::Handler,
    Crypto: lakers::Crypto,
    CryptoFactory: Fn() -> Crypto,
> {
    // It'd be tempted to have sharing among multiple handlers for multiple CoAP stacks, but
    // locks for such sharing could still be acquired in a factory (at which point it may make
    // sense to make this a &mut).
    pool: SecContextPool<Crypto>,
    own_identity: (&'a lakers::Credential, &'a lakers::BytesP256ElemLen),

    // FIXME: This currently bakes in the assumption that there is a single tree both for
    // unencrypted and encrypted resources. We may later generalize this by making this a factory,
    // or a single item that has two AsMut<impl Handler> accessors for separate encrypted and
    // unencrypted tree.

    // FIXME That assumption could be easily violated by code changes that don't take the big
    // picture into account. It might make sense to wrap the inner into some
    // zero-cost/build-time-only wrapper that verifies that either request_is_allowed() has been
    // called, or an AuthorizationChecked::Allowed is around.
    inner: H,

    crypto_factory: CryptoFactory,
}

impl<'a, H: coap_handler::Handler, Crypto: lakers::Crypto, CryptoFactory: Fn() -> Crypto>
    OscoreEdhocHandler<'a, H, Crypto, CryptoFactory>
{
    // FIXME: Apart from an own identity, this will also need a function to convert ID_CRED_I into
    // a (CRED_I, AifStaticRest) pair.
    pub fn new(
        own_identity: (&'a lakers::Credential, &'a lakers::BytesP256ElemLen),
        inner: H,
        crypto_factory: CryptoFactory,
    ) -> Self {
        Self {
            pool: Default::default(),
            own_identity,
            inner,
            crypto_factory,
        }
    }

    // FIXME: this should be configurable
    fn unauthenticated_edhoc_user_authorization(&self) -> AifStaticRest {
        AifStaticRest {
            may_use_stdout: false,
        }
    }

    // FIXME: this should be configurable
    fn nosec_authorization(&self) -> AifStaticRest {
        AifStaticRest {
            may_use_stdout: false,
        }
    }

    /// Process a CoAP request containing a message sent to /.well-known/edhoc.
    ///
    /// The caller has already checked Uri-Path and all other critical options.
    fn extract_edhoc<M: ReadableMessage>(
        &mut self,
        request: &M,
    ) -> Result<OwnRequestData<Result<H::RequestData, H::ExtractRequestError>>, CoAPError> {
        if request.code().into() != coap_numbers::code::POST {
            return Err(CoAPError::method_not_allowed());
        }

        let (first_byte, edhoc_m1) = request
            .payload()
            .split_first()
            .ok_or_else(CoAPError::bad_request)?;
        let starts_with_true = first_byte == &0xf5;

        if starts_with_true {
            info!("Processing incoming EDHOC message 1");
            let message_1 =
                &lakers::EdhocMessageBuffer::new_from_slice(edhoc_m1).map_err(too_small)?;

            let (responder, c_i, ead_1) = lakers::EdhocResponder::new(
                (self.crypto_factory)(),
                lakers::EDHOCMethod::StatStat,
                *self.own_identity.1,
                *self.own_identity.0,
            )
            .process_message_1(message_1)
            .map_err(render_error)?;

            if ead_1.is_some_and(|e| e.is_critical) {
                // FIXME: send error message
                return Err(CoAPError::bad_request());
            }

            // Let's pick one now already: this allows us to use the identifier in our
            // request data.
            let c_r = COwn::not_in_iter(
                self.pool
                    .iter()
                    .filter_map(|entry| entry.corresponding_cown())
                    // C_R does not only need to be unique, it also must not be identical
                    // to C_I. If it is not expressible as a COwn (as_slice gives []),
                    // that's fine and we don't have to consider it.
                    .chain(COwn::from_kid(c_i.as_slice()).as_slice().iter().cloned()),
            );

            debug!("Entries in pool:");
            for (i, e) in self.pool.entries.iter().enumerate() {
                debug!("{}. {}", i, e);
            }
            debug!("Sequence:");
            for index in self.pool.sorted.iter() {
                debug!("* {}", index);
            }
            let evicted = self.pool.force_insert(SecContextState {
                protocol_stage: SecContextStage::EdhocResponderProcessedM1 {
                    c_r,
                    c_i,
                    responder,
                },
                authorization: self.unauthenticated_edhoc_user_authorization(),
            });
            if let Some(evicted) = evicted {
                warn!("To insert new EDHOC, evicted {}", evicted);
            } else {
                debug!("To insert new EDHOC, evicted none");
            }

            Ok(OwnRequestData::EdhocOkSend2(c_r))
        } else {
            // for the time being we'll only take the EDHOC option
            Err(CoAPError::bad_request())
        }
    }

    /// Process a CoAP request containing an OSCORE option and possibly an EDHOC option.
    fn extract_oscore_edhoc<M: ReadableMessage>(
        &mut self,
        request: &M,
        oscore_option: OscoreOption,
        with_edhoc: bool,
    ) -> Result<OwnRequestData<Result<H::RequestData, H::ExtractRequestError>>, CoAPError> {
        let payload = request.payload();

        // We know this to not fail b/c we only got here due to its presence
        let oscore_option = liboscore::OscoreOption::parse(&oscore_option)
            .map_err(|_| CoAPError::bad_option(coap_numbers::option::OSCORE))?;

        let kid = COwn::from_kid(
            oscore_option
                .kid()
                .ok_or(CoAPError::bad_option(coap_numbers::option::OSCORE))?,
        )
        // same as if it's not found in the pool
        .ok_or_else(CoAPError::bad_request)?;
        // If we don't make progress, we're dropping it altogether. Unless we use the
        // responder we might legally continue (because we didn't send data to EDHOC), but
        // once we've received something that (as we now know) looks like a message 3 and
        // isn't processable, it's unlikely that another one would come up and be.
        let mut taken = self
            .pool
            .lookup(|c| c.corresponding_cown() == Some(kid), core::mem::take)
            // following RFC8613 Section 8.2 item 2.2
            // FIXME unauthorized (unreleased in coap-message-utils)
            .ok_or_else(CoAPError::bad_request)?;

        let front_trim_payload = if with_edhoc {
            // We're not supporting block-wise here -- but could later, to the extent we support
            // outer block-wise.

            // Workaround for https://github.com/openwsn-berkeley/lakers/issues/255
            let mut decoder = minicbor::decode::Decoder::new(payload);
            let _ = decoder
                .decode::<&minicbor::bytes::ByteSlice>()
                .map_err(|_| CoAPError::bad_request())?;
            let cutoff = decoder.position();

            if let SecContextState {
                protocol_stage:
                    SecContextStage::EdhocResponderSentM2 {
                        responder,
                        c_r,
                        c_i,
                    },
                authorization: original_authorization, // So far, this is self.unauthenticated_edhoc_user_authorization()
            } = taken
            {
                debug_assert_eq!(c_r, kid, "State was looked up by KID");
                #[allow(clippy::indexing_slicing, reason = "slice fits by construction")]
                let msg_3 = lakers::EdhocMessageBuffer::new_from_slice(&payload[..cutoff])
                    .map_err(|e| too_small(e))?;

                let (responder, id_cred_i, ead_3) =
                    responder.parse_message_3(&msg_3).map_err(render_error)?;

                if ead_3.is_some_and(|e| e.is_critical) {
                    // FIXME: send error message
                    return Err(CoAPError::bad_request());
                }

                let cred_i;
                let authorization;

                if id_cred_i.reference_only() {
                    match id_cred_i.as_encoded_value() {
                        &[43] => {
                            info!("Peer indicates use of the one preconfigured key");

                            use hexlit::hex;
                            const CRED_I: &[u8] = &hex!("A2027734322D35302D33312D46462D45462D33372D33322D333908A101A5010202412B2001215820AC75E9ECE3E50BFC8ED60399889522405C47BF16DF96660A41298CB4307F7EB62258206E5DE611388A4B8A8211334AC7D37ECB52A387D257E6DB3C2A93DF21FF3AFFC8");

                            cred_i = lakers::Credential::parse_ccs(CRED_I)
                                .expect("Static credential is not processable");

                            // FIXME: learn from CRED_I
                            authorization = AifStaticRest {
                                may_use_stdout: true,
                            };
                        }
                        _ => {
                            // FIXME: send better message
                            return Err(CoAPError::bad_request());
                        }
                    }
                } else {
                    let ccs = id_cred_i
                        .get_ccs()
                        .expect("Lakers only knows IdCred as reference or as credential");
                    info!(
                        "Got credential CCS by value: {:?}..",
                        &ccs.bytes.get_slice(0, 5)
                    );

                    cred_i = lakers::Credential::parse_ccs(ccs.bytes.as_slice())
                        // FIXME What kind of error do we send here?
                        .map_err(|_| CoAPError::bad_request())?;

                    // FIXME: Do we want to continue at all? At least we don't allow
                    // stdout, but let's otherwise continue with the privileges of an
                    // unencrypted peer (allowing opportunistic encryption b/c we have
                    // enough slots to spare for some low-priority connections)
                    //
                    // The original_authorization may even have a hint (like, we might
                    // continue if it is not completely empty)
                    authorization = original_authorization;
                }

                let (mut responder, _prk_out) =
                    responder.verify_message_3(cred_i).map_err(render_error)?;

                let oscore_secret = responder.edhoc_exporter(0u8, &[], 16); // label is 0
                let oscore_salt = responder.edhoc_exporter(1u8, &[], 8); // label is 1
                let oscore_secret = &oscore_secret[..16];
                let oscore_salt = &oscore_salt[..8];
                #[allow(
                    clippy::indexing_slicing,
                    reason = "secret necessarily contains more than 40 bits"
                )]
                {
                    debug!("OSCORE secret: {:?}...", &oscore_secret[..5]);
                }
                debug!("OSCORE salt: {:?}", &oscore_salt);

                let sender_id = c_i.as_slice();
                let recipient_id = kid.0;

                // FIXME probe cipher suite
                let hkdf = liboscore::HkdfAlg::from_number(5).unwrap();
                let aead = liboscore::AeadAlg::from_number(10).unwrap();

                let immutables = liboscore::PrimitiveImmutables::derive(
                    hkdf,
                    oscore_secret,
                    oscore_salt,
                    None,
                    aead,
                    sender_id,
                    // FIXME need KID form (but for all that's supported that works still)
                    &[recipient_id],
                )
                // FIXME convert error
                .unwrap();

                let context = liboscore::PrimitiveContext::new_from_fresh_material(immutables);

                taken = SecContextState {
                    protocol_stage: SecContextStage::Oscore(context),
                    authorization,
                };
            } else {
                // Return the state. Best bet is that it was already advanced to an OSCORE
                // state, and the peer sent message 3 with multiple concurrent in-flight
                // messages. We're ignoring the EDHOC value and continue with OSCORE
                // processing.
            }

            info!(
                "Processed {} bytes at start of message into new EDHOC context",
                cutoff
            );

            cutoff
        } else {
            0
        };

        let SecContextState {
            protocol_stage: SecContextStage::Oscore(mut oscore_context),
            authorization,
        } = taken
        else {
            // FIXME: How'd we even get there?
            //
            // ... and return taken
            return Err(CoAPError::bad_request());
        };

        // Until liboscore can work on an arbitrary message, in particular a
        // `StrippingTheEdhocOptionAndPayloadPart<M>`, we have to create a copy.
        // (Conveniently, that also sidesteps the need to `downcast_from` to a type
        // libOSCORE knows, but that's not why we do it, that's what downcasting would be
        // for.)

        // embedded-nal-coap uses this max size, and our messages are same size or smaller,
        // so it's a guaranteed fit.
        const MAX_SIZE: usize = 1152;
        let mut read_copy = [0u8; MAX_SIZE];
        let mut code_copy = 0;
        let mut copied_message = coap_message_implementations::inmemory_write::Message::new(
            &mut code_copy,
            &mut read_copy[..],
        );
        // We could also do
        //     copied_message.set_from_message(request);
        // if we specified a "hiding EDHOC" message view.
        copied_message.set_code(request.code().into());
        // This may panic in theory on options being added in the wrong sequence; as we
        // don't downcast, we don't get the information on whether the underlying
        // implementation produces the options in the right sequence. Practically
        // (typically, and concretely in Ariel OS), it is given. (And it's not like we have
        // a fallback: inmemory_write has no more expensive option for reshuffling).
        for opt in request.options() {
            if opt.number() == coap_numbers::option::EDHOC {
                continue;
            }
            copied_message
                .add_option(opt.number(), opt.value())
                .unwrap();
        }
        #[allow(clippy::indexing_slicing, reason = "slice fits by construction")]
        copied_message
            .set_payload(&payload[front_trim_payload..])
            .unwrap();

        let decrypted = liboscore::unprotect_request(
            &mut copied_message,
            oscore_option,
            &mut oscore_context,
            |request| {
                if authorization.request_is_allowed(request) {
                    AuthorizationChecked::Allowed(self.inner.extract_request_data(request))
                } else {
                    AuthorizationChecked::NotAllowed
                }
            },
        );

        // With any luck, this never moves out.
        //
        // Storing it even on decryption failure to avoid DoS from the first message (but
        // FIXME, should we increment an error count and lower priority?)
        let evicted = self.pool.force_insert(SecContextState {
            protocol_stage: SecContextStage::Oscore(oscore_context),
            authorization,
        });
        debug_assert!(matches!(evicted, Some(SecContextState { protocol_stage: SecContextStage::Empty, .. }) | None), "A Default (Empty) was placed when an item was taken, which should have the lowest priority");

        let Ok((correlation, extracted)) = decrypted else {
            // FIXME is that the right code?
            error!("Decryption failure");
            return Err(CoAPError::unauthorized());
        };

        Ok(OwnRequestData::EdhocOscoreRequest {
            kid,
            correlation,
            extracted,
        })
    }
}

/// Wrapper around for a handler's inner RequestData
pub enum AuthorizationChecked<I> {
    /// Middleware checks were successful, data was extracted
    Allowed(I),
    /// Middleware checks failed, return an encrypted 4.03
    NotAllowed,
}

pub enum OwnRequestData<I> {
    // Taking a small state here: We already have a slot in the pool, storing the big data there
    #[expect(private_interfaces, reason = "should be addressed eventually")]
    EdhocOkSend2(COwn),
    // Could have a state Message3Processed -- but do we really want to implement that? (like, just
    // use the EDHOC option)
    EdhocOscoreRequest {
        #[expect(private_interfaces, reason = "should be addressed eventually")]
        kid: COwn,
        correlation: liboscore::raw::oscore_requestid_t,
        extracted: AuthorizationChecked<I>,
    },
    ProcessedToken,
}

// FIXME: It'd be tempting to implement Drop for Response to set the slot back to Empty -- but
// that'd be easier if we could avoid the Drop during enum destructuring, which AIU is currently
// not supported in match or let destructuring. (But our is_gc_eligible should be good enough
// anyway).

/// Render a MessageBufferError into the common Error type.
///
/// It is yet to be determined whether anything more informative should be returned (likely it
/// should; maybe Request Entity Too Large or some error code about unusable credential.
///
/// Places using this function may be simplified if From/Into is specified (possibly after
/// enlarging the Error type)
fn too_small(_e: lakers::MessageBufferError) -> CoAPError {
    CoAPError::bad_request()
}

/// Render an EDHOCError into the common Error type.
///
/// It is yet to be decided based on the EDHOC specification which EDHOCError values would be
/// reported with precise data, and which should rather produce a generic response.
///
/// Places using this function may be simplified if From/Into is specified (possibly after
/// enlarging the Error type)
fn render_error(_e: lakers::EDHOCError) -> CoAPError {
    CoAPError::bad_request()
}

#[derive(Debug)]
pub enum OrInner<O, I> {
    Own(O),
    Inner(I),
}

impl<O, I> From<O> for OrInner<O, I> {
    fn from(own: O) -> Self {
        OrInner::Own(own)
    }
}

impl<O: RenderableOnMinimal, I: RenderableOnMinimal> RenderableOnMinimal for OrInner<O, I> {
    type Error<IE> = OrInner<O::Error<IE>, I::Error<IE>> where IE: RenderableOnMinimal, IE: core::fmt::Debug;
    fn render<M: MinimalWritableMessage>(
        self,
        msg: &mut M,
    ) -> Result<(), Self::Error<M::UnionError>> {
        match self {
            OrInner::Own(own) => own.render(msg).map_err(OrInner::Own),
            OrInner::Inner(inner) => inner.render(msg).map_err(OrInner::Inner),
        }
    }
}

impl<'a, H: coap_handler::Handler, Crypto: lakers::Crypto, CryptoFactory: Fn() -> Crypto>
    coap_handler::Handler for OscoreEdhocHandler<'a, H, Crypto, CryptoFactory>
{
    type RequestData = OrInner<
        OwnRequestData<Result<H::RequestData, H::ExtractRequestError>>,
        AuthorizationChecked<H::RequestData>,
    >;

    type ExtractRequestError = OrInner<CoAPError, H::ExtractRequestError>;
    type BuildResponseError<M: MinimalWritableMessage> =
        OrInner<M::UnionError, H::BuildResponseError<M>>;

    fn extract_request_data<M: ReadableMessage>(
        &mut self,
        request: &M,
    ) -> Result<Self::RequestData, Self::ExtractRequestError> {
        use OrInner::{Inner, Own};

        #[derive(Default, Clone, Debug)]
        enum Recognition {
            #[default]
            Start,
            /// Seen an OSCORE option
            Oscore { oscore: OscoreOption },
            /// Seen an OSCORE option and an EDHOC option
            Edhoc { oscore: OscoreOption },
            /// Seen path ".well-known" (after not having seen an OSCORE option)
            WellKnown,
            /// Seen path ".well-known" and "edhoc"
            WellKnownEdhoc,
            /// Seen path "authz-info"
            // FIXME: Should we allow arbitrary paths here?
            AuthzInfo,
            /// Seen anything else (where the request handler, or more likely the ACL filter, will
            /// trip over the critical options)
            Unencrypted,
        }
        use Recognition::*;

        impl Recognition {
            /// Given a state and an option, produce the next state and whether the option should
            /// be counted as consumed for the purpose of assessing .well-known/edchoc's
            /// ignore_elective_others().
            fn update(self, o: &impl MessageOption) -> (Self, bool) {
                use coap_numbers::option;

                match (self, o.number(), o.value()) {
                    // FIXME: Store full value (but a single one is sufficient while we do EDHOC
                    // extraction)
                    (Start, option::OSCORE, optval) => match optval.try_into() {
                        Ok(oscore) => (Oscore { oscore }, false),
                        _ => (Start, true),
                    },
                    (Start, option::URI_PATH, b".well-known") => (WellKnown, false),
                    (Start, option::URI_PATH, b"authz-info") => (AuthzInfo, false),
                    (Start, option::URI_PATH, _) => (Unencrypted, true /* doesn't matter */),
                    (Oscore { oscore }, option::EDHOC, b"") => {
                        (Edhoc { oscore }, true /* doesn't matter */)
                    }
                    (WellKnown, option::URI_PATH, b"edhoc") => (WellKnownEdhoc, false),
                    (AuthzInfo, option::CONTENT_FORMAT, &[19]) => (AuthzInfo, false),
                    (AuthzInfo, option::ACCEPT, &[19]) => (AuthzInfo, false),
                    (any, _, _) => (any, true),
                }
            }
        }

        let mut state = Recognition::Start;

        // Some small potential for optimization by cutting iteration short on Edhoc, but probably
        // not worth it.
        let extra_options = request
            .options()
            .filter(|o| {
                let (new_state, filter) = state.clone().update(o);
                state = new_state;
                filter
            })
            // FIXME: This aborts early on critical options, even when the result is later ignored
            .ignore_elective_others();

        if let (Err(error), WellKnownEdhoc | AuthzInfo) = (extra_options, &state) {
            // Critical options in all other cases are handled by the Unencrypted or Oscore
            // handlers
            return Err(Own(error));
        }

        match state {
            Start | WellKnown | Unencrypted => {
                if self.nosec_authorization().request_is_allowed(request) {
                    self.inner
                        .extract_request_data(request)
                        .map(|extracted| Inner(AuthorizationChecked::Allowed(extracted)))
                        .map_err(Inner)
                } else {
                    Ok(Inner(AuthorizationChecked::NotAllowed))
                }
            }
            WellKnownEdhoc => self.extract_edhoc(&request).map(Own).map_err(Own),
            AuthzInfo => Ok(Own(OwnRequestData::ProcessedToken)),
            Edhoc { oscore } => self
                .extract_oscore_edhoc(&request, oscore, true)
                .map(Own)
                .map_err(Own),
            Oscore { oscore } => self
                .extract_oscore_edhoc(&request, oscore, false)
                .map(Own)
                .map_err(Own),
        }
    }
    fn estimate_length(&mut self, req: &Self::RequestData) -> usize {
        match req {
            OrInner::Own(_) => 2 + lakers::MAX_BUFFER_LEN,
            OrInner::Inner(AuthorizationChecked::Allowed(i)) => self.inner.estimate_length(i),
            OrInner::Inner(AuthorizationChecked::NotAllowed) => 1,
        }
    }
    fn build_response<M: MutableWritableMessage>(
        &mut self,
        response: &mut M,
        req: Self::RequestData,
    ) -> Result<(), Self::BuildResponseError<M>> {
        use OrInner::{Inner, Own};

        match req {
            Own(OwnRequestData::EdhocOkSend2(c_r)) => {
                // FIXME: Why does the From<O> not do the map_err?
                response.set_code(
                    M::Code::new(coap_numbers::code::CHANGED).map_err(|x| Own(x.into()))?,
                );

                let message_2 = self.pool.lookup(
                    |c| c.corresponding_cown() == Some(c_r),
                    |matched| {
                        // temporary default will not live long (and may be only constructed if
                        // prepare_message_2 fails)
                        let taken = core::mem::take(matched);
                        let SecContextState {
                            protocol_stage:
                                SecContextStage::EdhocResponderProcessedM1 {
                                    c_r: matched_c_r,
                                    c_i,
                                    responder: taken,
                                },
                            authorization,
                        } = taken
                        else {
                            todo!();
                        };
                        debug_assert_eq!(
                            matched_c_r, c_r,
                            "The first lookup function ensured this property"
                        );
                        let (responder, message_2) = taken
                            // We're sending our ID by reference: we have a CCS and don't expect anyone to
                            // run EDHOC with us who can not verify who we are (and from the CCS there is
                            // no better way). Also, conveniently, this covers our privacy well.
                            // (Sending ByValue would still work)
                            .prepare_message_2(
                                lakers::CredentialTransfer::ByReference,
                                Some(c_r.into()),
                                &None,
                            )
                            // FIXME error handling
                            .unwrap();
                        *matched = SecContextState {
                            protocol_stage: SecContextStage::EdhocResponderSentM2 {
                                responder,
                                c_i,
                                c_r,
                            },
                            authorization,
                        };
                        message_2
                    },
                );

                let Some(message_2) = message_2 else {
                    // FIXME render late error (it'd help if CoAPError also offered a type that unions it
                    // with an arbitrary other error). As it is, depending on the CoAP stack, there may be
                    // DoS if a peer can send many requests before the server starts rendering responses.
                    panic!("State vanished before response was built.");
                };

                response
                    .set_payload(message_2.as_slice())
                    .map_err(|x| Own(x.into()))?;
            }
            Own(OwnRequestData::ProcessedToken) => {
                response.set_code(M::Code::new(coap_numbers::code::INTERNAL_SERVER_ERROR).unwrap());
            }
            Own(OwnRequestData::EdhocOscoreRequest {
                kid,
                mut correlation,
                extracted,
            }) => {
                response.set_code(
                    M::Code::new(coap_numbers::code::CHANGED).map_err(|x| Own(x.into()))?,
                );

                self.pool
                    .lookup(|c| c.corresponding_cown() == Some(kid), |matched| {
                        // Not checking authorization any more: we don't even have access to the
                        // request any more, that check was done.
                        let SecContextState { protocol_stage: SecContextStage::Oscore(ref mut oscore_context), .. } = matched else {
                            // FIXME render late error (it'd help if CoAPError also offered a type that unions it
                            // with an arbitrary other error). As it is, depending on the CoAP stack, there may be
                            // DoS if a peer can send many requests before the server starts rendering responses.
                            panic!("State vanished before response was built.");
                        };

                        let response = coap_message_implementations::inmemory_write::Message::downcast_from(response)
                            .expect("OSCORE handler currently requires a response message implementation that is of fixed type");

                        response.set_code(coap_numbers::code::CHANGED);

                        if liboscore::protect_response(
                            response,
                            // SECURITY BIG FIXME: How do we make sure that our correlation is really for
                            // what we find in the pool and not for what wound up there by the time we send
                            // the response? (Can't happen with the current stack, but conceptually there
                            // should be a tie; carry the OSCORE context in an owned way?).
                            oscore_context,
                            &mut correlation,
                            |response| match extracted {
                                AuthorizationChecked::Allowed(Ok(extracted)) => match self.inner.build_response(response, extracted) {
                                    Ok(()) => {
                                        // All fine, response was built
                                    },
                                    // One attempt to render rendering errors
                                    // FIXME rewind message
                                    Err(e) => {
                                        error!("Rendering successful extraction failed with {:?}", Debug2Format(&e));
                                        match e.render(response) {
                                            Ok(()) => {
                                                error!("Error rendered.");
                                            },
                                            Err(e2) => {
                                                error!("Error could not be rendered: {:?}.", Debug2Format(&e2));
                                                // FIXME rewind message
                                                response.set_code(coap_numbers::code::INTERNAL_SERVER_ERROR);
                                            }
                                        };
                                    },
                                },
                                AuthorizationChecked::Allowed(Err(inner_request_error)) => {
                                    error!("Extraction failed with {:?}.", Debug2Format(&inner_request_error));
                                    match inner_request_error.render(response) {
                                        Ok(()) => {
                                            error!("Original error rendered successfully.");
                                        },
                                        Err(e) => {
                                            error!("Original error could not be rendered due to {:?}:", Debug2Format(&e));
                                            // Two attempts to render extraction errors
                                            // FIXME rewind message
                                            match e.render(response) {
                                                Ok(()) => {
                                                    error!("Error was rendered fine.");
                                                },
                                                Err(e2) => {
                                                    error!("Rendering error caused {:?}.", Debug2Format(&e2));
                                                    // FIXME rewind message
                                                    response.set_code(
                                                        coap_numbers::code::INTERNAL_SERVER_ERROR,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                AuthorizationChecked::NotAllowed => {
                                    response.set_code(
                                        coap_numbers::code::UNAUTHORIZED,
                                    );
                                }
                            },
                        )
                        .is_err()
                        {
                            error!("Oups, responding with weird state");
                            // todo!("Thanks to the protect API we've lost access to our response");
                        }
                    });
            }
            Inner(AuthorizationChecked::Allowed(i)) => {
                self.inner.build_response(response, i).map_err(Inner)?
            }
            Inner(AuthorizationChecked::NotAllowed) => {
                response.set_code(
                    M::Code::new(coap_numbers::code::UNAUTHORIZED).map_err(|x| Own(x.into()))?,
                );
            }
        };
        Ok(())
    }
}
