#!/bin/bash

# --------------------------------------------------
# スクリプト実行前の準備として、別のターミナルで以下を実行する
# --------------------------------------------------
# - 古いコンテンツを削除
# $ dfx stop
# $ rm -rf .dfx

# - キャニスター実行環境を起動する
# $ dfx start --clean
# --------------------------------------------------


# Compile
rustup target add wasm32-unknown-unknown

## Trouble `sh: webpack: command not found`
# npm install

# Setup user principal
# Panicチェックで使用するユーザーを作成する
dfx identity new --disable-encryption unregistered_user
# デプロイ等に使用するdefaultユーザーをセットする
dfx identity use default
export ROOT_PRINCIPAL=$(dfx identity get-principal)

# Deploy
dfx deploy

echo "# ----------------------"
echo "#      デバイスの登録      "
echo "# ----------------------"
# デバイス1を登録する
dfx canister call encrypted_notes_backend registerDevice '("DEVICE_1", "PUBLIC_KEY_1")'
# (true)

# デバイス2を登録する
dfx canister call encrypted_notes_backend registerDevice '("DEVICE_2", "PUBLIC_KEY_2")'
# (true)

# [Error]: 同じデバイス名は登録できないことを確認する
dfx canister call encrypted_notes_backend registerDevice '("DEVICE_2", "PUBLIC_KEY_2")'
# (false)

# 登録したデバイスを取得する
dfx canister call encrypted_notes_backend getDevices
# (
#   vec {
#     record { "DEVICE_2"; "PUBLIC_KEY_2" };
#     record { "DEVICE_1"; "PUBLIC_KEY_1" };
#   },
# )

echo "# -----------------"
echo "#      鍵の登録     "
echo "# -----------------"
# まだseedが登録されていないことを確認する
dfx canister call encrypted_notes_backend isSeed
# (true)

# Encrypted Secretをアップロードする
## `Err(SecretError::Unknown)`: 登録したPublic Key以外ではエラーが返ることを確認する
dfx canister call encrypted_notes_backend uploadSeedSecret '("UNKNOWN_PUBLIC_KEY", "ENCRYPTED_SECRET_1")'
# (variant { Err = variant { Unknown } })

dfx canister call encrypted_notes_backend uploadSeedSecret '("PUBLIC_KEY_1", "ENCRYPTED_SECRET_1")'
# (variant { Ok })

# seedが登録されたことを確認する
dfx canister call encrypted_notes_backend isSeed
# (false)

# アップロードされたEncrypted Secretを取得する
dfx canister call encrypted_notes_backend getEncryptedSecrets '("PUBLIC_KEY_1")'
# (variant { Ok = "ENCRYPTED_SECRET_1" })

echo "# -----------------"
echo "#      鍵の同期     "
echo "# -----------------"
# `Err(SecretError::NotSynced)`: 非同期のPublic Keyに対してEncrypted Secretは取得できないことを確認する 
dfx canister call encrypted_notes_backend getEncryptedSecrets '("PUBLIC_KEY_2")'
# (variant { Err = variant { NotSynced } })

# 未同期の鍵一覧を取得する
dfx canister call encrypted_notes_backend getUnsyncedPublicKeys
# (vec { "PUBLIC_KEY_2" })

# 同期した鍵のペア（Public Key - Encrypted Secret）をアップロードする
dfx canister call encrypted_notes_backend uploadEncryptedSecrets '(vec { record {"PUBLIC_KEY_2"; "ECNRYPTED_SECRET_2"}})'
# ()

# Public Keyに対応するEncrypted Secretを取得する
dfx canister call encrypted_notes_backend getEncryptedSecrets '("PUBLIC_KEY_2")'
# (variant { Ok = "ECNRYPTED_SECRET_2" })

echo "# ----------------------"
echo "#      デバイスの削除     "
echo "# ----------------------"
# デバイス2を削除する
dfx canister call encrypted_notes_backend deleteDevice '("DEVICE_2")'
# ()

# デバイス一覧を取得して、デバイス2が削除されたことを確認する
dfx canister call encrypted_notes_backend getDevices
# (vec { record { "DEVICE_1"; "PUBLIC_KEY_1" } })

# デバイス2の鍵情報も削除されたことを確認する
dfx canister call encrypted_notes_backend getEncryptedSecrets '("PUBLIC_KEY_2")'
# (variant { Err = variant { Unknown } })

echo "# ---------------------"
echo "#      ノートの操作     "
echo "# ---------------------"
# ノートを追加する
dfx canister call encrypted_notes_backend addNote '("First text!")'
# (0 : nat)
dfx canister call encrypted_notes_backend addNote '("Second text!")'
# (1 : nat)

# ノート一覧を取得する
dfx canister call encrypted_notes_backend getNotes
#(
#  vec {
#    record { id = 0 : nat; encrypted_text = "First text!" };
#    record { id = 1 : nat; encrypted_text = "Second text!" };
#  },
#)

# ノートを編集する
dfx canister call encrypted_notes_backend updateNote '(0, "Edit first text!")'
# ()

# ノート一覧を取得して、テキストが編集されていることを確認する
dfx canister call encrypted_notes_backend getNotes
#(
#  vec {
#    record { id = 0 : nat; encrypted_text = "Edit first text!" };
#    record { id = 1 : nat; encrypted_text = "Second text!" };
#  },
#)

# ノートを削除する
dfx canister call encrypted_notes_backend deleteNote '(1)'
# ()

# ノート一覧を取得して、削除されたことを確認する
dfx canister call encrypted_notes_backend getNotes
#(vec { record { id = 0 : nat; encrypted_text = "Edit first text!" } })

echo "# ------------------"
echo "#      Panicked     "
echo "# ------------------"
# [ユーザーのエラー]
# anonymous principal(匿名ユーザー)を使用する
echo "PANICKED TEST 1: "
PANICKED_MSG_1=`dfx --identity anonymous canister call encrypted_notes_backend registerDevice '("anonymous_DEVICE", "anonymous_PUBLIC_KEY")' 2>&1`
if [ $? -eq 255 ]; then
    echo Ok
else
    echo Error
fi

# 登録していないPrincipalで操作しようとする
echo "PANICKED TEST 2: "
PANICKED_MSG_1=`dfx --identity unregistered_user canister call encrypted_notes_backend getNotes 2>&1`
if [ $? -eq 255 ]; then
    echo Ok
else
    echo Error
fi

# [鍵に関するエラー]
# 1つしか登録されていないデバイスを削除する
echo "PANICKED TEST 3: "
PANICKED_MSG_2=`dfx canister call encrypted_notes_backend deleteDevice '("DEVICE_1")' 2>&1`
if [ $? -eq 255 ]; then
    echo Ok
else
    echo Error
fi

# 後処理
dfx identity remove unregistered_user