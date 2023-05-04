use crate::*;
use futures::stream::TryStreamExt;
use ipfs_api::TryFromUri;
use ipfs_api::{IpfsApi, IpfsClient};
use serde::{Deserialize, Serialize};
use serde_json;
use std::default::Default;
use std::io::Cursor;
use std::result::Result;
use tokio::runtime::Handle;

#[derive(Clone)]
pub struct IPFSManager {
    client: IpfsClient,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Err {
    IpfsNetworkError,
    IpfsParseError,
}

impl IPFSManager {
    pub fn new() -> Self {
        Self {
            client: IpfsClient::from_str("https://ipfs.infura.io:5001")
                .unwrap()
                .with_credentials(
                    "2OuGHppwQzKcS9HErVCZ12ngIIr",
                    "cbfd29f500aee37e0b4139dabdbe396a",
                ),
        }
    }

    pub async fn get_object<T>(&self, cid: String) -> Result<T, Err>
    where
        T: for<'a> Deserialize<'a> + Default,
    {
        let handle = Handle::current();

        let client = self.client.clone();
        let res = handle
            .spawn_blocking(move || {
                let handle = Handle::current();
                handle.block_on(client.cat(&cid).map_ok(|chunk| chunk.to_vec()).try_concat())
            })
            .await
            .map_err(|_e| Err::IpfsNetworkError)?
            .unwrap();
        serde_json::from_slice(&res).map_err(|_| Err::IpfsParseError)
    }

    pub async fn set_object<T>(&self, my_object: T) -> Result<String, Err>
    where
        T: Serialize + Default,
    {
        let content = serde_json::to_string(&my_object).map_err(|_| Err::IpfsParseError)?;
        let content = content.as_bytes().to_vec();
        let cursor = Cursor::new(content);

        let add_result = self
            .client
            .add(cursor)
            .await
            .map_err(|_| Err::IpfsNetworkError)?;
        Ok(add_result.hash.to_string())
    }
}
