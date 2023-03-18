# 1.環境構築

### rust の環境構築

以下コマンドを実行

`curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh`

[Rust 公式ドキュメント](https://doc.rust-lang.org/book/ch01-01-installation.html)より

### github からクローン

`git clone git@github.com:tosaken1116/rust_mosaic_art.git`

`cd rust_mosaic_art`

# 2.画像の用意

-   seed_images ディレクトリ配下に素材となる画像を用意
-   source ディレクトリ配下に生成したい画像を`seed.jpg`として用意

# 3.実行

初回実行時(画像を追加した時)には以下を実行

`cargo run update`

画像生成のみなら以下を実行

`cargo run`
# 4.備考
`static THREAD_NUM: u32 = 6;`
の部分を変更すれば並行処理のスレッド数を変更可能

スペックに合わせてご利用ください
