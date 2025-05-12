use std::{collections::HashMap, sync::LazyLock};

use mirajazz::state::DeviceStateUpdate;
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use crate::{
    device::DeviceMessage,
    mappings::{COL_COUNT, ENCODER_COUNT, ROW_COUNT},
};

use openaction::OUTBOUND_EVENT_MANAGER;

pub static DISP_TX: LazyLock<Mutex<Option<Sender<DeviceMessage>>>> =
    LazyLock::new(|| Mutex::new(None));

/// This task juggles events between devices and OpenDeck, while keeping track of all the
/// connected devices and their channels
pub async fn dispatcher_task(mut disp_rx: Receiver<DeviceMessage>) {
    let mut devices: HashMap<String, Sender<DeviceMessage>> = HashMap::new();

    log::info!("Running dispatcher");

    loop {
        let message = disp_rx.recv().await.unwrap();

        log::debug!("Dispatcher got a message: {:#?}", message);

        match message {
            DeviceMessage::Connected(id, kind, device_tx) => {
                log::info!("Registering device {}", id);

                devices.insert(id.clone(), device_tx);

                if let Some(outbound) = OUTBOUND_EVENT_MANAGER.lock().await.as_mut() {
                    outbound
                        .register_device(
                            id.clone(),
                            kind.human_name(),
                            ROW_COUNT as u8,
                            COL_COUNT as u8,
                            ENCODER_COUNT as u8,
                            7, //stream deck plus
                        )
                        .await
                        .unwrap();
                }
            }
            DeviceMessage::Disconnected(id) => {
                log::info!("Removing device {}", id);

                devices.remove_entry(&id);

                if let Some(outbound) = OUTBOUND_EVENT_MANAGER.lock().await.as_mut() {
                    outbound.deregister_device(id.clone()).await.unwrap();
                }
            }
            DeviceMessage::ShutdownAll => {
                for (_id, device_tx) in devices.iter() {
                    device_tx.send(DeviceMessage::ShutdownAll).await.unwrap();
                }
            }
            DeviceMessage::Update(id, update) => {
                if devices.contains_key(&id) {
                    if let Some(outbound) = OUTBOUND_EVENT_MANAGER.lock().await.as_mut() {
                        match update {
                            DeviceStateUpdate::ButtonDown(key) => {
                                outbound.key_down(id, key).await.unwrap()
                            }
                            DeviceStateUpdate::ButtonUp(key) => {
                                outbound.key_up(id, key).await.unwrap()
                            }
                            DeviceStateUpdate::EncoderDown(encoder) => {
                                outbound.encoder_down(id, encoder).await.unwrap();
                            }
                            DeviceStateUpdate::EncoderUp(encoder) => {
                                outbound.encoder_up(id, encoder).await.unwrap();
                            }
                            DeviceStateUpdate::EncoderTwist(encoder, val) => {
                                outbound
                                    .encoder_change(id, encoder, val as i16)
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                } else {
                    log::error!("Received an event for unknown device: {}", id);
                }
            }
            DeviceMessage::SetImage(id, event) => {
                if let Some(device_tx) = devices.get(&id) {
                    device_tx
                        .send(DeviceMessage::SetImage(id, event.clone()))
                        .await
                        .unwrap();
                } else {
                    log::error!("Received an event for unknown device: {}", id);
                }
            }
            DeviceMessage::SetBrightness(id, brightness) => {
                if let Some(device_tx) = devices.get(&id) {
                    device_tx
                        .send(DeviceMessage::SetBrightness(id, brightness))
                        .await
                        .unwrap();
                } else {
                    log::error!("Received an event for unknown device: {}", id);
                }
            }
        }
    }
}
