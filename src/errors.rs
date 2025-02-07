// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use crate::model::write::WriteResponse;

#[derive(Debug)]
pub enum Error {
    /// Error from the running server.
    Server(ServerError),

    /// Error from the rpc.
    /// Note that any error caused by a running server wont be wrapped in the
    /// grpc errors.
    Rpc(tonic::Status),

    /// Error about rpc.
    /// It will be throw while connection between client and server is broken
    /// and try for reconnecting is failed(timeout).
    Connect {
        addr: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Error from the client and basically the rpc request has not been called
    /// yet or the rpc request has already been finished successfully.
    Client(String),

    /// Error about authentication
    AuthFail(AuthFailStatus),

    ///
    ClusterWriteError(ClusterWriteError),

    /// Error unknown
    Unknown(String),
}

#[derive(Debug)]
pub struct ClusterWriteError {
    pub ok: (Vec<String>, WriteResponse), // (metrics, write_response)
    pub errors: Vec<(Vec<String>, Error)>, // [(metrics, erros)]
}

impl From<Vec<(Vec<String>, Result<WriteResponse>)>> for ClusterWriteError {
    fn from(wirte_results: Vec<(Vec<String>, Result<WriteResponse>)>) -> Self {
        let mut success_total = 0;
        let mut failed_total = 0;
        let mut ok_metrics = Vec::new();
        let mut errors = Vec::new();
        for (metrics, write_result) in wirte_results {
            match write_result {
                Ok(write_resp) => {
                    success_total += write_resp.success;
                    failed_total += write_resp.failed;
                    ok_metrics.extend(metrics);
                }
                Err(e) => {
                    errors.push((metrics, e));
                }
            }
        }

        Self {
            ok: (ok_metrics, WriteResponse::new(success_total, failed_total)),
            errors,
        }
    }
}

impl ClusterWriteError {
    pub fn all_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ServerError {
    pub code: u32,
    pub msg: String,
}

#[derive(Debug, Clone)]
pub struct AuthFailStatus {
    pub code: AuthCode,
    pub msg: String,
}

#[derive(Debug, Clone)]
pub enum AuthCode {
    Ok = 0,

    InvalidTenantMeta = 1,

    InvalidTokenMeta = 2,
}

pub type Result<T> = std::result::Result<T, Error>;
