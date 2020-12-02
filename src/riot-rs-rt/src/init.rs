use linkme::distributed_slice;

#[distributed_slice]
pub static INIT_FUNCS: [fn()] = [..];
