use crate::store::NOTES_STORE;
use candid::CandidType;
use ic_cdk::export::Principal;
use serde::Deserialize;
use std::collections::hash_map::Entry::*;
use std::collections::HashMap;

pub type EncryptedSecret = String;
pub type DeviceAlias = String;
pub type PublicKey = String;

#[derive(Debug, CandidType, Deserialize, PartialEq, Eq)]
pub enum SecretError {
    Unknown,
    NotSynced,
}

pub type UploadSecretResult = Result<(), SecretError>;
pub type GetSecretResult = Result<EncryptedSecret, SecretError>;

#[derive(Default)]
pub struct DevicesStore {
    pub aliases: HashMap<Principal, HashMap<DeviceAlias, PublicKey>>,
    pub keys: HashMap<Principal, HashMap<PublicKey, EncryptedSecret>>,
}

impl DevicesStore {
    pub fn get_devices(&self, caller: Principal) -> Vec<(DeviceAlias, PublicKey)> {
        match self.aliases.get(&caller) {
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
        match self.aliases.entry(caller) {
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

                // 鍵を保存するkeysにユーザーを割り当てる
                self.keys = HashMap::new();
                self.keys.insert(caller, HashMap::new());

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
        let user_aliases = self
            .aliases
            .get_mut(&caller)
            .expect("No user is registered.");
        // 登録されているデバイスが残り1個のときはエラーとする
        assert!(user_aliases.len() > 1);

        // デバイスの削除
        let public_key = user_aliases.remove(&device_alias);
        // デバイスに関連するKeyのデータを削除
        if let Some(pk) = public_key {
            self.keys
                .get_mut(&caller)
                .expect("No user is registered.")
                .remove(&pk);
        }
    }

    pub fn is_seed(&self, caller: Principal) -> bool {
        let user_keys = self.keys.get(&caller).expect("No user is registered.");
        user_keys.is_empty()
    }

    // `keys`に公開鍵と関連するシークレットを格納する関数。
    // この関数は、ユーザーがすでに少なくとも1つの公開鍵と秘密を保存している場合は、使用しない。
    // そのため、フロントエンド側で、`is_seed`の結果を見て関数をコールするべきである。
    pub fn upload_seed_secret(
        &mut self,
        caller: Principal,
        public_key: PublicKey,
        encrypted_secret: EncryptedSecret,
    ) -> UploadSecretResult {
        let user_aliases = self.aliases.get(&caller).expect("No user is registered.");

        if !Self::is_known_public_key(user_aliases, public_key.clone()) {
            return Err(SecretError::Unknown);
        }

        let user_keys = self.keys.get_mut(&caller).expect("No user is registered.");
        user_keys.insert(public_key, encrypted_secret);
        Ok(())
    }
    // まだ同期されていないデバイスの公開鍵を返す関数
    pub fn get_unsynced_public_keys(&self, caller: Principal) -> Vec<PublicKey> {
        let user_keys = self.keys.get(&caller).expect("No user is registered.");

        match self.aliases.get(&caller) {
            Some(user_aliases) => user_aliases
                .values()
                .filter(|value| !user_keys.contains_key(*value))
                .cloned()
                .collect::<Vec<PublicKey>>(),
            None => {
                vec![]
            }
        }
    }

    // 暗号化（同期）したEncrypted Secretをアップロードする関数
    pub fn upload_encrypted_secrets(
        &mut self,
        caller: Principal,
        keys: Vec<(PublicKey, EncryptedSecret)>,
    ) {
        let user_aliases = self.aliases.get(&caller).expect("No user is registered.");
        let user_keys = self.keys.get_mut(&caller).expect("No user is registered.");
        for (pk, es) in keys {
            if !Self::is_known_public_key(user_aliases, pk.clone()) {
                continue;
            }
            user_keys.entry(pk).or_insert(es);
        }
    }

    // 保有するPublic keyに対応するEncrypted Secretを取得する関数
    //
    // Principal: ユーザーが存在するかどうかをチェックで弾かれる
    // PublicKey: キーがあるかどうかでチェック弾かれる
    // EncryptedSecret: getしたときに、なかったら（からだったら）弾かれる
    pub fn get_encrypted_secrets(
        &self,
        caller: Principal,
        public_key: PublicKey,
    ) -> GetSecretResult {
        // DEVICE_STOREに保存されていないPublic Keyのとき
        let user_aliases = self.aliases.get(&caller).expect("No user is registered.");
        if !Self::is_known_public_key(user_aliases, public_key.clone()) {
            return Err(SecretError::Unknown);
        }

        let user_keys = self.keys.get(&caller).expect("No user is registered.");
        match user_keys.get(&public_key) {
            // Encrypted Secretが保存されていないとき
            None => Err(SecretError::NotSynced),
            // Encrypted Secretが保存されているとき
            Some(encrypted_secret) => Ok(encrypted_secret.to_string()),
        }
    }

    // Public Keyがaliasesに保存されているかを確認する関数
    //
    // [return]
    // `true`: 保存されているとき
    // `false`: 保存されていないとき
    fn is_known_public_key(
        user_aliases: &HashMap<DeviceAlias, PublicKey>,
        public_key: PublicKey,
    ) -> bool {
        user_aliases.values().any(|value| *value == public_key)
    }
}
