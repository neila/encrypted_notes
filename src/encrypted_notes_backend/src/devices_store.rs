use crate::store::NOTES_STORE;
use ic_cdk::export::Principal;
use std::collections::hash_map::Entry::*;
use std::collections::HashMap;

pub type DeviceAlias = String;
pub type PublicKey = String;

#[derive(Default)]
pub struct DevicesStore {
    pub devices_store: HashMap<Principal, HashMap<DeviceAlias, PublicKey>>,
}

impl DevicesStore {
    pub fn get_devices(&self, caller: Principal) -> Vec<(DeviceAlias, PublicKey)> {
        match self.devices_store.get(&caller) {
            Some(devices) => devices
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<Vec<(DeviceAlias, PublicKey)>>(),
            None => Vec::new(),
        }
    }

    pub fn register_device(
        &mut self,
        caller: Principal,
        device_alias: DeviceAlias,
        public_key: PublicKey,
    ) -> bool {
        // TODO: 登録されている`alias`と`public_key`の数をチェック

        // entry()に渡す`key`は、そのまま要素としてインサートされるので、値渡しを行う点に注意
        match self.devices_store.entry(caller) {
            // エントリーが空いている（ユーザーが初めてデバイスを登録する）とき
            Vacant(empty_entry) => {
                // TODO 新たにユーザーが追加できるか、量をチェック

                // 既にノートが割り当てられていたらエラーとする
                let has_note =
                    NOTES_STORE.with(|notes_store_ref| notes_store_ref.borrow().has_note(caller));
                assert!(!has_note);

                // デバイスエイリアスと公開鍵を保存する
                let mut new_device = HashMap::new();
                new_device.insert(device_alias, public_key);
                empty_entry.insert(new_device);

                // ユーザーにノートを割り当てる
                NOTES_STORE
                    .with(|notes_store_ref| notes_store_ref.borrow_mut().assign_note(caller));

                true
            }
            // エントリーが埋まっている（ユーザーが追加でデバイスを登録する）とき
            Occupied(mut device_entry) => {
                // TODO 新たにデバイスが追加できるか、一人当たりのMAX_DEVICE_COUNTをチェック

                let device = device_entry.get_mut();
                match device.entry(device_alias) {
                    // エイリアスが未登録のとき
                    Vacant(empty_entry) => {
                        empty_entry.insert(public_key);
                        true
                    }
                    // 既にエイリアスが登録されているとき
                    Occupied(_) => {
                        // 既に同じエイリアスが登録されているので、何もせずに`false`を返す
                        false
                    }
                }
            }
        }
    }

    pub fn delete_device(&mut self, caller: Principal, device_alias: DeviceAlias) {
        let device_store = self
            .devices_store
            .get_mut(&caller)
            .expect("No user is registered.");
        // 登録されているデバイスが残り1個のときはエラーとする
        assert!(device_store.len() > 1);

        // デバイスの削除
        device_store.remove(&device_alias);
    }
}
