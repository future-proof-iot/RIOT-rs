module Riot_rs_runqueue.Runqueue.Clist
#set-options "--fuel 0 --ifuel 1 --z3rlimit 15"
open Core
open FStar.Mul

let impl__sentinel (v_N_QUEUES: usize) (v_N_THREADS: usize) (_: Prims.unit) : u8 = 255uy

type t_CList (v_N_QUEUES: usize) (v_N_THREADS: usize) = {
  f_tail:t_Array u8 v_N_QUEUES;
  f_next_idxs:t_Array u8 v_N_THREADS
}

let impl__advance (v_N_QUEUES v_N_THREADS: usize) (self: t_CList v_N_QUEUES v_N_THREADS) (rq: u8)
    : t_CList v_N_QUEUES v_N_THREADS =
  let self, hax_temp_output:(t_CList v_N_QUEUES v_N_THREADS & Prims.unit) =
    if
      (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8) <>.
      (impl__sentinel v_N_QUEUES v_N_THREADS () <: u8)
    then
      let self:t_CList v_N_QUEUES v_N_THREADS =
        {
          self with
          f_tail
          =
          Rust_primitives.Hax.Monomorphized_update_at.update_at_usize self.f_tail
            (cast (rq <: u8) <: usize)
            (self.f_next_idxs.[ cast (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8) <: usize ]
              <:
              u8)
        }
        <:
        t_CList v_N_QUEUES v_N_THREADS
      in
      self, () <: (t_CList v_N_QUEUES v_N_THREADS & Prims.unit)
    else self, () <: (t_CList v_N_QUEUES v_N_THREADS & Prims.unit)
  in
  self

let impl__is_empty (v_N_QUEUES v_N_THREADS: usize) (self: t_CList v_N_QUEUES v_N_THREADS) (rq: u8)
    : bool =
  (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8) =.
  (impl__sentinel v_N_QUEUES v_N_THREADS () <: u8)

let impl__new (v_N_QUEUES: usize) (v_N_THREADS: usize) (_: Prims.unit)
    : t_CList v_N_QUEUES v_N_THREADS =
  {
    f_tail = Rust_primitives.Hax.repeat (impl__sentinel v_N_QUEUES v_N_THREADS () <: u8) v_N_QUEUES;
    f_next_idxs
    =
    Rust_primitives.Hax.repeat (impl__sentinel v_N_QUEUES v_N_THREADS () <: u8) v_N_THREADS
  }
  <:
  t_CList v_N_QUEUES v_N_THREADS

let impl__peek_head (v_N_QUEUES v_N_THREADS: usize) (self: t_CList v_N_QUEUES v_N_THREADS) (rq: u8)
    : Core.Option.t_Option u8 =
  if
    (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8) =.
    (impl__sentinel v_N_QUEUES v_N_THREADS () <: u8)
  then Core.Option.Option_None <: Core.Option.t_Option u8
  else
    Core.Option.Option_Some
    self.f_next_idxs.[ cast (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8) <: usize ]
    <:
    Core.Option.t_Option u8

let impl__pop_head (v_N_QUEUES v_N_THREADS: usize) (self: t_CList v_N_QUEUES v_N_THREADS) (rq: u8)
    : (t_CList v_N_QUEUES v_N_THREADS & Core.Option.t_Option u8) =
  let self, hax_temp_output:(t_CList v_N_QUEUES v_N_THREADS & Core.Option.t_Option u8) =
    if
      (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8) =.
      (impl__sentinel v_N_QUEUES v_N_THREADS () <: u8)
    then
      self, (Core.Option.Option_None <: Core.Option.t_Option u8)
      <:
      (t_CList v_N_QUEUES v_N_THREADS & Core.Option.t_Option u8)
    else
      let head:u8 =
        self.f_next_idxs.[ cast (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8) <: usize ]
      in
      let self:t_CList v_N_QUEUES v_N_THREADS =
        if head =. (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8)
        then
          let self:t_CList v_N_QUEUES v_N_THREADS =
            {
              self with
              f_tail
              =
              Rust_primitives.Hax.Monomorphized_update_at.update_at_usize self.f_tail
                (cast (rq <: u8) <: usize)
                (impl__sentinel v_N_QUEUES v_N_THREADS () <: u8)
            }
            <:
            t_CList v_N_QUEUES v_N_THREADS
          in
          self
        else
          let self:t_CList v_N_QUEUES v_N_THREADS =
            {
              self with
              f_next_idxs
              =
              Rust_primitives.Hax.Monomorphized_update_at.update_at_usize self.f_next_idxs
                (cast (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8) <: usize)
                (self.f_next_idxs.[ cast (head <: u8) <: usize ] <: u8)
            }
            <:
            t_CList v_N_QUEUES v_N_THREADS
          in
          self
      in
      let self:t_CList v_N_QUEUES v_N_THREADS =
        {
          self with
          f_next_idxs
          =
          Rust_primitives.Hax.Monomorphized_update_at.update_at_usize self.f_next_idxs
            (cast (head <: u8) <: usize)
            (impl__sentinel v_N_QUEUES v_N_THREADS () <: u8)
        }
        <:
        t_CList v_N_QUEUES v_N_THREADS
      in
      self, (Core.Option.Option_Some head <: Core.Option.t_Option u8)
      <:
      (t_CList v_N_QUEUES v_N_THREADS & Core.Option.t_Option u8)
  in
  self, hax_temp_output <: (t_CList v_N_QUEUES v_N_THREADS & Core.Option.t_Option u8)

