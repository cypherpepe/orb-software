//! P2P Blob API

mod bootstrap;
mod hash;
mod tag;

pub use crate::bootstrap::Bootstrapper;
pub use crate::hash::{Hash, HashTopic};
pub use crate::tag::{Tag, TagTopic};

use async_stream::stream;
use eyre::{Context, Result};
use futures::StreamExt;
use iroh::NodeId;
use iroh_gossip::api::{ApiError, GossipApi};
use iroh_gossip::proto::TopicId;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

// Used to disambiguate from other contexts/topics.
const HASH_CTX: &str = "orb-blob-v0";

/// Topic for a blob, addressible either by hash or by tag.
#[derive(Debug, Eq, PartialEq, Hash, derive_more::From)]
pub enum BlobTopic {
    Hash(HashTopic),
    Tag(TagTopic),
}

impl BlobTopic {
    pub(crate) fn to_id(&self) -> TopicId {
        match self {
            BlobTopic::Hash(hash_topic) => hash_topic.to_id(),
            BlobTopic::Tag(tag_topic) => tag_topic.to_id(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum GossipMsg {
    Tag(crate::tag::TagGossipMsg),
    Hash(crate::hash::HashGossipMsg),
}

#[derive(Debug, bon::Builder, Clone)]
pub struct Client {
    gossip: GossipApi,
    bootstrap_nodes: Vec<NodeId>,
}

impl Client {
    pub async fn listen_for_peers(
        &self,
        topic: impl Into<BlobTopic>,
    ) -> Result<impl futures::Stream<Item = NodeId> + Send + 'static> {
        let blob_topic: BlobTopic = topic.into();
        let topic_id = blob_topic.to_id();
        let mut topic = self
            .gossip
            .subscribe_and_join(topic_id, self.bootstrap_nodes.clone())
            .await
            .wrap_err("failed to subscribe")?;

        Ok(stream! {
            while let Some(result) = topic.next().await {
                let event = match result {
                    Err(ApiError::Closed { .. }) => break,
                    Ok(e) => e,
                    Err(err) => {
                        error!("error while listening to gossip topic: {err}");
                        break;
                    }
                };
                let iroh_gossip::api::Event::Received(msg) = event else {
                    continue;
                };

                let deserialized: Result<GossipMsg, _> =
                    serde_json::from_slice(msg.content.as_ref());
                let gossip_msg = match deserialized {
                    Err(err) => {
                        warn!("peer had invalid message: {err}");
                        continue;
                    }
                    Ok(deserialized) => deserialized,
                };

                let hash_gossip_msg = match gossip_msg {
                    GossipMsg::Tag(_) => todo!("we will implement tags later"),
                    GossipMsg::Hash(m) => m,
                };
                yield hash_gossip_msg.node_id;
            }
        })
    }
}
