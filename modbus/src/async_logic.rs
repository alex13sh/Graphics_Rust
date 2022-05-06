use crate::{
    Devices, Device, DeviceID,
    DeviceError, DeviceResult,
};

pub struct UpdateLoop {
    devices: Devices,
    devices_queue: HashMap<DeviceID, Arc<Device>>,
}

impl UpdateLoop {
    pub fn new() -> Self {
        let devices = modbus::Devices::new();

        Self {
            devices: devices,
            devices_queue: HashMap::new(),
        }
    }
}

impl UpdateLoop {

    pub fn update_device(&mut self, d: Arc<Device>) {
        self.devices_queue.insert(d.id().clone(), d);
    }

    pub async fn update_loop(&self) {
        self.update_new_values();

                // Обновлять устройства из очереди
        let devices = std::mem::take(&mut self.devices_queue);

        let mut devices_reconnect = Vec::new();
        for (_, d) in devices {
            let d2 = d.clone();
            let res = async move {
//              log::trace!(target: "modbus::update::update_new_values", "{:?}", d.id());
//                 d.clone().update_new_values().await?;
//              log::trace!(target: "modbus::update::update_async", "{:?}", d.id());
                d.clone().update_async(UpdateReq::ReadOnlyOrLogable).await
            }.await;

            if let Err(e) = res {
                println!("Error: {:?} {:?}", &e, d2.id());
                log::trace!(target: "modbus::update", "[error] {:?} {:?}", &e, d2.id());
                match e {
                DeviceError::TimeOut => {
                    devices_reconnect.push(d2);
//                  self.devices_disconnect = true;
                },
                _ => {}
                }
            } else if d2.id().id == 2 {
                log::info!(target: "modbus::update::ok", "Device: {:?}", d2.id());
            }
        }

        Self::reconnect(devices_reconnect).await
    }

    async fn update_new_values(&self) {
    // набор фючер для принятия новых значений устройств.
        let f_update_new_values: Vec<_> = self.devices.iter()
            .cloned().map(Device::update_new_values)
            .collect();

        for f in f_update_new_values {
            if let Err(err) = f.await {
    //              println!("f_update_new_values err: {:?}", err);
            }
        }
    }

    async fn reconnect(devices: Vec<Arc<Device>>) -> DeviceResult {
        let mut res = None;
        for d in devices_reconnect {
            if let Err(e) = d.reconnect().await {
                dbg!(e);
//                             self.devices_disconnect = true;
                res = Some(Err(DeviceError::ContextNull));
            }
        }
        res
    }
}