let impl__push (v_N_QUEUES v_N_THREADS: usize) (self: t_CList v_N_QUEUES v_N_THREADS) (n rq: u8)
    : t_CList v_N_QUEUES v_N_THREADS =
  let _:Prims.unit =
    if ~.(n <. (impl__sentinel v_N_QUEUES v_N_THREADS () <: u8) <: bool)
    then
      Rust_primitives.Hax.never_to_any (Core.Panicking.panic "assertion failed: n < Self::sentinel()"

          <:
          Rust_primitives.Hax.t_Never)
  in
  let self, hax_temp_output:(t_CList v_N_QUEUES v_N_THREADS & Prims.unit) =
    if
      (self.f_next_idxs.[ cast (n <: u8) <: usize ] <: u8) =.
      (impl__sentinel v_N_QUEUES v_N_THREADS () <: u8)
    then
      if
        (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8) =.
        (impl__sentinel v_N_QUEUES v_N_THREADS () <: u8)
      then
        let self:t_CList v_N_QUEUES v_N_THREADS =
          {
            self with
            f_tail
            =
            Rust_primitives.Hax.Monomorphized_update_at.update_at_usize self.f_tail
              (cast (rq <: u8) <: usize)
              n
          }
          <:
          t_CList v_N_QUEUES v_N_THREADS
        in
        let self:t_CList v_N_QUEUES v_N_THREADS =
          {
            self with
            f_next_idxs
            =
            Rust_primitives.Hax.Monomorphized_update_at.update_at_usize self.f_next_idxs
              (cast (n <: u8) <: usize)
              n
          }
          <:
          t_CList v_N_QUEUES v_N_THREADS
        in
        self, () <: (t_CList v_N_QUEUES v_N_THREADS & Prims.unit)
      else
        let self:t_CList v_N_QUEUES v_N_THREADS =
          {
            self with
            f_next_idxs
            =
            Rust_primitives.Hax.Monomorphized_update_at.update_at_usize self.f_next_idxs
              (cast (n <: u8) <: usize)
              (self.f_next_idxs.[ cast (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8) <: usize ]
                <:
                u8)
          }
          <:
          t_CList v_N_QUEUES v_N_THREADS
        in
        let self:t_CList v_N_QUEUES v_N_THREADS =
          {
            self with
            f_next_idxs
            =
            Rust_primitives.Hax.Monomorphized_update_at.update_at_usize self.f_next_idxs
              (cast (self.f_tail.[ cast (rq <: u8) <: usize ] <: u8) <: usize)
              n
          }
          <:
          t_CList v_N_QUEUES v_N_THREADS
        in
        let self:t_CList v_N_QUEUES v_N_THREADS =
          {
            self with
            f_tail
            =
            Rust_primitives.Hax.Monomorphized_update_at.update_at_usize self.f_tail
              (cast (rq <: u8) <: usize)
              n
          }
          <:
          t_CList v_N_QUEUES v_N_THREADS
        in
        self, () <: (t_CList v_N_QUEUES v_N_THREADS & Prims.unit)
    else self, () <: (t_CList v_N_QUEUES v_N_THREADS & Prims.unit)
  in
  self
