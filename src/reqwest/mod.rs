//! Simple document and context loader based on [`reqwest`](https://crates.io/crates/reqwest)

use crate::{
    context::{self, RemoteContext},
    Error, ErrorCode, RemoteDocument,
};
use futures::future::{BoxFuture, FutureExt};
use iref::{Iri, IriBuf};
use json::JsonValue;
use std::collections::HashMap;

pub fn is_json_media_type(ty: &str) -> bool {
    ty == "application/json" || ty == "application/ld+json"
}

pub async fn load_remote_json_ld_document(url: Iri<'_>) -> Result<RemoteDocument, Error> {
    info!("loading remote document `{}'", url);
    use reqwest::header::*;

    let client = reqwest::Client::new();
    let request = client
        .get(url.as_str())
        .header(ACCEPT, "application/ld+json, application/json");
    let response = request.send().await?;

    if response
        .headers()
        .get_all(CONTENT_TYPE)
        .iter()
        .find(|&value| {
            if let Ok(value) = value.to_str() {
                is_json_media_type(value)
            } else {
                false
            }
        })
        .is_some()
    {
        let body = response.text().await?;

        match json::parse(body.as_str()) {
            Ok(doc) => Ok(RemoteDocument::new(doc, url.into())),
            Err(e) => panic!("invalid json: {:?}: {}", e, body.as_str()),
        }
    } else {
        panic!("not a json document")
    }
}

pub struct Loader {
    cache: HashMap<IriBuf, RemoteDocument>,
}

impl Loader {
    pub fn new() -> Loader {
        Loader {
            cache: HashMap::new(),
        }
    }

    pub async fn load(&mut self, url: Iri<'_>) -> Result<RemoteDocument, Error> {
        let url = IriBuf::from(url);
        match self.cache.get(&url) {
            Some(doc) => Ok(doc.clone()),
            None => {
                let doc = load_remote_json_ld_document(url.as_iri()).await?;
                self.cache.insert(url, doc.clone());
                Ok(doc)
            }
        }
    }
}

impl context::Loader for Loader {
    type Output = JsonValue;

    fn load_context<'a>(
        &'a mut self,
        url: Iri,
    ) -> BoxFuture<'a, Result<RemoteContext<JsonValue>, Error>> {
        let url = IriBuf::from(url);
        async move {
            match self.load(url.as_iri()).await {
                Ok(remote_doc) => {
                    let (doc, url) = remote_doc.into_parts();
                    if let JsonValue::Object(obj) = doc {
                        if let Some(context) = obj.get("@context") {
                            Ok(RemoteContext::from_parts(url, context.clone()))
                        } else {
                            Err(ErrorCode::InvalidRemoteContext.into())
                        }
                    } else {
                        Err(ErrorCode::InvalidRemoteContext.into())
                    }
                }
                Err(_) => Err(ErrorCode::LoadingRemoteContextFailed.into()),
            }
        }
        .boxed()
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error::new(ErrorCode::LoadingDocumentFailed, e)
    }
}
