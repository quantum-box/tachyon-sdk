#![allow(unused_imports)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::empty_docs)]
#![allow(clippy::into_iter_on_ref)]
#![allow(clippy::needless_return)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::uninlined_format_args)]

pub extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate serde_repr;
extern crate url;

pub mod apis;
pub mod models;

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "platform-traits")]
pub mod payment;

#[cfg(feature = "platform-traits")]
pub mod procurement;

#[cfg(feature = "platform-traits")]
pub mod secrets;

#[cfg(feature = "worker-runtime")]
pub mod worker_runtime;
