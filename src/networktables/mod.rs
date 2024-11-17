pub mod error;

use crate::prelude::*;
use std::time::Duration;

use error::*;
use futures::future::BoxFuture;
use nt_client::{
    data::{r#type::RawData, Properties, SubscriptionOptions},
    publish::Publisher,
    subscribe::ReceivedMessage,
    Client,
};
use tokio::runtime;

pub trait ThreadSafe = Send + Sync + 'static;

pub async fn worker<'a, C0, C1, C2>(
    camera: String,
    on_robot_pose_update: C0,
    on_photon_update: C1,
    on_dest_update: C2,
) -> Result<!, PhotonWorkerError>
where
    C0: for<'f> Fn(&'f mut Publisher<RawData>, Pose2d) -> BoxFuture<'f, ()> + ThreadSafe,
    C1: for<'f> Fn(&'f mut Publisher<RawData>, PhotonResult) -> BoxFuture<'f, ()> + ThreadSafe,
    C2: for<'f> Fn(&'f mut Publisher<RawData>, Pose2d) -> BoxFuture<'f, ()> + ThreadSafe,
{
    let nt = Client::new(Default::default());

    let photon = nt.topic(format!("/photonvision/{camera}/rawData"));
    let pose = nt.topic("/robot/pose");
    let dest = nt.topic("/robot/dest");
    let path = nt.topic("/pathforger/path");

    let mut photon_sub = photon
        .subscribe(SubscriptionOptions {
            periodic: Some(Duration::from_millis(20)),
            ..Default::default()
        })
        .await;

    let mut pose_sub = pose
        .subscribe(SubscriptionOptions {
            periodic: Some(Duration::from_millis(20)),
            ..Default::default()
        })
        .await;

    let mut dest_sub = dest
        .subscribe(SubscriptionOptions {
            periodic: Some(Duration::from_millis(20)),
            ..Default::default()
        })
        .await;

    let mut path_pub = path
        .publish::<RawData>(Properties {
            persistent: Some(false),
            retained: Some(true),
            cached: Some(true),
            ..Default::default()
        })
        .await?;

    let rt = runtime::Builder::new_multi_thread().enable_all().build()?;

    loop {
        let photon_res = photon_sub.recv().await?;
        let pose_res = pose_sub.recv().await?;
        let dest_res = dest_sub.recv().await?;

        match photon_res {
            ReceivedMessage::Updated((_, value)) => {
                if let Some(bytes) = value.as_slice() {
                    let result = deserialize(bytes)?;
                    rt.block_on(on_photon_update(&mut path_pub, result));
                };
            }
            _ => {}
        }

        match pose_res {
            ReceivedMessage::Updated((_, value)) => {
                if let Some(bytes) = value.as_slice() {
                    let pose = deserialize(bytes)?;
                    rt.block_on(on_robot_pose_update(&mut path_pub, pose));
                };
            }
            _ => {}
        }

        match dest_res {
            ReceivedMessage::Updated((_, value)) => {
                if let Some(bytes) = value.as_slice() {
                    let dest = deserialize(bytes)?;
                    rt.block_on(on_dest_update(&mut path_pub, dest));
                };
            }
            _ => {}
        }
    }
}
