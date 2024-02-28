module Riot_rs_runqueue.Runqueue
#set-options "--fuel 0 --ifuel 1 --z3rlimit 15"
open Core
open FStar.Mul

unfold
let t_RunqueueId = u8

unfold
let t_ThreadId = u8

let v_USIZE_BITS: usize = (Core.Mem.size_of () <: usize) *! sz 8

let impl__ffs (v_N_QUEUES v_N_THREADS v_val: usize) : u32 =
  (cast (v_USIZE_BITS <: usize) <: u32) -! (Core.Num.impl__usize__leading_zeros v_val <: u32)

type t_RunQueue (v_N_QUEUES: usize) (v_N_THREADS: usize) = {
  f_bitcache:usize;
  f_queues:Riot_rs_runqueue.Runqueue.Clist.t_CList v_N_QUEUES v_N_THREADS
}

let impl__add (v_N_QUEUES v_N_THREADS: usize) (self: t_RunQueue v_N_QUEUES v_N_THREADS) (n rq: u8)
    : t_RunQueue v_N_QUEUES v_N_THREADS =
  let _:Prims.unit =
    if true
    then
      let _:Prims.unit =
        if ~.((cast (n <: u8) <: usize) <. v_N_THREADS <: bool)
        then
          Rust_primitives.Hax.never_to_any (Core.Panicking.panic "assertion failed: (n as usize) < N_THREADS"

              <:
              Rust_primitives.Hax.t_Never)
      in
      ()
  in
  let _:Prims.unit =
    if true
    then
      let _:Prims.unit =
        if ~.((cast (rq <: u8) <: usize) <. v_N_QUEUES <: bool)
        then
          Rust_primitives.Hax.never_to_any (Core.Panicking.panic "assertion failed: (rq as usize) < N_QUEUES"

              <:
              Rust_primitives.Hax.t_Never)
      in
      ()
  in
  let self:t_RunQueue v_N_QUEUES v_N_THREADS =
    { self with f_bitcache = self.f_bitcache |. (sz 1 <<! rq <: usize) }
    <:
    t_RunQueue v_N_QUEUES v_N_THREADS
  in
  let self:t_RunQueue v_N_QUEUES v_N_THREADS =
    {
      self with
      f_queues
      =
      Riot_rs_runqueue.Runqueue.Clist.impl__push v_N_QUEUES v_N_THREADS self.f_queues n rq
    }
    <:
    t_RunQueue v_N_QUEUES v_N_THREADS
  in
  self

let impl__advance (v_N_QUEUES v_N_THREADS: usize) (self: t_RunQueue v_N_QUEUES v_N_THREADS) (rq: u8)
    : t_RunQueue v_N_QUEUES v_N_THREADS =
  let _:Prims.unit =
    if true
    then
      let _:Prims.unit =
        if ~.((cast (rq <: u8) <: usize) <. v_N_QUEUES <: bool)
        then
          Rust_primitives.Hax.never_to_any (Core.Panicking.panic "assertion failed: (rq as usize) < N_QUEUES"

              <:
              Rust_primitives.Hax.t_Never)
      in
      ()
  in
  let hax_temp_output, self:(Prims.unit & t_RunQueue v_N_QUEUES v_N_THREADS) =
    (),
    ({
        self with
        f_queues
        =
        Riot_rs_runqueue.Runqueue.Clist.impl__advance v_N_QUEUES v_N_THREADS self.f_queues rq
      }
      <:
      t_RunQueue v_N_QUEUES v_N_THREADS)
    <:
    (Prims.unit & t_RunQueue v_N_QUEUES v_N_THREADS)
  in
  self

