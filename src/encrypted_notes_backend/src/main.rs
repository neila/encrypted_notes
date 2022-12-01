use crate::notes_store::*;
use crate::store::NOTES_STORE;
use ic_cdk::export::Principal;
use ic_cdk_macros::*;
use std::collections::btree_map::Entry::*;
use std::{cell::RefCell, collections::BTreeMap};

mod notes_store;
mod store;

type DeviceAlias = String;
type PublicKey = String;

type DeviceStore = BTreeMap<DeviceAlias, PublicKey>;

thread_local! {
    static DEVICE_STORE: RefCell<BTreeMap<Principal, DeviceStore>> = RefCell::default();
}

fn main() {}

#[query(name = "getDevices")]
fn get_devices(caller: Principal) -> Vec<(DeviceAlias, PublicKey)> {
    // TODO ユーザーが登録されているかチェック
    DEVICE_STORE.with(|device_ref| {
        let device = device_ref.borrow();
        match device.get(&caller) {
            Some(devices) => devices
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<Vec<(DeviceAlias, PublicKey)>>(),
            None => Vec::new(),
        }
    })
}

#[update(name = "registerDevice")]
fn register_device(caller: Principal, device_alias: DeviceAlias, public_key: PublicKey) -> bool {
    // TODO: 登録されている`alias`と`public_key`の数をチェック

    DEVICE_STORE.with(|device_ref| {
        let mut writer = device_ref.borrow_mut();

        // entry()に渡す`key`は、そのまま要素としてインサートされるので、値渡しを行う点に注意
        match writer.entry(caller) {
            // エントリーが空いている（ユーザーが初めてデバイスを登録する）とき
            Vacant(empty_entry) => {
                // TODO 新たにユーザーが追加できるか、量をチェック

                // 既にノートが割り当てられていたらエラーとする
                let has_note =
                    NOTES_STORE.with(|notes_store_ref| notes_store_ref.borrow().has_note(caller));
                assert!(!has_note);

                // デバイスエイリアスと公開鍵を保存する
                let mut new_device = BTreeMap::new();
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
    })
}

#[update(name = "deleteDevice")]
fn delete_device(caller: Principal, device_alias: DeviceAlias) {
    // TODO ユーザーが登録されているかチェック

    DEVICE_STORE.with(|device_ref| {
        let mut writer = device_ref.borrow_mut();

        let device_store = writer.get_mut(&caller).expect("No user is registered.");
        // 登録されているデバイスが残り1個のときはエラーとする
        assert!(device_store.len() > 1);

        // デバイスの削除
        device_store.remove(&device_alias);
    });
}

#[query(name = "getNotes")]
fn get_notes(caller: Principal) -> Vec<EncryptedNote> {
    // TODO ユーザーが登録されているかチェック

    NOTES_STORE.with(|notes_store_ref| notes_store_ref.borrow().get_notes(caller))
}

#[update(name = "addNote")]
fn add_note(caller: Principal, encrypted_text: String) -> u128 {
    // TODO ユーザーが登録されているかチェック

    // TODO: Stringの文字数をチェック

    NOTES_STORE.with(|notes_store_ref| {
        notes_store_ref
            .borrow_mut()
            .add_note(caller, encrypted_text)
    })
}

#[update(name = "updateNote")]
// fn update_note(caller: Principal, update_note: EncryptedNote) {
fn update_note(caller: Principal, update_id: u128, update_text: String) {
    // TODO ユーザーが登録されているか(匿名アカウントではないか)チェック

    // TODO: Stringの文字数をチェック

    NOTES_STORE.with(|notes_store_ref| {
        notes_store_ref
            .borrow_mut()
            .update_note(caller, update_id, update_text)
    });
}

#[update(name = "deleteNote")]
fn delete_note(caller: Principal, delete_id: u128) {
    // TODO ユーザーが登録されているかチェック

    NOTES_STORE.with(|notes_store_ref| notes_store_ref.borrow_mut().delete_note(caller, delete_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const TEST_ACCOUNT_1: &str = "h4a5i-5vcfo-5rusv-fmb6m-vrkia-mjnkc-jpoow-h5mam-nthnm-ldqlr-bqe";

    #[test]
    fn test_register_and_get_devices() {
        let device_info = ("Brave".to_string(), "TEST_KEY".to_string());

        // デバイスを登録する
        let principal = Principal::from_str(TEST_ACCOUNT_1).unwrap();
        let res = register_device(principal, device_info.0.clone(), device_info.1.clone());

        assert!(res);

        // デバイス一覧を取得する
        let device_list = get_devices(principal);

        assert_eq!(device_list.len(), 1);
        assert_eq!(device_list[0], device_info);
    }

    #[test]
    fn test_register_and_delete_device() {
        let device_info_1 = ("Brave".to_string(), "TEST_KEY".to_string());
        let device_info_2 = ("Chrome".to_string(), "TEST_KEY".to_string());

        // デバイス1を登録する
        let principal = Principal::from_str(TEST_ACCOUNT_1).unwrap();
        let res = register_device(principal, device_info_1.0.clone(), device_info_1.1.clone());
        assert!(res);

        // デバイス2を登録する
        let principal = Principal::from_str(TEST_ACCOUNT_1).unwrap();
        let res = register_device(principal, device_info_2.0.clone(), device_info_2.1.clone());
        assert!(res);

        // デバイス一覧を取得する
        let device_list = get_devices(principal);
        assert_eq!(device_list.len(), 2);

        // デバイス1を削除する
        delete_device(principal, device_info_1.0);

        // デバイス一覧を取得する
        let device_list = get_devices(principal);
        assert_eq!(device_list.len(), 1);
    }

    #[test]
    fn test_register_device_duplication_err() {
        let device_info = ("Brave".to_string(), "TEST_KEY".to_string());

        // デバイスを登録する
        let principal = Principal::from_str(TEST_ACCOUNT_1).unwrap();
        let res = register_device(principal, device_info.0.clone(), device_info.1.clone());
        assert!(res);

        // 同じデバイスエイリアスを再度登録しようとする
        let principal = Principal::from_str(TEST_ACCOUNT_1).unwrap();
        let res = register_device(principal, device_info.0.clone(), device_info.1.clone());

        // false
        assert!(!res);
    }

    #[test]
    #[should_panic(expected = "assertion failed: device_store.len() > 1")]
    fn test_delete_device_err() {
        let device_info = ("Brave".to_string(), "TEST_KEY".to_string());

        // デバイスを登録する
        let principal = Principal::from_str(TEST_ACCOUNT_1).unwrap();
        let _res = register_device(principal, device_info.0.clone(), device_info.1.clone());

        delete_device(principal, device_info.0);
    }

    #[test]
    fn add_and_delete_note() {
        let device_info = ("Brave".to_string(), "TEST_KEY".to_string());

        // デバイスを登録する
        let principal = Principal::from_str(TEST_ACCOUNT_1).unwrap();
        let res = register_device(principal, device_info.0.clone(), device_info.1.clone());
        assert!(res);

        // テキスト1を追加する
        let text_1 = "My first text!".to_string();
        let id_1 = add_note(principal, text_1);

        // テキスト2を追加する
        let text_2 = "My second text!".to_string();
        let _id_2 = add_note(principal, text_2);

        // ノートを取得する
        let notes = get_notes(principal);
        assert_eq!(notes.len(), 2);
        println!("{:?}", notes); // TODO delete

        // テキスト1を削除する
        delete_note(principal, id_1);
        // ノートを再取得する
        let notes = get_notes(principal);
        assert_eq!(notes.len(), 1);
        println!("{:?}", notes); // TODO delete
    }

    #[test]
    fn add_and_update_note() {
        let device_info = ("Brave".to_string(), "TEST_KEY".to_string());

        // デバイスを登録する
        let principal = Principal::from_str(TEST_ACCOUNT_1).unwrap();
        let res = register_device(principal, device_info.0.clone(), device_info.1.clone());
        assert!(res);

        // テキストを追加する
        let text = "My first text!".to_string();
        let id = add_note(principal, text);

        // ノートを取得する
        let notes = get_notes(principal);
        assert_eq!(notes.len(), 1);
        println!("{:?}", notes); // TODO delete

        // ノートを更新する
        let update_text = "Update text!".to_string();

        update_note(principal, id, update_text.clone());

        // ノートを再取得する
        let notes = get_notes(principal);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].encrypted_text, update_text);
        println!("{:?}", notes); // TODO delete
    }
}
