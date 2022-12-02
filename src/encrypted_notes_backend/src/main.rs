use crate::devices_store::*;
use crate::notes_store::*;
use crate::store::{DEVICES_STORE, NOTES_STORE};
use ic_cdk::export::Principal;
use ic_cdk_macros::*;

mod devices_store;
mod notes_store;
mod store;

fn main() {}

#[query(name = "getDevices")]
fn get_devices(caller: Principal) -> Vec<(DeviceAlias, PublicKey)> {
    // TODO ユーザーが登録されているかチェック
    DEVICES_STORE.with(|devices_store| devices_store.borrow().get_devices(caller))
}

#[update(name = "registerDevice")]
fn register_device(caller: Principal, device_alias: DeviceAlias, public_key: PublicKey) -> bool {
    // TODO: 登録されている`alias`と`public_key`の数をチェック

    DEVICES_STORE.with(|devices_store| {
        devices_store
            .borrow_mut()
            .register_device(caller, device_alias, public_key)
    })
}

#[update(name = "deleteDevice")]
fn delete_device(caller: Principal, device_alias: DeviceAlias) {
    // TODO ユーザーが登録されているかチェック

    DEVICES_STORE.with(|devices_store| {
        devices_store
            .borrow_mut()
            .delete_device(caller, device_alias)
    })
}

#[query(name = "getNotes")]
fn get_notes(caller: Principal) -> Vec<EncryptedNote> {
    // TODO ユーザーが登録されているかチェック

    NOTES_STORE.with(|notes_store| notes_store.borrow().get_notes(caller))
}

#[update(name = "addNote")]
fn add_note(caller: Principal, encrypted_text: String) -> u128 {
    // TODO ユーザーが登録されているかチェック

    // TODO: Stringの文字数をチェック

    NOTES_STORE.with(|notes_store| notes_store.borrow_mut().add_note(caller, encrypted_text))
}

#[update(name = "updateNote")]
fn update_note(caller: Principal, update_id: u128, update_text: String) {
    // TODO ユーザーが登録されているか(匿名アカウントではないか)チェック

    // TODO: Stringの文字数をチェック

    NOTES_STORE.with(|notes_store| {
        notes_store
            .borrow_mut()
            .update_note(caller, update_id, update_text)
    });
}

#[update(name = "deleteNote")]
fn delete_note(caller: Principal, delete_id: u128) {
    // TODO ユーザーが登録されているかチェック

    NOTES_STORE.with(|notes_store| notes_store.borrow_mut().delete_note(caller, delete_id))
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

        // テキスト1を削除する
        delete_note(principal, id_1);
        // ノートを再取得する
        let notes = get_notes(principal);
        assert_eq!(notes.len(), 1);
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

        // ノートを更新する
        let update_text = "Update text!".to_string();

        update_note(principal, id, update_text.clone());

        // ノートを再取得する
        let notes = get_notes(principal);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].encrypted_text, update_text);
    }
}
