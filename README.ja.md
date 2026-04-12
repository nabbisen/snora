# Snora Framework Documentation

## 1. 概要 (Overview)
**Snora** は、RustのピュアGUIライブラリ `iced` の上に構築された、宣言的かつ構造的なUIフレームワークです。

ローカルファーストなアプリケーション、特に背後で重い処理（AI推論やローカルデータベース検索など）が走る環境において、UIのスレッドをクリーンに保ち、開発者が「状態管理」と「レイアウトの論理的構造」にのみ集中できるように設計されています。

## 2. 哲学 (Philosophy)

Snoraの根底には、開発者とユーザーの双方から「摩擦（Friction）」を取り除くための3つの哲学があります。

* **Accessible by Default and by Design (ABDD)**
    アクセシビリティは後付けのオプションではありません。Snoraは、左から右（LTR）だけでなく、右から左（RTL）へのレイアウト切り替えをフレームワークの基礎レベルでサポートしています。物理的な「左右」ではなく、論理的な「開始（Start）」と「終了（End）」でUIを構築することで、あらゆる言語圏のユーザー、あるいは一時的な認知の切り替えが必要な環境に自然に適応します。
* **Frictionless Architecture（摩擦のないアーキテクチャ）**
    巨大な `match` 文や、使わないアイコンアセットによるビルド時間の肥大化を許容しません。必要なアイコンや機能は `Cargo` の `feature` フラグを通じて選択的にオプトインされ、コンパイラのデッドコード削除（DCE）を最大限に活かします。
* **純粋な関心事の分離**
    「データがどうあるべきか（Contract）」と「どう描画されるべきか（Render）」を厳格に分離しています。これにより、ドメインロジックのテストが極めて容易になります。

## 3. 設計方針 (Design Principles)

* **CoreとRenderの分離**: `snora-core` クレートは `iced` に一切依存しません。純粋なRustの列挙型（Enum）と構造体のみで構成され、UIの「規約」を定義します。
* **型の柔軟性と厳格さの同居**: アイコンの指定などにおいて、機能が無効な場合でも `Option<String>` のように直感的にフォールバックできる設計を採用し、開発者のタイピング負荷を軽減します（`From<&str>` の積極的な活用）。
* **統一されたフィードバックループ**: `Toast` や `Footer` のログ機能により、非同期処理のステータスやシステムのエラー（Failure）を、ユーザーへ一貫したデザインで通知する仕組みを標準で備えています。

## 4. 構成 (Structure)

Snoraはワークスペースとして以下の層に分かれています。

```text
snora-workspace/
├── snora-core/   # [規約層] UIコンポーネントのデータ構造 (LayoutDirection, Icon, Toast)
└── snora/        # [描画層] iced を用いた物理的なピクセルへの変換とレイアウト
```

依存関係の矢印は常に `snora` → `snora-core` となり、逆流することはありません。

---

## 5. チュートリアル: 最初の Snora アプリ

Snoraを使って、RTLレイアウトに対応したシンプルなアプリケーションを構築する手順です。

### Step 1: 依存関係の追加
アプリ側の `Cargo.toml` に `snora` を追加し、必要な機能（ここではLucideアイコン）を有効化します。

```toml
[dependencies]
iced = { version = "0.14", features = ["tokio"] }
snora = { version = "0", features = ["lucide-icons"] }
```

### Step 2: メニューとアイコンの定義
`snora::icons` を介して、型安全にアイコン付きのメニュー項目を構築します。featureフラグのおかげで、依存関係を直接記述する必要はありません。

```rust
use snora::{Icon, MenuItem, icons};

let items = vec![
    MenuItem {
        label: "設定".into(),
        icon: Some(Icon::Lucide(icons::Settings)), // Lucideアイコン
        action: Some(Message::OpenSettings),
    },
    MenuItem {
        label: "ホーム".into(),
        icon: Some("🏠".into()), // 文字列からの自動変換
        action: Some(Message::GoHome),
    },
];
```

### Step 3: レイアウトの論理方向の決定
RTL（右から左）か、LTR（左から右）かを指定します。このフラグ一つで、サイドバーの位置や要素の並び順が自動的に最適化されます。

```rust
use snora::LayoutDirection;

let direction = LayoutDirection::Rtl; // アラビア語対応や、UIの反転テストに
```

### Step 4: ページの組み上げと描画
`PageLayout` 構造体に各エリアのウィジェットを当てはめ、`build_layout` 関数に渡すだけで、すべての要素が論理的に正しい位置に配置されます。

```rust
use snora::PageLayout;
use snora::components::header::app_header;
use snora::layout::build_layout;

// ヘッダーの生成
let header = app_header("My Snora App", items, None);

// 骨格の定義
let layout = PageLayout {
    direction,
    header: Some(header),
    body: main_content_widget,
    aside: Some(sidebar_widget),
    footer: None,
    dialog: None, // todo: moved to AppLayout
    bottom_sheet: None, // todo: moved to AppLayout
    toasts: vec![], // todo: moved to AppLayout
};

// iced::Element への最終変換
build_layout(layout)
```

この堅牢でスケーラブルなUIの基盤があなたのためになれば幸いです。