let impl__del (v_N_QUEUES v_N_THREADS: usize) (self: t_RunQueue v_N_QUEUES v_N_THREADS) (n rq: u8)
    : t_RunQueue v_N_QUEUES v_N_THREADS =
  let _:Prims.unit =
    if true
    then
      let _:Prims.unit =
        if ~.((cast (n <: u8) <: usize) <. v_N_THREADS <: bool)
        then
          Rust_primitives.Hax.never_to_any (Core.Panicking.panic "assertion failed: (n as usize) < N_THREADS"

              <:
              Rust_primitives.Hax.t_Never)
      in
      ()
  in
  let _:Prims.unit =
    if true
    then
      let _:Prims.unit =
        if ~.((cast (rq <: u8) <: usize) <. v_N_QUEUES <: bool)
        then
          Rust_primitives.Hax.never_to_any (Core.Panicking.panic "assertion failed: (rq as usize) < N_QUEUES"

              <:
              Rust_primitives.Hax.t_Never)
      in
      ()
  in
  let tmp0, out:(Riot_rs_runqueue.Runqueue.Clist.t_CList v_N_QUEUES v_N_THREADS &
    Core.Option.t_Option u8) =
    Riot_rs_runqueue.Runqueue.Clist.impl__pop_head v_N_QUEUES v_N_THREADS self.f_queues rq
  in
  let self:t_RunQueue v_N_QUEUES v_N_THREADS =
    { self with f_queues = tmp0 } <: t_RunQueue v_N_QUEUES v_N_THREADS
  in
  let popped:Core.Option.t_Option u8 = out in
  let _:Prims.unit =
    match
      popped, (Core.Option.Option_Some n <: Core.Option.t_Option u8)
      <:
      (Core.Option.t_Option u8 & Core.Option.t_Option u8)
    with
    | left_val, right_val ->
      if ~.(left_val =. right_val <: bool)
      then
        let kind:Core.Panicking.t_AssertKind =
          Core.Panicking.AssertKind_Eq <: Core.Panicking.t_AssertKind
        in
        Rust_primitives.Hax.never_to_any (Core.Panicking.assert_failed kind
              left_val
              right_val
              (Core.Option.Option_None <: Core.Option.t_Option Core.Fmt.t_Arguments)
            <:
            Rust_primitives.Hax.t_Never)
  in
  let self, hax_temp_output:(t_RunQueue v_N_QUEUES v_N_THREADS & Prims.unit) =
    if Riot_rs_runqueue.Runqueue.Clist.impl__is_empty v_N_QUEUES v_N_THREADS self.f_queues rq
    then
      let self:t_RunQueue v_N_QUEUES v_N_THREADS =
        { self with f_bitcache = self.f_bitcache &. (~.(sz 1 <<! rq <: usize) <: usize) }
        <:
        t_RunQueue v_N_QUEUES v_N_THREADS
      in
      self, () <: (t_RunQueue v_N_QUEUES v_N_THREADS & Prims.unit)
    else self, () <: (t_RunQueue v_N_QUEUES v_N_THREADS & Prims.unit)
  in
  self

let impl__get_next (v_N_QUEUES v_N_THREADS: usize) (self: t_RunQueue v_N_QUEUES v_N_THREADS)
    : Core.Option.t_Option u8 =
  let rq_ffs:u32 = impl__ffs v_N_QUEUES v_N_THREADS self.f_bitcache in
  if rq_ffs >. 0ul
  then
    let rq:u8 = cast (rq_ffs -! 1ul <: u32) <: u8 in
    Riot_rs_runqueue.Runqueue.Clist.impl__peek_head v_N_QUEUES v_N_THREADS self.f_queues rq
  else Core.Option.Option_None <: Core.Option.t_Option u8

let impl__new (v_N_QUEUES: usize) (v_N_THREADS: usize) (_: Prims.unit)
    : t_RunQueue v_N_QUEUES v_N_THREADS =
  {
    f_bitcache = sz 0;
    f_queues = Riot_rs_runqueue.Runqueue.Clist.impl__new v_N_QUEUES v_N_THREADS ()
  }
  <:
  t_RunQueue v_N_QUEUES v_N_THREADS
